use crate::page::PAGE_SIZE;
use crate::page::Page;


pub struct Leaf { pub node: Node }

pub struct Node { pub page: Page }

impl Node {
    pub fn new(page: Page) -> Self {
        let mut node = Node { page: page };
        node.set_number_of_pointer(0);
        node.set_end_of_free_space(PAGE_SIZE as u16);
        node
    }

    pub fn add_slot<K, V>(&mut self, slot: &Slot<K, V>) where 
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        let bytes = slot.to_bytes();
        let end_of_free_space = self.end_of_free_space() as usize;
        let offset = end_of_free_space - bytes.len();
        self.page.set_bytes(offset, bytes);
    }

    fn set_number_of_pointer(&mut self, number: u16) {
        self.page.set_u16_bytes(0, number);
    }

    fn set_end_of_free_space(&mut self, number: u16) {
        self.page.set_u16_bytes(2, number);
    }

    // fn number_of_pointer(&self) -> u16 {
    //     self.page.u16_bytes(0)
    // }

    fn end_of_free_space(&self) -> u16 {
        self.page.u16_bytes(2)
    }
}

pub struct Slot<K, V> where
    K: SlotBytes + Clone,
    V: SlotBytes + Clone,
{
    key: K, value: V
}

impl<K, V> Slot<K, V> where
    K: SlotBytes + Clone,
    V: SlotBytes + Clone,
{
    pub fn new(key: K, value: V) -> Self {
        Slot { key: key, value: value }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut key_bytes = self.key.into_bytes().to_vec();
        let value_bytes = self.value.into_bytes().to_vec();
        key_bytes.extend(value_bytes);
        dbg!(&key_bytes);
        key_bytes
    }
}

pub trait SlotBytes {
    fn into_bytes(&self) -> Vec<u8>;
}

impl SlotBytes for u8 {
    fn into_bytes(&self) -> Vec<u8> {
        vec![1, self.clone()]
    }
}

impl SlotBytes for u16 {
    fn into_bytes(&self) -> Vec<u8> {
        let mut res = vec![2];
        res.extend(self.to_le_bytes().to_vec());
        res
    }
}

impl SlotBytes for u32 {
    fn into_bytes(&self) -> Vec<u8> {
        let mut res = vec![4];
        res.extend(self.to_le_bytes().to_vec());
        res
    }
}

impl SlotBytes for &str {
    fn into_bytes(&self) -> Vec<u8> {
        let bytes = self.bytes().collect::<Vec<_>>();
        let len = bytes.len() as u16;
        let mut res = len.to_le_bytes().to_vec();
        res.extend(bytes);
        res
    }
}