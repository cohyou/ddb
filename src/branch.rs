use crate::page::Page;
use crate::error::Error;

#[repr(C)]
#[derive(Debug)]
pub struct Branch {
    page: Page
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Header {
    number_of_pointer: u16,
    end_of_free_space: u16,
    max_pointer: u16,
    _padding3: u16,
}

#[repr(C)]
#[derive(Debug)]
struct Pointer(pub u16);

impl Branch {
    pub fn new() -> Self {
        let page = Page::new();
        Branch { page: page }
    }

    pub fn search(&self, _searching_key: u16) -> Result<u16, Error> {
        let next_pointer = self.max_pointer();
        Ok(next_pointer)
    }

    pub fn set_max_pointer(&mut self, number: u16) {
        self.page.set_u16_bytes(4, number);
    }

    pub fn add(&mut self) {
        self.add_pointer();
    }

    pub fn max_pointer(&self) -> u16 {
        let header = self.header();
        header.max_pointer
    }

    fn add_pointer(&mut self) {
        let _end_of_free_space = self.end_of_free_space();
        let offset = 8;
        self.page.set_u16_bytes(offset, 23);
    }

    fn end_of_free_space(&self) -> u16 {
        let header = self.header();
        header.end_of_free_space
    }

    fn header(&self) -> Header {
        let ptr_bytes = &self.page.bytes[0..4];
        let p = ptr_bytes.as_ptr() as *const Header;
        let header: Header;
        unsafe { header = *p; }
        header
    }
}
