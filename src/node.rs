use std::convert::TryInto;
use std::marker::PhantomData;
use std::mem::size_of;
use std::ops::Range;

use crate::error::Error;
use crate::page::PAGE_SIZE;
use crate::page::Page;
use crate::slot::Slot;
use crate::slot::SlotBytes;


pub struct Leaf<K: Ord + SlotBytes, V> { pub node: Node<K, V> }

impl<K: Ord + SlotBytes, V> Leaf<K, V> {
    pub fn new(mut node: Node<K, V>) -> Self {
        node.set_node_type(NodeType::Leaf);
        Leaf { node: node }
    }
}

pub struct Branch<K: Ord + SlotBytes, V> { pub node: Node<K, V> }

impl<K: Ord + SlotBytes, V> Branch<K, V> {
    pub fn new(mut node: Node<K, V>) -> Self {
        node.set_node_type(NodeType::Branch);
        Branch { node: node }
    }

    pub fn set_max_page_id(&mut self, number: u16) {
        self.node.page.set_u16_bytes(6, number);
    }

    pub fn max_page_id(&self) -> u16 {
        self.node.page.u16_bytes(6)
    }
}

pub struct Node<K: Ord + SlotBytes, V> {
    pub page: Page,
    _phantom_key: PhantomData<fn() -> K>,
    _phantom_value: PhantomData<fn() -> V>,
}

pub enum NodeType { Leaf, Branch, }

const HEADER_LEN: usize = 8;
const LEN_OF_LEN_OF_POINTER: usize = 2;
const LEN_OF_LEN_OF_VALUE: usize = 2;

impl<K: Ord + SlotBytes, V> Node<K, V> {
    pub fn new(page: Page) -> Self {
        Node::<K, V> {
            page: page, 
            _phantom_key: PhantomData,
            _phantom_value: PhantomData,
        }
    }

    pub fn create(page: Page) -> Self {
        let mut node = Node::new(page);
        node.set_number_of_pointer(0);
        node.set_end_of_free_space(PAGE_SIZE as u16);
        node
    }

