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

    fn allocate_page(&mut self) -> Page {
        let id = self.next_page_id;
        self.next_page_id += 1;
        Page::new(id)
    }
}

#[cfg(test)]
mod test {
    use std::fs::OpenOptions;
    use std::fs::remove_file;
    use std::io::Write;

    use crate::storage::Storage;
    use crate::storage::PAGE_SIZE;

    #[test]
    fn test_from_path_zero() {
        let temp_file_path = "tmp0";
        let storage = Storage::from_path(temp_file_path);
        assert_eq!(storage.next_page_id, 0);
        let _ = remove_file(temp_file_path);
    }

    #[test]
    fn test_from_path_single_page() {
        let temp_file_path = "tmp1";
        let mut f = OpenOptions::new()
            .write(true).truncate(true).create(true)
            .open(temp_file_path).unwrap();
        let bytes = [0; PAGE_SIZE];
        let _ = f.write_all(&bytes);
        let storage = Storage::from_path(temp_file_path);
        assert_eq!(storage.next_page_id, 1);
        let _ = remove_file(temp_file_path);
    }

    #[test]
    fn test_from_path_multi_page() {
        let page_count = 475u16;
        let bytes_count = PAGE_SIZE * page_count as usize;
        let temp_file_path = "tmp_n";
        let mut f = OpenOptions::new()
            .write(true).truncate(true).create(true)
            .open(temp_file_path).unwrap();
        let mut bytes = Vec::with_capacity(bytes_count);
        // bytes.fill(Default::default());
        bytes.extend(std::iter::repeat(0).take(bytes_count));
        let _ = f.write_all(&bytes);
        let storage = Storage::from_path(temp_file_path);
        assert_eq!(storage.next_page_id, page_count);
        let _ = remove_file(temp_file_path);
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
