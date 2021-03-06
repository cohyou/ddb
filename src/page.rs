use std::convert::TryInto;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::fs::File;


pub const PAGE_SIZE: usize = 64;


#[derive(Debug, Clone)]
pub struct Page { pub id: u16, pub bytes: [u8; PAGE_SIZE] }

impl Page {
    pub fn new(id: u16) -> Self {
        Page { id: id, bytes: [0; PAGE_SIZE] }
    }

    pub fn i16_bytes(&self, offset: usize) -> i16 {
        let bytes = &self.bytes[offset..offset + 2];
        i16::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn set_u16_bytes(&mut self, offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            self.bytes[offset + i] = byte.clone()
        }
    }

    pub fn u16_bytes(&self, offset: usize) -> u16 {
        let bytes = &self.bytes[offset..offset + 2];
        u16::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn set_bytes<'a>(&mut self, offset: usize, bytes: Vec<u8>) {
        for (i, byte) in bytes.into_iter().enumerate() {
            self.bytes[offset + i] = byte.clone();
        }
    }

    pub fn read(&mut self, file: &mut File) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * self.id as u64;
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut self.bytes)
    }

    pub fn write(&mut self, file: &mut File) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * self.id as u64;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&self.bytes)
    }
}
