struct Page([u16; PAGE_SIZE]);
struct Node { page_id: u16, page: Page }
struct RootNode(Node);
struct BranchNode(Node);
struct LeafNode(Node);
struct BTree { max_page_id: u16, root: Node }

impl BTree {
    pub fn new() -> Self {
        let root = Node::new(0);
        BTree { max_page_id: 0, root: root}
    }
}

impl Node {
    fn new(page_id: u16) -> Self {
        Node { page_id: page_id, page: Page::new() }
    }
}

impl Page {
    fn new() -> Self {
        Page([0; PAGE_SIZE])
    }

    fn write(page_id: u16) {
        let file_name = "f";
        let mut f = OpenOptions::new().write(true).create(true).open(file_name)?;
        f.write_all(self.0.as_ref())
    }
}

const PAGE_SIZE: usize = 64;

// pub fn new(heap_file: File) -> io::Result<Self> {
//     let heap_file_size = heap_file.metadata()?.len();
//     let next_page_id = heap_file_size / PAGE_SIZE as u64;
//     Ok(Self {
//         heap_file,
//         next_page_id,
//     })
// }

// pub fn open(heap_file_path: impl AsRef<Path>) -> io::Result<Self> {
//     let heap_file = OpenOptions::new()
//         .read(true)
//         .write(true)
//         .create(true)
//         .open(heap_file_path)?;
//     Self::new(heap_file)
// }

// pub fn read_page_data(&mut self, page_id: PageId, data: &mut [u8]) -> io::Result<()> {
//     let offset = PAGE_SIZE as u64 * page_id.to_u64();
//     self.heap_file.seek(SeekFrom::Start(offset))?;
//     self.heap_file.read_exact(data)
// }

// pub fn write_page_data(&mut self, page_id: PageId, data: &[u8]) -> io::Result<()> {
//     let offset = PAGE_SIZE as u64 * page_id.to_u64();
//     self.heap_file.seek(SeekFrom::Start(offset))?;
//     self.heap_file.write_all(data)
// }

// pub fn allocate_page(&mut self) -> PageId {
//     let page_id = self.next_page_id;
//     self.next_page_id += 1;
//     PageId(page_id)
// }