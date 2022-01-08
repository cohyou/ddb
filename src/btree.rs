use std::cell::RefCell;
use std::path::Path;
use crate::storage::Storage;
use crate::storage::Page;
use crate::error::Error;

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

    pub fn search<T>(&self, value: T) -> Result<T, Error> {
        if let Some(root_page_id) = self.root_page_id {
            Err(Error::NoPage)
        } else {
            Err(Error::NoPage)
        }
    }

    pub fn insert<K, V>(&self, key: K, value: V) {

    }
}

#[cfg(test)]
mod test {
    use std::fs::remove_file;

    use crate::btree::BTree;
    use crate::error::Error;

    #[test]
    fn test_search_empty() {
        let p = "f0";
        let btree = BTree::create(p);
        let error: Result<&str, Error> = Err(Error::NoPage);
        assert_eq!(btree.search(""), error);
        let _ = remove_file(p);
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

