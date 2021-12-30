use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::path::Path;


const PAGE_SIZE: usize = 64;



struct Node { page: Page }
// struct RootNode(Node);
// struct BranchNode(Node);
// struct LeafNode(Node);
pub struct BTree { root: Node }



impl BTree {
    pub fn new(file_path: impl AsRef<Path>) -> Self {
        let root = Node::new(0);
        BTree { root: root }
    }

    fn open_file(file_path: impl AsRef<Path>) -> File { 
        OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path).unwrap()
    }
}

impl Node {
    fn new(page_id: u16) -> Self {
        Node { page: Page::new(page_id) }
    }
}

struct Slot<K, V> { key: K, value: V }

impl<K, V> Slot<K, V> {
    fn new(key: K, value: V) -> Self {
        Slot { key: key, value: value }
    }
}

struct Storage { next_page_id: u16, file: File }

impl Storage {
    fn from_path(file_path: impl AsRef<Path>) -> Self {
        let file = BTree::open_file(file_path);
        let file_size = file.metadata().unwrap().len();
        let next_page_id = file_size / PAGE_SIZE as u64;
        Storage { next_page_id: next_page_id as u16, file: file }
    }
    
    fn allocate_page(&mut self) -> u16 {
        let id = self.next_page_id;
        self.next_page_id += 1;
        id
    }
}

struct Page { id: u16, bytes: [u8; PAGE_SIZE] }

impl Page {
    fn new(id: u16) -> Self {
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
