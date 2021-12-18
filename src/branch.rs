use crate::page::Page;

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

    pub fn set_number_of_pointer(&mut self, number: u16) {
        let bytes = number.to_le_bytes();
        self.page.bytes[4] = bytes[0];
        self.page.bytes[5] = bytes[1];
    }
}
