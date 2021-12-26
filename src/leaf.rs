use std::slice;
use std::convert::TryInto;
use crate::{
    error::Error, 
    page::{
        Page,
        LEN_PAGE,
    }
};

#[repr(C)]
#[derive(Debug)]
pub struct Leaf {
    pub page: Page
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Header {
    number_of_pointer: u16,
    end_of_free_space: u16,
    _padding2: u16,
    _padding3: u16,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
struct Pointer(pub u16);


impl Leaf {
    pub fn new() -> Self {
        let mut page = Page::new();

        page.set_u16_bytes(2, LEN_PAGE);

        Leaf { page: page }
    }

    pub fn add(&mut self, key: u16, value: String) -> Result<(), Error> {
        if !self.can_add(value.len() as u16) { return Err(Error::FullLeaf) }
        self.add_value(value);
        self.add_key(key);
        self.add_pointer(key);
        self.increment_number_of_pointer();
        Ok(())
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
                let value = self.resolve_value(offset);
                return Ok(value);
            }
        }
        Err(Error::NotFound)
    }

    pub fn can_add(&self, value_length: u16) -> bool {
        let rest_of_space = self.rest_of_space();
        // println!("2 + 2 + 2 + value_length: {:?} rest_of_space: {:?}", 2 + 2 + 2 + value_length, rest_of_space);
        // pointer 2
        // key 2
        // value_length 2
        2 + 2 + 2 + value_length <= rest_of_space
    }

    pub fn list(&self) -> Vec<u16> {
        // self.pointers().map(|pointer| pointer)
        let mut keys = vec![];
        for pointer in self.pointers() {
            let Pointer(p) = pointer;
            keys.push(self.resolve_key(p.clone() as usize));
        }
        keys
    }

    fn resolve_key(&self, offset: usize) -> u16 {
        let key_bytes = &self.page.bytes[offset..offset+2];
        u16::from_le_bytes(key_bytes.try_into().expect("resolve_key"))
    }

    fn resolve_value(&self, offset: usize) -> String {
        let value_offset = offset + 4;
        let value_size_bytes = &self.page.bytes[offset+2..offset+4];
        let value_size = u16::from_le_bytes(value_size_bytes.try_into().expect("resolve_value"));
        let value_bytes = self.page.bytes[value_offset..value_offset + value_size as usize].to_vec();
        unsafe { String::from_utf8_unchecked(value_bytes) }
    }

    fn rest_of_space(&self) -> u16 {
        let end = self.end_of_free_space();
        let free_space_count = 4 + 2 * self.number_of_pointer();
        let rest_of_space = end - free_space_count;
        
        rest_of_space
    }

    fn add_key(&mut self, key: u16) {
        let len = 2;
        let end_of_free_space = self.end_of_free_space() as usize;
        
        self.page.set_u16_bytes(end_of_free_space - len, key);
        self.set_end_of_free_space((end_of_free_space - len) as u16);
    }

    fn add_value(&mut self, value: String) {
        let end_of_free_space = self.end_of_free_space() as usize;
        let len = value.len();
        let offset = end_of_free_space - len;

        self.page.set_string_bytes(offset, value);
        self.page.set_u16_bytes(offset - 2, len as u16);
        self.set_end_of_free_space((offset - 2) as u16);
    }

    fn add_pointer(&mut self, key: u16) {
        let offset = self.find_offset_of_new_pointer(key);
        let end_of_free_space = self.end_of_free_space();
        
        let start_of_free_space = self.start_of_free_space() as usize;

        if offset < self.start_of_free_space() as usize {
            self.page.bytes.copy_within(offset..start_of_free_space, offset + 2);
        }

        self.page.set_u16_bytes(offset, end_of_free_space);
    }

    fn find_offset_of_new_pointer(&self, key: u16) -> usize {
        let length_of_header = 4;
        let mut i = 0;
        for c in self.pointers() {
            let resolved_key = self.resolve_key(c.0 as usize);
            if resolved_key > key {
                return length_of_header + 2 * i as usize;
            }
            i += 1;
        }

        self.start_of_free_space() as usize
    }

    fn start_of_free_space(&self) -> u16 {
        let length_of_header = 4;
        let number_of_pointer = self.number_of_pointer();
        length_of_header + 2 * number_of_pointer
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
        self.page.set_u16_bytes(0, number);
    }

    fn set_end_of_free_space(&mut self, number: u16) {
        self.page.set_u16_bytes(2, number);
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
        unsafe { header = *p; }
        header
    }
}

#[test]
fn test() {
    let mut leaf = Leaf::new();
    let _ = leaf.page.load();

    let _ = leaf.add(13, "abc".to_string());
    let _ = leaf.add(2000, "defg".to_string());
    let _ = leaf.add(200, "こんにちは".to_string());
    let _ = leaf.add(8976, "ありがとう".to_string());

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

#[test]
fn test2() {
    let mut leaf = Leaf::new();

    let _ = leaf.add(13, "abc".to_string());
    let _ = leaf.add(2000, "defg".to_string());
    let _ = leaf.add(200, "こんにちは".to_string());
    let _ = leaf.add(8976, "ありがと".to_string());
    println!("leaf: {:?}", leaf);
    assert_eq!(leaf.list(), vec![13, 200, 2000, 8976]);
}