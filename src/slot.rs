use std::convert::TryInto;


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

pub trait SlotBytes {
    fn into_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

impl SlotBytes for u8 {
    fn into_bytes(&self) -> Vec<u8> {
        vec![1, self.clone()]
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        bytes[0]
    }
}

impl SlotBytes for u16 {
    fn into_bytes(&self) -> Vec<u8> {
        let mut res = vec![2];
        res.extend(self.to_le_bytes().to_vec());
        res
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        u16::from_le_bytes(bytes.try_into().unwrap())
    }
}

impl SlotBytes for u32 {
    fn into_bytes(&self) -> Vec<u8> {
        let mut res = vec![4];
        res.extend(self.to_le_bytes().to_vec());
        res
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
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

    fn from_bytes<'a>(bytes: &'a [u8]) -> Self {
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}