    pub fn insert(&mut self, slot: &Slot<K, V>) -> Result<(), Error> where
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        if self.is_full(slot) {
            return Err(Error::FullLeaf)
        } 
        self.add_slot(slot);
        self.insert_pointer(&slot.key);
        self.increment_number_of_pointer();
        Ok(())
    }

    pub fn search(&self, key: &K) -> Option<V> where V: SlotBytes {
        match self.search_slot_offset(key) {
            Some(offset) => {
                let len_of_len_of_key = 2;
                let len_of_key = std::mem::size_of::<K>();
                let start_of_len_of_value = (offset + len_of_len_of_key + len_of_key) as usize;
                let len_of_value = Self::offset_to_u16(&self.page.bytes, start_of_len_of_value);

                let start_of_value = start_of_len_of_value + LEN_OF_LEN_OF_VALUE;
                let bytes = &self.page.bytes[Self::range(start_of_value, len_of_value as usize)];
 
                Some(V::from_bytes(bytes, self.node_type()))
            },
            None => None,
        }
    }

    pub fn delete(&mut self, key: &K) -> Result<(), Error> {
        match self.search_pointer(key) {
            Some(pointer_index) => {
                let slot_offset = self.pointer_index_to_slot_offset(pointer_index);

                // delete slot
                let end_of_free_space = self.end_of_free_space() as usize;
                let slot_len = self.slot_len(slot_offset);
                if end_of_free_space < slot_offset {
                    let bytes = &mut self.page.bytes;
                    bytes.copy_within(end_of_free_space..slot_offset, end_of_free_space + slot_len);
                }
                &self.page.bytes[Self::range(end_of_free_space, slot_len)].fill(0);

                // end_of_free_space
                let end_of_free_space = self.end_of_free_space();
                self.set_end_of_free_space(end_of_free_space + slot_len as u16);

                // delete pointer
                let pointer_offset = Self::pointer_offset(pointer_index);
                let pointer_last = self.start_of_free_space();
                let range = pointer_offset + Self::pointer_size()..pointer_last;
                let bytes = &mut self.page.bytes;
                bytes.copy_within(range, pointer_offset);
                bytes[pointer_last - Self::pointer_size()..pointer_last].fill(0);

                // number_of_pointer
                self.decrement_number_of_pointer();

                // rewrite offset
                let range = self.pointers_range();
                let rewriting_indices = self.page.bytes[range].chunks(2)
                    .map(|chunk| Self::offset_to_u16(chunk, 0))
                    .enumerate()
                    .filter(|(_, offset)| offset < &(slot_offset as u16))
                    .collect::<Vec<_>>();

                for (pointer_index, slot_offset) in rewriting_indices.iter() {
                    let pointer_offset = Self::pointer_offset(pointer_index.clone());
                    self.page.set_u16_bytes(pointer_offset, slot_offset.clone() + slot_len as u16);
                }

                Ok(())
            },
            None => Err(Error::NotFound),
        }
    }

    pub fn keys<'a>(&self) -> Vec<K> where K: SlotBytes {
        let range = self.pointers_range();
        self.page.bytes[range].chunks(2)
            .map(|chunk| Self::offset_to_u16(chunk, 0))
            .map(|slot_offset| {
                let key_size = size_of::<K>();
                let offset = slot_offset as usize + key_size;
                let bytes = &self.page.bytes[Self::range(offset, Self::pointer_size())];
                K::from_bytes(bytes, self.node_type())
            })
            .collect::<Vec<K>>()
    }

    fn slot_len(&self, offset: usize) -> usize {
        let len_of_slot_key = LEN_OF_LEN_OF_POINTER + Self::pointer_size();
        let start_of_len_of_value = offset + len_of_slot_key;
        let len_of_value = Self::offset_to_u16(&self.page.bytes, start_of_len_of_value);

        len_of_slot_key + LEN_OF_LEN_OF_VALUE + len_of_value as usize
    }

    fn search_slot_offset(&self, key: &K) -> Option<usize> {
        self.search_pointer(key).and_then(|index| {
            Some(self.pointer_index_to_slot_offset(index))
        })
    }

    fn pointer_index_to_slot_offset(&self, index: usize) -> usize {
        let pointer = Self::pointer_offset(index.clone());
        Self::offset_to_u16(&self.page.bytes, pointer) as usize
    }

    fn search_pointer(&self, key: &K) -> Option<usize> {
        self.keys().binary_search(key).ok()
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

    fn offset_to_u16(chunk: &[u8], offset: usize) -> u16 {
        let bytes = &chunk[offset..offset + size_of::<u16>()];
        u16::from_le_bytes(bytes.try_into().unwrap())
    }

    fn pointers_range(&self) -> Range<usize> {
        HEADER_LEN..self.start_of_free_space()
    }

    fn start_of_free_space(&self) -> usize {
        let number_of_pointer = self.number_of_pointer();
        Self::pointer_offset(number_of_pointer as usize)
    }

    fn pointer_offset(index: usize) -> usize {
        HEADER_LEN + Self::pointer_size() * index
    }

    fn add_slot(&mut self, slot: &Slot<K, V>) where 
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        let bytes = slot.to_bytes();
        let end_of_free_space = self.end_of_free_space() as usize;
        let offset = end_of_free_space - bytes.len();
        self.page.set_bytes(offset, bytes);
        self.set_end_of_free_space(offset.try_into().unwrap());
    }

    fn insert_pointer(&mut self, key: &K) where K: SlotBytes {
        let keys = self.keys();
        let insertion_point = 
            if let Some(p) = keys.iter()
                .position(|k| key < k ) {
                p
            } else {
                keys.len()
            };
        let start_offset = HEADER_LEN + Self::pointer_size() * insertion_point;
        let end_offset = self.start_of_free_space();
        self.page.bytes.copy_within(start_offset..end_offset, start_offset + Self::pointer_size());
        self.page.set_u16_bytes(start_offset.into(), self.end_of_free_space())
    }

    fn increment_number_of_pointer(&mut self) {
        let current = self.number_of_pointer();
        self.set_number_of_pointer(current + 1);
    }

    fn decrement_number_of_pointer(&mut self) {
        let current = self.number_of_pointer();
        self.set_number_of_pointer(current - 1);
    }

    pub fn set_node_type(&mut self, node_type: NodeType) {
        match node_type {
            NodeType::Leaf => self.page.set_u16_bytes(4, u16::MIN),
            NodeType::Branch => self.page.set_u16_bytes(4, u16::MAX),
        }
    }

    pub fn node_type(&self) -> NodeType {
        match self.page.u16_bytes(4) {
            u16::MIN => NodeType::Leaf,
            u16::MAX => NodeType::Branch,
            v @ _ => panic!("invalid node type value: {}", v),
        }
    }

    fn pointer_size() -> usize {
        size_of::<u16>()
    }

    fn set_number_of_pointer(&mut self, number: u16) {
        self.page.set_u16_bytes(0, number);
    }

    fn set_end_of_free_space(&mut self, number: u16) {
        self.page.set_u16_bytes(2, number);
    }

    fn number_of_pointer(&self) -> u16 {
        self.page.u16_bytes(0)
    }

    fn end_of_free_space(&self) -> u16 {
        self.page.u16_bytes(2)
    }

    fn range(start: usize, len: usize) -> std::ops::Range<usize> {
        start..start + len
    }
}

