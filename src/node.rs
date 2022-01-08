use std::convert::TryInto;

use crate::page::PAGE_SIZE;
use crate::page::Page;


pub struct Leaf { pub node: Node }

impl Leaf {
    pub fn new(node: Node) -> Self {
        Leaf { node: node}
    }
}

pub struct Node { pub page: Page }

pub enum NodeType { Leaf, Branch, }

impl Node {
    pub fn new(page: Page) -> Self {
        Node { page: page }
    }

    pub fn create(page: Page) -> Self {
        let mut node = Node { page: page };
        node.set_number_of_pointer(0);
        node.set_end_of_free_space(PAGE_SIZE as u16);
        node
    }

    pub fn add<K, V>(&mut self, slot: &Slot<K, V>) where 
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        self.add_slot(&slot);
        self.add_pointer();
        self.increment_number_of_pointer();
    }

    fn add_slot<K, V>(&mut self, slot: &Slot<K, V>) where 
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        let bytes = slot.to_bytes();
        let end_of_free_space = self.end_of_free_space() as usize;
        let offset = end_of_free_space - bytes.len();
        self.page.set_bytes(offset, bytes);
        self.set_end_of_free_space(offset.try_into().unwrap());
    }

    fn add_pointer(&mut self) {
        let header_len = 8;
        let number_of_pointer = self.number_of_pointer();
        let end_of_free_space = self.end_of_free_space();
        let offset = header_len + 2 * number_of_pointer;
        self.page.set_u16_bytes(offset.into(), end_of_free_space)
    }

    fn increment_number_of_pointer(&mut self) {
        let current = self.number_of_pointer();
        self.set_number_of_pointer(current + 1);
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