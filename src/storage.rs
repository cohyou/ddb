use std::fs::File;
use std::fs::OpenOptions;

use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;

use std::path::Path;


const PAGE_SIZE: usize = 64;

pub struct Storage { next_page_id: u16, file: File }

impl Storage {
    pub fn from_path(file_path: impl AsRef<Path>) -> Self {
        let file = Self::open_file(file_path);
        let file_size = file.metadata().unwrap().len();
        let next_page_id = file_size / PAGE_SIZE as u64;
        Storage { next_page_id: next_page_id as u16, file: file }
    }
    
    fn open_file(file_path: impl AsRef<Path>) -> File { 
        OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path).unwrap()
    }

    fn allocate_page(&mut self) -> u16 {
        let id = self.next_page_id;
        self.next_page_id += 1;
        id
    }
}

pub struct Page { id: u16, bytes: [u8; PAGE_SIZE] }

impl Page {
    pub fn new(id: u16) -> Self {
        Page { id: id, bytes: [0; PAGE_SIZE] }
    }

    fn read(&mut self, mut file: File) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * self.id as u64;
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut self.bytes)
    }

    fn write(&mut self, mut file: File) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * self.id as u64;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&self.bytes)
    }
}
