use crate::page::{
    Page,
    LEN_PAGE,
};
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
        let mut page = Page::new();

        page.set_u16_bytes(2, LEN_PAGE);

        Branch { page: page }
    }

    pub fn search(&self, _searching_key: u16) -> Result<u16, Error> {
        let next_pointer = self.max_pointer();
        Ok(next_pointer)
    }

    pub fn set_max_pointer(&mut self, number: u16) {
        self.page.set_u16_bytes(4, number);
    }

    pub fn add(&mut self, key: u16) {
        let page_id = 1;
        self.add_page_id(page_id);
        self.add_key(key);
        self.add_pointer();
        self.increment_number_of_pointer();
    }

    fn add_key(&mut self, key: u16) {
        let len = 2;
        let end_of_free_space = self.end_of_free_space() as usize;
        
        self.page.set_u16_bytes(end_of_free_space - len, key);
        self.set_end_of_free_space((end_of_free_space - len) as u16);
    }

    pub fn max_pointer(&self) -> u16 {
        let header = self.header();
        header.max_pointer
    }

    fn add_page_id(&mut self, key: u16) {
        let len = 2;
        let end_of_free_space = self.end_of_free_space() as usize;
        
        self.page.set_u16_bytes(end_of_free_space - len, key);
        self.set_end_of_free_space((end_of_free_space - len) as u16);
    }

    fn add_pointer(&mut self) {
        let _end_of_free_space = self.end_of_free_space();
        let offset = 8;
        self.page.set_u16_bytes(offset, 23);
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

    fn header(&self) -> Header {
        let ptr_bytes = &self.page.bytes[0..4];
        let p = ptr_bytes.as_ptr() as *const Header;
        let header: Header;
        unsafe { header = *p; }
        header
    }
}
