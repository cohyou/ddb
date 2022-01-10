use std::convert::TryInto;

use crate::node::NodeType;


pub struct Slot<K, V> where
    K: SlotBytes + Clone,
    V: SlotBytes + Clone,
{
    pub key: K, value: V
}

impl<K, V> Slot<K, V> where
    K: SlotBytes + Clone,
    V: SlotBytes + Clone,
{
    pub fn new(key: K, value: V) -> Self {
        Slot { key: key, value: value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut key_bytes = self.key.into_bytes().to_vec();
        let value_bytes = self.value.into_bytes().to_vec();
        key_bytes.extend(value_bytes);
        key_bytes
    }
}

#[allow(dead_code)]
pub enum SlotValue<V> {
    Leaf(V),
    PageId(u16),
}

impl<V: Clone> SlotValue<V> {
    pub fn leaf_value(&self) -> V {
        match self {
            Self::Leaf(v) => v.clone(),
            Self::PageId(_) => panic!(),
        }
    }

    pub fn page_id(&self) -> u16 {
        match self {
            Self::Leaf(_) => panic!(),
            Self::PageId(page_id) => page_id.clone(),
        }
    }
}


pub trait SlotBytes {
    fn into_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8], node_type: NodeType) -> Self;
}

impl SlotBytes for u8 {
    fn into_bytes(&self) -> Vec<u8> {
        vec![1, 0, self.clone()]
    }

    fn from_bytes(bytes: &[u8], _node_type: NodeType) -> Self {
        bytes[0]
    }
}

impl SlotBytes for u16 {
    fn into_bytes(&self) -> Vec<u8> {
        let mut res = vec![2, 0];
        res.extend(self.to_le_bytes().to_vec());
        res
    }

    fn from_bytes(bytes: &[u8], _node_type: NodeType) -> Self {
        u16::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl SlotBytes for u32 {
    fn into_bytes(&self) -> Vec<u8> {
        let mut res = vec![4, 0];
        res.extend(self.to_le_bytes().to_vec());
        res
    }

    fn from_bytes(_bytes: &[u8], _node_type: NodeType) -> Self {
        unimplemented!()
    }
}

impl SlotBytes for String {
    fn into_bytes(&self) -> Vec<u8> {
        let bytes = self.bytes().collect::<Vec<_>>();
        let len = bytes.len() as u16;
        let mut res = len.to_le_bytes().to_vec();
        res.extend(bytes);
        res
    }

    fn from_bytes<'a>(bytes: &'a [u8], _node_type: NodeType) -> Self {
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

impl<V: SlotBytes> SlotBytes for SlotValue<V> {
    fn into_bytes(&self) -> Vec<u8> {
        match self {
            SlotValue::Leaf(v) => v.into_bytes(), 
            SlotValue::PageId(_) => panic!("SlotValue into_bytes: cant PageId"),
        }
    }

    fn from_bytes<'a>(bytes: &'a [u8], node_type: NodeType) -> Self {
        match node_type {
            NodeType::Leaf => SlotValue::Leaf(V::from_bytes(bytes, node_type)), 
            NodeType::Branch => SlotValue::PageId(u16::from_bytes(bytes, node_type)),
        }
    }
}
