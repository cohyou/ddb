use std::marker::PhantomData;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::Path;

use crate::page::PAGE_SIZE;
use crate::page::Page;


pub struct Storage<K, V> {
    pub next_page_id: u16, 
    file: File,
    _phantom_key: PhantomData<fn() -> K>,
    _phantom_value: PhantomData<fn() -> V>,
}

impl<K, V> Storage<K, V> {
    pub fn from_path(file_path: impl AsRef<Path>) -> Self {
        let file = Self::open_file(file_path);
        let file_size = file.metadata().unwrap().len();
        let next_page_id = file_size / PAGE_SIZE as u64;
        Storage::new(next_page_id as u16, file)
    }
    
    pub fn allocate_page(&mut self) -> Page {
        let id = self.next_page_id;
        self.next_page_id += 1;
        Page::new(id)
    }

    pub fn write_page(&mut self, page: &mut Page) {
        let _ = page.write(&mut self.file);
    }

    pub fn read_page(&mut self, page: &mut Page) {
        let _ = page.read(&mut self.file);
    }

    fn new(next_page_id: u16, file: File) -> Self {
        Storage::<K, V> {
            next_page_id: next_page_id, 
            file: file,
            _phantom_key: PhantomData,
            _phantom_value: PhantomData,
        }
    }

    fn open_file(file_path: impl AsRef<Path>) -> File { 
        OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path).unwrap()
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
        let temp_file_path = "test_from_path_zero";
        let storage = Storage::<u16, &str>::from_path(temp_file_path);
        let _ = remove_file(temp_file_path);
        assert_eq!(storage.next_page_id, 0);
    }

    #[test]
    fn test_from_path_single_page() {
        let temp_file_path = "test_from_path_single_page";
        let mut f = OpenOptions::new()
            .write(true).truncate(true).create(true)
            .open(temp_file_path).unwrap();
        let bytes = [0; PAGE_SIZE];
        let _ = f.write_all(&bytes);
        let storage = Storage::<u16, &str>::from_path(temp_file_path);
        let _ = remove_file(temp_file_path);
        assert_eq!(storage.next_page_id, 1);
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
        bytes.extend(std::iter::repeat(0).take(bytes_count));
        let _ = f.write_all(&bytes);
        let storage = Storage::<u16, &str>::from_path(temp_file_path);
        assert_eq!(storage.next_page_id, page_count);
        let _ = remove_file(temp_file_path);
    }
}

