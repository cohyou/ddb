const LEN_PAGE: u16 = 64;

#[repr(C)]
#[derive(Debug)]
struct Page {
    bytes: [u8; LEN_PAGE as usize]
}

use std::io::Write;
use std::fs::File;
use std::io::Read;
use std::fs::OpenOptions;

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
}

#[repr(C)]
#[derive(Debug)]
struct Pointer(pub u16);
use std::slice;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Header {
    number_of_pointer: u16,
    end_of_free_space: u16,
    _padding2: u16,
    _padding3: u16,
}

#[repr(C)]
#[derive(Debug)]
struct Leaf {
    page: Page
}

#[derive(Debug)]
enum Error {
    NotFound,
}

use std::convert::TryInto;
impl Leaf {
    pub fn new() -> Self {
        let mut page = Page::new();
        let len_bytes = LEN_PAGE.to_le_bytes();
        page.bytes[2] = len_bytes[0];
        page.bytes[3] = len_bytes[1];
        Leaf { page: page }
    }

    pub fn add(&mut self, key: u16, value: String) {
        if self.can_add(value.len() as u16) {
            self.set_value(value);
            self.set_key(key);
            self.add_pointer();
            self.increment_number_of_pointer();    
        } else {
            println!("can not add");
        }
    }

    pub fn search(&self, searching_key: u16) -> Result<String, Error> {
        let pointers = self.pointers();
        for pointer in pointers {
            // println!("{:?}", pointer);
            let offset = pointer.0 as usize;
            let key_bytes = &self.page.bytes[offset..offset+2];
            // println!("{:?}", key_bytes);
            let key = u16::from_le_bytes(key_bytes.try_into().expect("search"));
            if key == searching_key {
                let value_offset = offset + 4;
                let value_size_bytes = &self.page.bytes[offset+2..offset+4];
                let value_size = u16::from_le_bytes(value_size_bytes.try_into().expect(""));
                let value_bytes = self.page.bytes[value_offset..value_offset + value_size as usize].to_vec();
                let value = unsafe { String::from_utf8_unchecked(value_bytes) };
                return Ok(value);
            }
        }
        Err(Error::NotFound)
    }

    fn can_add(&self, value_length: u16) -> bool {
        let rest_of_space = self.rest_of_space();
        println!("2 + 2 + 2 + value_length: {:?} rest_of_space: {:?}", 2 + 2 + 2 + value_length, rest_of_space);
        // pointer 2
        // key 2
        // value_length 2
        2 + 2 + 2 + value_length <= rest_of_space
    }

    fn rest_of_space(&self) -> u16 {
        let end = self.end_of_free_space();
        let free_space_count = 4 + 2 * self.number_of_pointer();
        let rest_of_space = end - free_space_count;
        println!("end: {:?} free_space_count: {:?}", end, free_space_count);
        rest_of_space
    }

    fn set_key(&mut self, key: u16) {
        let len = 2;
        let end_of_free_space = self.end_of_free_space() as usize;
        let bytes = key.to_le_bytes();
        let mut n = 0;
        for byte in bytes.iter() {
            self.page.bytes[end_of_free_space - len + n] = byte.clone();
            n += 1;
        }
        self.set_end_of_free_space((end_of_free_space - n) as u16);
    }

    fn set_value(&mut self, value: String) {
        let len = value.len();
        let end_of_free_space = self.end_of_free_space() as usize;
        let bytes = value.bytes();
        let mut value_size: u16 = 0;
        let starting_offset = end_of_free_space - len;
        for byte in bytes {
            self.page.bytes[starting_offset + (value_size as usize)] = byte;
            value_size += 1;
        }
        let value_size_bytes = value_size.to_le_bytes();
        self.page.bytes[starting_offset - 2] = value_size_bytes[0];
        self.page.bytes[starting_offset - 1] = value_size_bytes[1];

        self.set_end_of_free_space((starting_offset - 2) as u16);
    }

    fn add_pointer(&mut self) {
        let number_of_pointer = self.number_of_pointer();
        let length_of_header = 4;
        let offset = length_of_header + 2 * number_of_pointer;
        let end_of_free_space = self.end_of_free_space();
        let bytes = end_of_free_space.to_le_bytes();
        self.page.bytes[offset as usize] = bytes[0];
        self.page.bytes[(offset+1) as usize] = bytes[1];
    }

    fn increment_number_of_pointer(&mut self) {
        let mut number_of_pointer = self.number_of_pointer();
        number_of_pointer += 1;
        self.set_number_of_pointer(number_of_pointer);
    }

    fn number_of_pointer(&self) -> u16 {
        let header = self.header();
        header.number_of_pointer
    }

    fn end_of_free_space(&self) -> u16 {
        let header = self.header();
        header.end_of_free_space
    }

    fn set_number_of_pointer(&mut self, number: u16) {
        let bytes = number.to_le_bytes();
        self.page.bytes[0] = bytes[0];
        self.page.bytes[1] = bytes[1];
    }

    fn set_end_of_free_space(&mut self, number: u16) {
        let bytes = number.to_le_bytes();
        self.page.bytes[2] = bytes[0];
        self.page.bytes[3] = bytes[1];
    }

    fn pointers<'a>(&self) -> &'a [Pointer] {
        let ptr_bytes = &self.page.bytes[4..8];
        let p = ptr_bytes.as_ptr() as *const Pointer;
        let number_of_pointer = self.number_of_pointer();
        unsafe {
            slice::from_raw_parts(p, number_of_pointer as usize)
        }   
    }

    fn header(&self) -> Header {
        let ptr_bytes = &self.page.bytes[0..4];
        let p = ptr_bytes.as_ptr() as *const Header;
        let header: Header;
        unsafe {
            header = *p;
        }
        header
    }
}

fn main() {
    let mut leaf = Leaf::new();
    let _ = leaf.page.load();

    leaf.add(13, "abc".to_string());
    leaf.add(2000, "defg".to_string());
    leaf.add(200, "こんにちは".to_string());
    leaf.add(8976, "ありがとう".to_string());

    let res = leaf.search(13);
    println!("search 13: {:?}", res);
    let res = leaf.search(2000);
    println!("search 2000: {:?}", res);
    let res = leaf.search(200);
    println!("search 200: {:?}", res);
    let res = leaf.search(8976);
    println!("search 8976: {:?}", res);

    println!("leaf: {:?}", leaf);
    // println!("rest_of_space: {:?}", leaf.rest_of_space());

    let _ = leaf.page.save();
}
