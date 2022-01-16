use std::convert::TryInto;

#[derive(Debug)]
pub struct Slot<K, V> where
    K: SlotBytes + Clone,
    V: SlotBytes + Clone,
{
    pub key: K, pub value: V
}

impl<K, V> Slot<K, V> where
    K: SlotBytes + Clone,
    V: SlotBytes + Clone,
{
    pub fn new(key: K, value: V) -> Self {
        Slot { key: key, value: value }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut key_bytes = self.key.into_bytes();
        let value_bytes = self.value.into_bytes();
        key_bytes.extend(value_bytes);
        key_bytes
    }

    pub fn key_size(&self) -> u16 {
        self.key.into_bytes().len() as u16
    }

    pub fn value_size(&self) -> u16 {
        self.value.into_bytes().len() as u16
    }
}


pub trait SlotBytes {
    fn into_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl SlotBytes for u8 {
    fn into_bytes(&self) -> Vec<u8> {
        vec![self.clone()]
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }
}

impl SlotBytes for u16 {
    fn into_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        if let Ok(bytes) = bytes.try_into() {
            u16::from_le_bytes(bytes)
        } else {
            panic!("SlotBytes for u16 from_bytes bytes: {:?}", bytes);
        }
    }
}

impl SlotBytes for u32 {
    fn into_bytes(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        unimplemented!()
    }
}

impl SlotBytes for String {
    fn into_bytes(&self) -> Vec<u8> {
        self.bytes().collect::<Vec<_>>()
    }

    fn from_bytes<'a>(bytes: &'a [u8]) -> Self {
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}

