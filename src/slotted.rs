pub mod pointer;
mod fmt;
#[cfg(test)]
mod test;


use std::convert::TryInto;
use std::marker::PhantomData;
use std::ops::Range;

use crate::error::Error;
use crate::node::NodeType;
use crate::page::PAGE_SIZE;
use crate::page::Page;
use crate::slot::Slot;
use crate::slot::SlotBytes;
use crate::slotted::pointer::Pointer;


const HEADER_LEN: usize = 8;

pub struct Slotted<K: Ord + SlotBytes, V: SlotBytes, P: Pointer> {
    pub page: Page,
    _phantom_key: PhantomData<fn() -> K>,
    _phantom_value: PhantomData<fn() -> V>,
    _phantom_pointer: PhantomData<fn() -> P>,
}

impl<K: Ord + SlotBytes, V: SlotBytes, P: Pointer> Slotted<K, V, P> {
    pub fn new(page: Page) -> Self {
        Slotted::<K, V, P> {
            page: page, 
            _phantom_key: PhantomData,
            _phantom_value: PhantomData,
            _phantom_pointer: PhantomData,
        }
    }

    pub fn create(page: Page) -> Self {
        let mut slotted = Slotted::new(page);
        slotted.set_number_of_pointer(0);
        slotted.set_end_of_free_space(PAGE_SIZE as u16);
        slotted
    }

    pub fn insert(&mut self, slot: &Slot<K, V>) -> Result<(), Error> where
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        if self.is_full(slot) {
            return Err(Error::FullLeaf)
        } 
        self.add_slot(slot);
        self.insert_pointer(&slot);
        self.increment_number_of_pointer();
        Ok(())
    }

    pub fn search(&self, key: &K) -> Option<V> where V: SlotBytes {
        match self.search_slot_offset(key) {
            Some(pointer) => {
                let bytes = &self.page.bytes[pointer.value_range()];
                Some(V::from_bytes(bytes))
            },
            None => None,
        }
    }

    pub fn delete(&mut self, key: &K) -> Result<(), Error> {
        match self.search_pointer(key) {
            Some(pointer_index) => {
                let pointer = self.pointer_index_to_pointer(pointer_index);

                self.delete_slot(&pointer);

                // end_of_free_space
                let end_of_free_space = self.end_of_free_space();
                self.set_end_of_free_space(end_of_free_space + pointer.slot_size());

                self.delete_pointer(pointer_index);

                self.decrement_number_of_pointer();

                self.update_slot_offsets(pointer);

                Ok(())
            },
            None => Err(Error::NotFound),
        }
    }

    pub fn set_node_type(&mut self, node_type: NodeType) {
        let current = self.page.u16_bytes(0);
        match node_type {
            NodeType::Leaf => self.page.set_u16_bytes(0, current & 0x7FFF),
            NodeType::Branch => self.page.set_u16_bytes(0, current | 0x8000),
        }
    }

    pub fn pointers(&self) -> Vec<P> {
        self.page.bytes[self.pointers_range()].chunks(P::len())
            .map(|chunk| Self::offset_to_pointer(chunk, 0))
            .collect::<Vec<_>>()
    }

    pub fn keys<'a>(&self) -> Vec<K>
        where K: SlotBytes
    {
        let range = self.pointers_range();
        self.page.bytes[range].chunks(Self::pointer_size())
            .map(|chunk| Self::offset_to_pointer(chunk, 0))
            .map(|pointer| {
                let bytes = &self.page.bytes[pointer.key_range()];
                K::from_bytes(bytes)
            })
            .collect::<Vec<_>>()
    }

    pub fn slots(&self) -> Vec<(K, V)> {
        let pointers = self.page.bytes[self.pointers_range()].chunks(P::len())
            .map(|chunk| Self::offset_to_pointer(chunk, 0))
            .collect::<Vec<P>>();

        pointers.iter().map(|pointer| {
            let key = K::from_bytes(&self.page.bytes[pointer.key_range()]);
            let value = V::from_bytes(&self.page.bytes[pointer.value_range()]);
            (key, value)
        }).collect::<Vec<_>>()
    }

    fn is_full(&self, slot: &Slot<K, V>) -> bool where
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        let offset_pointer = self.start_of_free_space();

        let bytes = slot.to_bytes();
        let end_of_free_space = self.end_of_free_space() as usize;
        if end_of_free_space < bytes.len() {
            return true;
        }
        let offset_slot = end_of_free_space - bytes.len() - Self::pointer_size();

        offset_slot < offset_pointer
    }

    fn add_slot(&mut self, slot: &Slot<K, V>)
        where 
            K: SlotBytes + Clone,
            V: SlotBytes + Clone,
    {
        let bytes = slot.to_bytes();
        let end_of_free_space = self.end_of_free_space() as usize;
        let offset = end_of_free_space - bytes.len();
        self.page.set_bytes(offset, bytes);
        self.set_end_of_free_space(offset.try_into().unwrap());
    }

    fn insert_pointer(&mut self, slot: &Slot<K, V>)
        where K: SlotBytes + Clone,
              V: SlotBytes + Clone,
    {
        let keys = self.keys();
        let insertion_point = 
            if let Some(p) = keys.iter()
                .position(|k| &slot.key < k ) {
                p
            } else {
                keys.len()
            };
        let start_offset = Self::pointer_offset(insertion_point);
        let end_offset = self.start_of_free_space();
        self.page.bytes.copy_within(start_offset..end_offset, start_offset + Self::pointer_size());

        let mut pointer_bytes = self.end_of_free_space().to_le_bytes().to_vec();
        pointer_bytes.append(&mut slot.key_size().to_le_bytes().to_vec());
        pointer_bytes.append(&mut slot.value_size().to_le_bytes().to_vec());
        self.page.set_bytes(start_offset.into(), pointer_bytes);
    }

    fn delete_slot(&mut self, pointer: &impl Pointer) {
        let start_of_slots = self.end_of_free_space() as usize;
        let start_of_deleting_slot = pointer.slot_offset() as usize;
        let slot_len = pointer.slot_size() as usize;
        if start_of_slots < start_of_deleting_slot {
            let bytes = &mut self.page.bytes;
            bytes.copy_within(start_of_slots..start_of_deleting_slot, start_of_slots + slot_len);
        }
        &self.page.bytes[range(start_of_slots, slot_len)].fill(0);
    }

    fn delete_pointer(&mut self, pointer_index: usize) {
        let start_of_deleting_pointer = Self::pointer_offset(pointer_index);
        let start_of_pointers = start_of_deleting_pointer + Self::pointer_size();
        let end_of_pointers = self.start_of_free_space();
        let range = start_of_pointers..end_of_pointers;
        let bytes = &mut self.page.bytes;
        bytes.copy_within(range, start_of_deleting_pointer);
        bytes[end_of_pointers - Self::pointer_size()..end_of_pointers].fill(0);
    }

    fn update_slot_offsets(&mut self, pointer: impl Pointer) {
        let range = self.pointers_range();
        let rewriting_indices = self.page.bytes[range].chunks(Self::pointer_size())
            .map(|chunk| Self::offset_to_pointer(chunk, 0))
            .enumerate()
            .filter(|(_, ptr)| ptr.slot_offset() < pointer.slot_offset())
            .collect::<Vec<_>>();

        for (ptr_index, ptr) in rewriting_indices.iter() {
            let ptr_offset = Self::pointer_offset(ptr_index.clone());
            self.page.set_u16_bytes(ptr_offset, ptr.slot_offset() + pointer.slot_size());
        }
    }

    fn search_slot_offset(&self, key: &K) -> Option<P> {
        self.search_pointer(key).map(|key_index| {
            self.pointer_index_to_pointer(key_index)
        })
    }
}

