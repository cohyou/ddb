use std::convert::TryInto;
use std::marker::PhantomData;

use crate::page::PAGE_SIZE;
use crate::page::Page;
use crate::slot::Slot;
use crate::slot::SlotBytes;
use crate::slot::AsKey;


pub struct Leaf<K: Ord + AsKey, V> { pub node: Node<K, V> }

impl<K: Ord + AsKey, V> Leaf<K, V> {
    pub fn new(node: Node<K, V>) -> Self {
        Leaf { node: node }
    }
}

pub struct Node<K: Ord + AsKey, V> {
    pub page: Page,
    _phantom_key: PhantomData<fn() -> K>,
    _phantom_value: PhantomData<fn() -> V>,
}

pub enum NodeType { Leaf, Branch, }

impl<K: Ord + AsKey, V> Node<K, V> {
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

    pub fn insert(&mut self, slot: &Slot<K, V>) where
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        self.add_slot(&slot);
        self.insert_pointer(&slot.key);
        self.increment_number_of_pointer();
    }

    fn keys<'a>(&self) -> Vec<K> where K: AsKey {
        let header_len = 8 as usize;
        let number_of_pointer = self.number_of_pointer();
        let offset = header_len + 2 * number_of_pointer as usize;
        let range = header_len..offset as usize;
        self.page.bytes[range].chunks(2)
            .map(|chunk| {
                let chunk = &chunk[0..2];
                u16::from_le_bytes(chunk.try_into().unwrap())
            })
            .map(|offset| {
                let key_size = std::mem::size_of::<K>();
                let offset = offset as usize + key_size - 1;
                let bytes = &self.page.bytes[offset..offset + 2];
                K::from_bytes(bytes)
            })
            .collect::<Vec<K>>()
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

    fn insert_pointer(&mut self, key: &K) where K: AsKey {
        let keys = self.keys();
        let insert_point = 
            if let Some(p) = keys.iter()
                .position(|k| key < k ) {
                p
            } else {
                keys.len()
            };
        let header_len = 8usize;
        let number_of_pointer = self.number_of_pointer() as usize;
        let end_of_free_space = self.end_of_free_space();
        let end_offset = header_len + 2 * number_of_pointer;
        let start_offset = header_len + 2 * insert_point;
        self.page.bytes.copy_within(start_offset..end_offset, start_offset + 2);
        self.page.set_u16_bytes(start_offset.into(), end_of_free_space)
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

#[cfg(test)]
mod test {
    use crate::node::Node;
    use crate::page::Page;
    use crate::slot::Slot;

    #[test]
    fn test_pointers_sorted() {
        let page = Page::new(Default::default());
        let mut node = Node::<u16, &str>::create(page);
        node.insert(&Slot::new(2u16, "abc"));
        node.insert(&Slot::new(7u16, "ありがと"));
        node.insert(&Slot::new(5u16, "defg"));
        node.insert(&Slot::new(1u16, "ぽぽ"));
        let pointers = node.keys();
        println!("{:?}", &node.page);
        assert_eq!(pointers, [1, 2, 5, 7]);
    }
}