use std::cell::RefCell;
use std::path::Path;
use crate::error::Error;
use crate::node::Leaf;
use crate::node::Node;
use crate::node::NodeType;
use crate::page::Page;
use crate::slot::Slot;
use crate::slot::SlotBytes;
use crate::storage::Storage;


pub struct BTree<K, V> {
    root_page_id: Option<u16>,
    storage: RefCell<Storage<K, V>>,
}

impl<K: Ord + SlotBytes, V> BTree<K, V> {
    pub fn create(file_path: impl AsRef<Path>) -> Self {
        let storage = Storage::from_path(file_path);
        BTree { root_page_id: Default::default(), storage: RefCell::new(storage) }
    }

    pub fn search(&self, key: &K) -> Result<V, Error> where 
        V: SlotBytes
    {
        if let Some(root_page_id) = self.root_page_id {
            let node = self.read_node(root_page_id);
            match node.node_type() {
                NodeType::Leaf => {
                    if let Some(v) = node.search(key) {
                        Ok(v)
                    } else {
                        Err(Error::NotFound)
                    }
                },
                NodeType::Branch => {
                    unimplemented!()
                },
            }
        } else {
            Err(Error::NoPage)
        }
    }

    pub fn insert(&mut self, key: K, value: V) where
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        if let Some(root_page_id) = self.root_page_id {
            let mut node = self.read_node(root_page_id);
            match node.node_type() {
                NodeType::Leaf => {
                    let slot = Slot::new(key, value);
                    match node.insert(&slot) {
                        Ok(_) => {
                            let mut leaf = Leaf::new(node);
                            self.write_leaf(&mut leaf);
                        },
                        Err(_) => {
                            self.split(slot);
                            unimplemented!();
                        },
                    }

                },
                NodeType::Branch => unimplemented!(),
            }
        } else {
            let mut leaf = self.create_leaf();
            let slot = Slot::new(key, value);
            let _ = leaf.node.insert(&slot);
            self.write_leaf(&mut leaf);
            self.root_page_id = Some(0);
        }
    }

    pub fn delete(&mut self, key: &K) where K: SlotBytes {
        if let Some(root_page_id) = self.root_page_id {
            let mut node = self.read_node(root_page_id);
            match node.node_type() {
                NodeType::Leaf => {
                    let _ = node.delete(key);
                },
                NodeType::Branch => unimplemented!(),
            }
        }
    }

    fn split(&self, _slot: Slot<K, V>) where
        K: SlotBytes + Clone,
        V: SlotBytes + Clone, 
    {

    }

    fn create_leaf(&self) -> Leaf<K, V> {
        let page = self.storage.borrow_mut().allocate_page();
        let mut node = Node::<K, V>::create(page);
        node.set_node_type(NodeType::Leaf);
        Leaf { node: node }
    }

    fn write_leaf(&self, leaf: &mut Leaf<K, V>) {
        self.storage.borrow_mut().write_page(&mut leaf.node.page);
    }

    fn read_node(&self, page_id: u16) -> Node<K, V> {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);
        Node::<K, V>::new(page)
    }

    // fn root_page_id(&self) -> u16 { self.root_page_id.unwrap() }
}

#[cfg(test)]
mod test {
    use std::fs::OpenOptions;
    use std::fs::remove_file;
    
    use std::io::Read;

    use std::path::Path;

    use crate::btree::BTree;
    use crate::error::Error;
    use crate::page::PAGE_SIZE;

    #[test]
    fn test_search_empty() {
        let p = "test_search_empty";
        let btree = BTree::<u16, String>::create(p);
        let error: Result<String, Error> = Err(Error::NoPage);
        let _ = remove_file(p);
        assert_eq!(btree.search(&0), error);
    }

    #[test]
    fn test_insert_first_slot() {
        let key = 123u8;
        let value = "abc".to_string();
        let p = "test_insert_first";
        let mut btree = BTree::create(p);
        let value_len = value.len();
        btree.insert(key, value);
        let mut f = OpenOptions::new()
            .read(true).write(true)
            .open(p).unwrap();
        let mut buf = Vec::with_capacity(PAGE_SIZE);
        let _ = f.read_to_end(&mut buf);
        
        let res = file_bytes(p);

        let slot_len = key.to_le_bytes().len() + value_len + 3;
        let res = &res[PAGE_SIZE - slot_len..PAGE_SIZE];

        let _ = remove_file(p);
        assert_eq!(res, [1, 123, 3, 0, 97, 98, 99]); 
    }

    #[test]
    fn test_insert_multi() {
        let p = "test_insert_multi";
        let mut btree = BTree::create(p);
        btree.insert(13u16, "abc".to_string());
        btree.insert(8976u16, "ありがと".to_string());
        let res = file_bytes(p);
        let _ = remove_file(p);
        assert_eq!(res, [2, 0, 39, 0, 0, 0, 0, 0, 56, 0, 39, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 16, 35, 12, 0, 227, 129, 130, 227, 130, 138, 227, 129, 140, 227, 129, 168, 2, 13, 0, 3, 0, 97, 98, 99]); 
    }

    // #[test]
    // fn test_insert_split() {
    //     let p = "test_insert_split";
    //     let mut btree = BTree::create(p);
    //     btree.insert(13u16, "abc".to_string());
    //     btree.insert(2000u16, "defg".to_string());
    //     btree.insert(8976u16, "ありがと".to_string());
    //     btree.insert(7u16, "ぽぽ".to_string());
    //     let res = file_bytes(p);
    //     let _ = remove_file(p);
    //     assert_eq!(res, []); 
    // }

    fn file_bytes(path: impl AsRef<Path>) -> Vec<u8> {
        let mut f = OpenOptions::new()
            .read(true).write(true)
            .open(path).unwrap();
        let mut buf = Vec::with_capacity(PAGE_SIZE);
        let _ = f.read_to_end(&mut buf);
        buf
    }
}