impl<K, V: SlotBytes, P: Pointer> Slotted<K, V, P>
    where K: Ord + SlotBytes 
{
    fn search_pointer(&self, key: &K) -> Option<usize> {
        self.keys().binary_search(key).ok()
    }

    fn pointer_index_to_pointer(&self, key_index: usize) -> P {
        let pointer = Self::pointer_offset(key_index.clone());
        Self::offset_to_pointer(&self.page.bytes, pointer)
    }

    fn offset_to_pointer(chunk: &[u8], offset: usize) -> P {
        let bytes = &chunk[range(offset, P::len())];
        P::from_bytes(bytes)
    }

    fn pointers_range(&self) -> Range<usize> {
        HEADER_LEN..self.start_of_free_space()
    }

    fn start_of_free_space(&self) -> usize {
        let number_of_pointer = self.number_of_pointer();
        Self::pointer_offset(number_of_pointer as usize)
    }

    fn increment_number_of_pointer(&mut self) {
        let current = self.number_of_pointer();
        self.set_number_of_pointer(current + 1);
    }

    fn decrement_number_of_pointer(&mut self) {
        let current = self.number_of_pointer();
        self.set_number_of_pointer(current - 1);
    }

    fn set_number_of_pointer(&mut self, number: u16) {
        let number = number & 0x7FFF;
        let current = self.page.u16_bytes(0) & 0x8000;
        self.page.set_u16_bytes(0, number | current);
    }

    fn set_end_of_free_space(&mut self, number: u16) {
        self.page.set_u16_bytes(2, number);
    }

    fn number_of_pointer(&self) -> u16 {
        self.page.u16_bytes(0) & 0x7FFF
    }

    fn end_of_free_space(&self) -> u16 {
        self.page.u16_bytes(2)
    }

    fn pointer_offset(index: usize) -> usize {
        HEADER_LEN + Self::pointer_size() * index
    }

    fn pointer_size() -> usize {
        P::len()
    }    
}

fn range(start: usize, len: usize) -> Range<usize> {
    start..start + len
}
