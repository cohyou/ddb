use std::cell::RefCell;
use std::path::Path;
use crate::storage::Storage;
use crate::storage::Page;


struct Node { page: Page }
// struct RootNode(Node);
// struct BranchNode(Node);
// struct LeafNode(Node);
pub struct BTree {
    root_page_id: Option<u16>,
    storage: RefCell<Storage>,
}

impl BTree {
    pub fn create(file_path: impl AsRef<Path>) -> Self {
        let storage = Storage::from_path(file_path);
        BTree { root_page_id: Default::default(), storage: RefCell::new(storage) }
    }

    pub fn insert<K, V>(&self, key: K, value: V) {

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

#[test]
fn test() {
    let btree = BTree::create("");
    btree.insert(13, "abc".to_string());
}
