use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

use crate::page::Page;

pub struct Meta { pub page: Page }

impl Meta {
    pub fn new(page: Page) -> Self {
        Meta { page: page }
    }

    pub fn root_page_id(&self) -> u16 {
        self.page.u16_bytes(0)
    }

    pub fn set_root_page_id(&mut self, root_page_id: u16) {
        self.page.set_u16_bytes(0, root_page_id);
    }
}

impl Debug for Meta {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "root_page_id={:?} ", self.root_page_id())
    }
}
