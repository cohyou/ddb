use std::convert::TryInto;
use std::marker::PhantomData;

use crate::error::Error;
use crate::page::PAGE_SIZE;
use crate::page::Page;
use crate::slot::Slot;
use crate::slot::SlotBytes;


pub struct Leaf<K: Ord + SlotBytes, V> { pub node: Node<K, V> }

impl<K: Ord + SlotBytes, V> Leaf<K, V> {
    pub fn new(node: Node<K, V>) -> Self {
        Leaf { node: node }
    }
}

pub struct Node<K: Ord + SlotBytes, V> {
    pub page: Page,
    _phantom_key: PhantomData<fn() -> K>,
    _phantom_value: PhantomData<fn() -> V>,
}

pub enum NodeType { Leaf, Branch, }

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

    pub fn search(&self, key: &K) -> Option<V> where
        V: SlotBytes
    {
        let keys = self.keys();
        match &keys.binary_search(key) {
            Ok(index) => {
                let header_len = 8 as usize;
                let pointer = header_len + 2 * index;
                let bytes = &self.page.bytes[pointer..pointer + 2];
                let offset = u16::from_le_bytes(bytes.try_into().unwrap());
                let start_of_value = (offset + 3) as usize;
                let bytes = &self.page.bytes[start_of_value..start_of_value + 2];
                let len_of_value = u16::from_le_bytes(bytes.try_into().unwrap()) as usize;
                let start_of_value = start_of_value + std::mem::size_of::<u16>();
                let bytes = &self.page.bytes[start_of_value..start_of_value + len_of_value];
                let value = V::from_bytes(bytes);
                Some(value)
            },
            Err(_insertion_point) => None,
        }
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
        let offset_slot = end_of_free_space - bytes.len();

        offset_slot <= offset_pointer
    }

    fn keys<'a>(&self) -> Vec<K> where K: SlotBytes {
        let offset = self.start_of_free_space();
        let header_len = 8 as usize;
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

    fn start_of_free_space(&self) -> usize {
        let header_len = 8 as usize;
        let number_of_pointer = self.number_of_pointer();
        header_len + 2 * number_of_pointer as usize
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
    use crate::error::Error;
    use crate::node::Node;
    use crate::page::Page;
    use crate::slot::Slot;

    #[test]
    fn test_pointers_sorted() {
        let page = Page::new(Default::default());
        let mut node = Node::<u16, String>::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        let _ = node.insert(&Slot::new(7u16, "ありがと".to_string()));
        let _ = node.insert(&Slot::new(5u16, "defg".to_string()));
        let _ = node.insert(&Slot::new(1u16, "ぽぽ".to_string()));
        let pointers = node.keys();
        println!("{:?}", &node.page);
        assert_eq!(pointers, [1, 2, 5, 7]);
    }

    #[test]
    fn test_pointers_full() {
        let page = Page::new(Default::default());
        let mut node = Node::<u16, String>::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        let _ = node.insert(&Slot::new(7u16, "ありがと".to_string()));
        let _ = node.insert(&Slot::new(5u16, "defg".to_string()));
        let _ = node.insert(&Slot::new(1u16, "ぽぽ".to_string()));
        let res = node.insert(&Slot::new(100u16, "あふれちゃう".to_string()));
        let pointers = node.keys();
        // println!("{:?}", &node.page);
        assert_eq!(pointers, [1, 2, 5, 7]);
        assert_eq!(res, Err(Error::FullLeaf));
    }

    #[test]
    fn test_search_hit() {
        let page = Page::new(Default::default());
        let mut node = Node::<u16, String>::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        assert_eq!(node.search(&2u16), Some("abc".to_string()));
    }

    #[test]
    fn test_search_notfound() {
        let page = Page::new(Default::default());
        let mut node = Node::<u16, String>::create(page);
        let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
        assert_eq!(node.search(&5u16), None);
    }
}