#[cfg(test)]
mod test {
    use crate::error::Error;
    use crate::node::Node;
    use crate::page::Page;
    use crate::page::PAGE_SIZE;
    use crate::slot::Slot;
    use super::LEN_OF_LEN_OF_POINTER;
    use super::LEN_OF_LEN_OF_VALUE;


    type TestNode = Node::<u16, String>;

    #[test]
    fn test_pointers_sorted() {
        let page = Page::new(Default::default());
        let mut node = TestNode::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        let _ = node.insert(&Slot::new(7u16, "ありがと".to_string()));
        let _ = node.insert(&Slot::new(5u16, "defg".to_string()));
        let _ = node.insert(&Slot::new(1u16, "ぽ".to_string()));
        let keys = node.keys();
        println!("{:?}", &node.page);
        assert_eq!(keys, [1, 2, 5, 7]);
    }

    #[test]
    fn test_pointers_full() {
        let page = Page::new(Default::default());
        let mut node = TestNode::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        let _ = node.insert(&Slot::new(7u16, "ありがと".to_string()));
        let _ = node.insert(&Slot::new(5u16, "defg".to_string()));
        let res = node.insert(&Slot::new(1u16, "pppppp".to_string()));
        // let res = node.insert(&Slot::new(100u16, "あふれちゃう".to_string()));
        let pointers = node.keys();
        println!("{:?}", &node.page);
        assert_eq!(pointers, [2, 5, 7]);
        assert_eq!(res, Err(Error::FullLeaf));
    }

    #[test]
    fn test_search_hit() {
        let page = Page::new(Default::default());
        let mut node = TestNode::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        assert_eq!(node.search(&2u16), Some("abc".to_string()));
    }

    #[test]
    fn test_search_notfound() {
        let page = Page::new(Default::default());
        let mut node = TestNode::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        assert_eq!(node.search(&5u16), None);
    }

    #[test]
    fn test_search_remove_offset() {
        let page = Page::new(Default::default());
        let mut node = TestNode::create(page);
        let target_value = "defg";
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        let _ = node.insert(&Slot::new(7u16, "ありがと".to_string()));
        let _ = node.insert(&Slot::new(5u16, target_value.to_string()));
        assert_eq!(
            node.slot_len(node.search_slot_offset(&5).unwrap()), 
            LEN_OF_LEN_OF_POINTER + TestNode::pointer_size() +
            LEN_OF_LEN_OF_VALUE + target_value.len()
        )
    }

    #[test]
    fn test_delete_notfound() {
        let page = Page::new(Default::default());
        let mut node = TestNode::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        assert!(node.delete(&5).is_err());
    }

    #[test]
    fn test_delete_one() {
        let page = Page::new(Default::default());
        let mut node = TestNode::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        assert!(node.delete(&2).is_ok());
        let mut res = [0u8; PAGE_SIZE];
        res[2] = 64;
        assert_eq!(node.page.bytes, res);
    }

    #[test]
    fn test_delete_multi() {
        let mut node1 = TestNode::create(Page::new(Default::default()));
        let target_value = "defg";
        let _ = node1.insert(&Slot::new(2u16, "abc".to_string()));
        let _ = node1.insert(&Slot::new(7u16, "ありがと".to_string()));
        let _ = node1.insert(&Slot::new(5u16, target_value.to_string()));
        assert!(node1.delete(&5).is_ok());


        let mut node2 = TestNode::create(Page::new(Default::default()));
        let _ = node2.insert(&Slot::new(2u16, "abc".to_string()));
        let _ = node2.insert(&Slot::new(7u16, "ありがと".to_string()));
        assert_eq!(node1.page.bytes, node2.page.bytes);
    }

    #[test]
    fn test_delete_transfer() {
        let mut node1 = TestNode::create(Page::new(Default::default()));
        let _ = node1.insert(&Slot::new(13u16, "abc".to_string()));
        let _ = node1.insert(&Slot::new(7u16, "ぽぽ".to_string()));
        assert!(node1.delete(&13).is_ok());

        let mut node2 = TestNode::create(Page::new(Default::default()));
        let _ = node2.insert(&Slot::new(7u16, "ぽぽ".to_string()));

        assert_eq!(node1.page.bytes, node2.page.bytes);
    }
}