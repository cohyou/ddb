use std::cell::RefCell;
use std::path::Path;
use crate::error::Error;
use crate::node::Leaf;
use crate::node::Node;
use crate::node::Slot;
use crate::node::SlotBytes;
use crate::storage::Storage;


pub struct BTree {
    root_page_id: Option<u16>,
    storage: RefCell<Storage>,
}

impl BTree {
    pub fn create(file_path: impl AsRef<Path>) -> Self {
        let storage = Storage::from_path(file_path);
        BTree { root_page_id: Default::default(), storage: RefCell::new(storage) }
    }

    pub fn search<T>(&self, _value: T) -> Result<T, Error> {
        if let Some(_root_page_id) = self.root_page_id {
            Err(Error::NoPage)
        } else {
            Err(Error::NoPage)
        }
    }

    pub fn insert<K, V>(&mut self, key: K, value: V)  where 
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        if let Some(_root_page_id) = self.root_page_id {
        } else {
            let mut leaf = self.create_leaf();
            let slot = Slot::new(key, value);
            leaf.node.add_slot(&slot);
            self.write_leaf(&mut leaf);
        }
    }

    fn create_leaf(&self) -> Leaf {
        let page = self.storage.borrow_mut().allocate_page();
        Leaf { node: Node::new(page) }
    }

    fn write_leaf(&self, leaf: &mut Leaf) {
        self.storage.borrow_mut().write_page(&mut leaf.node.page);
    }

    // fn root_page_id(&self) -> u16 { self.root_page_id.unwrap() }
}

#[cfg(test)]
mod test {
    use std::fs::OpenOptions;
    use std::fs::remove_file;
    
    use std::io::Read;

    use crate::btree::BTree;
    use crate::error::Error;
    use crate::page::PAGE_SIZE;

    #[test]
    fn test_search_empty() {
        let p = "test_search_empty";
        let btree = BTree::create(p);
        let error: Result<&str, Error> = Err(Error::NoPage);
        let _ = remove_file(p);
        assert_eq!(btree.search(""), error);
    }

    #[test]
    fn test_insert_first_slot() {
        let key = 123u8;
        let value = "abc";
        let p = "test_insert_first";
        let mut btree = BTree::create(p);
        btree.insert(key, value);
        let mut f = OpenOptions::new()
            .read(true).write(true)
            .open(p).unwrap();
        let mut buf = Vec::with_capacity(PAGE_SIZE);
        let _ = f.read_to_end(&mut buf);
        let slot_len = key.to_le_bytes().len() + value.len() + 3;
        let res = format!("{:?}", &buf[PAGE_SIZE - slot_len..PAGE_SIZE]);
        let _ = remove_file(p);
        assert_eq!(res, "[1, 123, 3, 0, 97, 98, 99]"); 
    }

    // #[test]
    // fn test_insert_multi() {
    //     let p = "test_insert_multi";
    //     let mut btree = BTree::create(p);
    //     btree.insert(13u32, "abc");
    //     btree.insert(2000u32, "defg");
    //     btree.insert(200u32, "こんにちは");
    //     btree.insert(8976u32, "ありがと");
    //     btree.insert(6u32, "ぽ");
    //     let _ = remove_file(p);
    // }
}

// enum NodeType { Leaf, Branch, }
