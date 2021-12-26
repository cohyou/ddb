use std::io::Write;
use std::fs::File;
use std::io::Read;
use std::fs::OpenOptions;

pub const LEN_PAGE: u16 = 64;

#[repr(C)]
#[derive(Debug)]
pub struct Page {
    pub bytes: [u8; LEN_PAGE as usize]
}

impl Page {
    pub fn new() -> Self {
        let bytes = [0; LEN_PAGE as usize];
        Page { bytes: bytes }
    }

    pub fn save(&self) -> std::io::Result<()> {
        let mut f = OpenOptions::new().truncate(true).open("f")?;
        f.write_all(self.bytes.as_ref())
    }

    pub fn load(&mut self) -> std::io::Result<()> {
        let mut f = File::open("f")?;
        f.read(self.bytes.as_mut())?;
        Ok(())
    }

    pub fn set_u16_bytes(&mut self, offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        for (i, byte) in bytes.iter().enumerate() {
            self.bytes[offset + i] = byte.clone()
        }
    }

    pub fn set_string_bytes(&mut self, offset: usize, value: String) {
        let bytes = value.bytes();
        for (i, byte) in bytes.enumerate() {
            self.bytes[offset + i] = byte
        }
    }
}

#[test]
fn test() {
    let s = std::mem::size_of::<u16>();
    assert_eq!(s, 1);
}