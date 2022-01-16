use std::convert::TryInto;
use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;
use std::mem::size_of;
use std::ops::Range;

pub trait Pointer {
    fn new(offset: u16, key_size: u16, value_size: u16) -> Self;
    fn len() -> usize;
    fn from_bytes(bytes: &[u8]) -> Self;
    fn slot_offset(&self) -> u16;
    fn key_size(&self) -> u16;
    fn value_size(&self) -> u16;
    fn key_range(&self) -> Range<usize> {
        let start = self.slot_offset() as usize;
        let end = (self.slot_offset() + self.key_size()) as usize;
        start..end
    }

    fn value_range(&self) -> Range<usize> {
        let start = (self.slot_offset() + self.key_size()) as usize;
        let end = (self.slot_offset() + self.key_size() + self.value_size()) as usize;
        start..end
    }

    fn slot_size(&self) -> u16 {
        self.key_size() + self.value_size()
    }
    fn to_bytes(&self) -> Vec<u8>;
}

pub struct LeafPointer {
    slot_offset: u16,
    key_size: u16,
    value_size: u16,
}

impl Pointer for LeafPointer {
    fn new(offset: u16, key_size: u16, value_size: u16) -> Self {
        LeafPointer { slot_offset: offset, key_size: key_size, value_size: value_size }
    }

    fn len() -> usize { size_of::<LeafPointer>() }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes_offset = &bytes[0..2];
        let offset = u16::from_le_bytes(bytes_offset.try_into().unwrap());
        let bytes_key_size = &bytes[2..4];
        let key_size = u16::from_le_bytes(bytes_key_size.try_into().unwrap());
        let bytes_value_size = &bytes[4..6];
        let value_size = u16::from_le_bytes(bytes_value_size.try_into().unwrap());
        LeafPointer { slot_offset: offset, key_size: key_size, value_size: value_size }
    }
    fn slot_offset(&self) -> u16 { self.slot_offset }
    fn key_size(&self) -> u16 { self.key_size } 
    fn value_size(&self) -> u16 { self.value_size }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.slot_offset.to_le_bytes().to_vec();
        bytes.append(&mut self.key_size().to_le_bytes().to_vec());
        bytes.append(&mut self.value_size().to_le_bytes().to_vec());
        bytes
    }
}

impl Debug for LeafPointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_tuple("")
            .field(&self.slot_offset)
            .field(&self.key_size)
            .field(&self.value_size)
            .finish()
    }
}

pub struct BranchPointer {
    slot_offset: u16,
    key_size: u16,
}

impl Pointer for BranchPointer {
    fn new(offset: u16, key_size: u16, _value_size: u16) -> Self {
        BranchPointer { slot_offset: offset, key_size: key_size }
    }

    fn len() -> usize { size_of::<BranchPointer>() }

    fn from_bytes(bytes: &[u8]) -> Self {
        let bytes_offset = &bytes[0..2];
        let offset = u16::from_le_bytes(bytes_offset.try_into().unwrap());
        let bytes_key_size = &bytes[2..4];
        let key_size = u16::from_le_bytes(bytes_key_size.try_into().unwrap());
        BranchPointer { slot_offset: offset, key_size: key_size }
    }
    fn slot_offset(&self) -> u16 { self.slot_offset }
    fn key_size(&self) -> u16 { self.key_size }
    fn value_size(&self) -> u16 { 2 }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.slot_offset.to_le_bytes().to_vec();
        bytes.append(&mut self.key_size().to_le_bytes().to_vec());
        bytes
    }
}

impl Debug for BranchPointer {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_tuple("")
            .field(&self.slot_offset)
            .field(&self.key_size)
            .finish()
    }
}
