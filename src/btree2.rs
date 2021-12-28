use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;


const PAGE_SIZE: usize = 64;


struct Page([u8; PAGE_SIZE]);
struct Node { page_id: u16, page: Page }
struct RootNode(Node);
struct BranchNode(Node);
struct LeafNode(Node);
struct BTree { next_page_id: u16, root: Node, file: File }


impl BTree {
    pub fn new() -> Self {
        let root = Node::new(0);
        let file = BTree::open_file();
        let file_size = file.metadata().unwrap().len();
        let next_page_id = file_size / PAGE_SIZE as u64;
        BTree { next_page_id: next_page_id as u16, root: root, file: file }
    }

    fn open_file() -> File {
        let file_path = "f";
        OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path).unwrap()
    }
}

impl Node {
    fn new(page_id: u16) -> Self {
        Node { page_id: page_id, page: Page::new() }
    }

    fn read(&mut self, mut file: File) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * self.page_id as u64;
        file.seek(SeekFrom::Start(offset))?;
        file.read_exact(&mut self.page.0)
    }

    fn write(&mut self, mut file: File) -> io::Result<()> {
        let offset = PAGE_SIZE as u64 * self.page_id as u64;
        file.seek(SeekFrom::Start(offset))?;
        file.write_all(&self.page.0)
    }
}

impl Page {
    fn new() -> Self {
        Page([0; PAGE_SIZE])
    }
}

// pub fn allocate_page(&mut self) -> PageId {
//     let page_id = self.next_page_id;
//     self.next_page_id += 1;
//     PageId(page_id)
// }