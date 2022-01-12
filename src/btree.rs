use std::cell::RefCell;
use std::path::Path;
use crate::error::Error;
use crate::branch::Branch;
use crate::leaf::Leaf;
use crate::node::Node;
use crate::node::NodeType;
use crate::page::Page;
use crate::slot::Slot;
use crate::slot::SlotBytes;
use crate::slot::SlotValue;
use crate::storage::Storage;


pub struct BTree<K, V> {
    root_page_id: Option<u16>,
    storage: RefCell<Storage<K, V>>,
}

impl<K: Ord + SlotBytes + std::fmt::Debug, V: Clone> BTree<K, V> {
    pub fn create(file_path: impl AsRef<Path>) -> Self {
        let storage = Storage::from_path(file_path);
        BTree { root_page_id: Default::default(), storage: RefCell::new(storage) }
    }

    pub fn search(&self, key: &K) -> Result<V, Error> where 
        V: SlotBytes
    {
        if let Some(root_page_id) = self.root_page_id {
            let mut breadcrumb = vec![];
            self.search_internal(root_page_id, key, &mut breadcrumb)
                .map(|v| v.leaf_value())
        } else {
            Err(Error::NoPage)
        }
    }

    fn search_internal(&self, page_id: u16, key: &K, breadcrumb: &mut Vec<u16>) -> Result<SlotValue<V>, Error>
        where V: SlotBytes
    {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);
        let node = Node::<K, SlotValue<V>>::new(page);

        match node.node_type() {
            NodeType::Leaf => {
                if let Some(v) = node.search(key) {
                    Ok(v)
                } else {
                    Err(Error::NotFound)
                }
            },
            NodeType::Branch => {
                let branch = Branch::new(node);
                breadcrumb.push(branch.node.page.id);
                for k in branch.node.keys() {
                    if key < &k {
                        let child_page_id = branch.node.search(&k).map(|v| v.page_id()).unwrap();
                        return self.search_internal(child_page_id, key, breadcrumb);
                    }
                }
                self.search_internal(branch.max_page_id(), key, breadcrumb)
            },
        }
    }

    pub fn insert(&mut self, key: K, value: V) where
        K: SlotBytes + Clone,
        V: SlotBytes + Clone,
    {
        if let Some(root_page_id) = self.root_page_id {
            let mut breadcrumb = vec![];
            self.insert_internal(root_page_id, key, value, &mut breadcrumb);
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

    fn insert_internal<Val>(&mut self, page_id: u16, key: K, value: Val, breadcrumb: &mut Vec<u16>) where
        K: SlotBytes + Clone,
        Val: SlotBytes + Clone,
    {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);

        match page.node_type() {
            NodeType::Leaf => {
                let mut node = Node::<K, Val>::new(page);
                let slot = Slot::new(key, value);
                match node.insert(&slot) {
                    Ok(_) => {
                        let mut leaf = Leaf::new(node);
                        self.write_leaf(&mut leaf);
                    },
                    Err(_) => {
                        self.split(&mut node, slot, breadcrumb);
                    },
                }

            },
            NodeType::Branch => {
                let node = Node::<K, u16>::new(page);
                let branch = Branch::new(node);
                breadcrumb.push(branch.node.page.id);
                for k in branch.node.keys() {
                    if key < k {
                        // let child_page_id = branch.node.search(&k).map(|v| v.page_id()).unwrap();
                        let child_page_id = branch.node.search(&k).unwrap();
                        return self.insert_internal(child_page_id, key, value, breadcrumb);
                    }
                }
                self.insert_internal(branch.max_page_id(), key, value, breadcrumb)
            },
        }
    }

    fn split<Val>(&mut self, node: &mut Node<K, Val>, slot: Slot<K, Val>, breadcrumb: &mut Vec<u16>) where
        K: SlotBytes + Clone,
        Val: SlotBytes + Clone, 
    {
        let new_page = self.storage.borrow_mut().allocate_page();

        match new_page.node_type() {
            NodeType::Leaf => {
                let mut new_node = Node::<K, Val>::create(new_page);
                new_node.set_node_type(node.node_type());

                let mut keys = node.keys();
                keys.push(slot.key.clone());
                keys.sort();

                let mut new_slot_inserted = false;
                for key in keys.split_at(keys.len() / 2).1.iter().rev() {
                    match node.search(key) {
                        Some(value) => {
                            let _ = new_node.insert(&Slot::new(key.clone(), value));
                            let _ = node.delete(key);
                        },
                        None => {
                            let _ = new_node.insert(&slot);
                            new_slot_inserted = true
                        }
                    }
                }
                if !new_slot_inserted {
                    let _ = node.insert(&slot);
                }

                // add new branch
                let page = if breadcrumb.is_empty() {
                    self.storage.borrow_mut().allocate_page()
                } else {
                    // let page_id = breadcrumb.last().unwrap().clone();
                    let page_id = breadcrumb.pop().unwrap();
                    let mut page = Page::new(page_id);
                    self.storage.borrow_mut().read_page(&mut page);
                    page
                };
                let mut parent_branch = Branch::new(Node::<K, u16>::create(page));

                let split_key = keys.split_at(keys.len() / 2).1.iter().next().unwrap();

                if breadcrumb.is_empty() {
                    let _ = parent_branch.node.insert(&Slot::new(split_key.clone(), node.page.id));

                    // set root page id
                    self.set_root_page_id(parent_branch.node.page.id);

                    parent_branch.set_max_page_id(new_node.page.id);
                } else {
                    let _ = self.insert_internal(parent_branch.node.page.id, split_key.clone(), node.page.id, breadcrumb);
                }
                
                // println!("node: {:?}", node.page.bytes);
                // println!("new_node: {:?}", new_node.page.bytes);
                // println!("new_branch: {:?}", new_branch.node.page.bytes);

                self.storage.borrow_mut().write_page(&mut node.page);
                self.storage.borrow_mut().write_page(&mut new_node.page);
                self.storage.borrow_mut().write_page(&mut parent_branch.node.page);
            },
            NodeType::Branch => {
                unimplemented!()
            },
        }
    }

    fn create_leaf(&self) -> Leaf<K, V> {
        let page = self.storage.borrow_mut().allocate_page();
        let mut node = Node::<K, V>::create(page);
        node.set_node_type(NodeType::Leaf);
        Leaf { node: node }
    }

    fn write_leaf<Val>(&self, leaf: &mut Leaf<K, Val>) {
        self.storage.borrow_mut().write_page(&mut leaf.node.page);
    }

    fn read_node(&self, page_id: u16) -> Node<K, V> {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);
        Node::<K, V>::new(page)
    }

    fn set_root_page_id(&mut self, page_id: u16) {
        self.root_page_id = Some(page_id);
    }
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
    use crate::slot::Slot;


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

        let slot_len = key.to_le_bytes().len() + value_len + 4;
        let res = &res[PAGE_SIZE - slot_len..PAGE_SIZE];

        let _ = remove_file(p);
        assert_eq!(res, [1, 0, 123, 3, 0, 97, 98, 99]);
    }

    #[test]
    fn test_insert_multi() {
        let p = "test_insert_multi";
        let mut btree = BTree::create(p);
        btree.insert(13u16, "abc".to_string());
        btree.insert(8976u16, "ありがと".to_string());
        let res = file_bytes(p);
        let _ = remove_file(p);
        assert_eq!(res, [2, 0, 37, 0, 0, 0, 0, 0, 55, 0, 37, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 16, 35, 12, 0, 227, 129, 130, 227, 130, 138, 227, 129, 140, 227, 129, 168, 2, 0, 13, 0, 3, 0, 97, 98, 99]); 
    }

    #[test]
    fn test_insert_split() {
        let p = "test_insert_split";
        let mut btree = BTree::create(p);
        btree.insert(22u16, "abc".to_string());
        btree.insert(55u16, "defg".to_string());
        btree.insert(33u16, "あ".to_string());
        btree.insert(66u16, "い".to_string());
        // btree.insert(11u16, "ぽ".to_string());

        let mut node = btree.read_node(btree.root_page_id.unwrap());
        let mut breadcrumb = vec![];
        btree.split(&mut node, Slot::new(44u16, "あふれちゃう".to_string()), &mut breadcrumb);

        let _ = remove_file(p);
        // assert_eq!(res, []);
    }

    #[test]
    fn test_search_split() {
        let p = "test_search_split";
        let mut btree = BTree::create(p);
        btree.insert(22u16, "abc".to_string());
        btree.insert(55u16, "defg".to_string());
        btree.insert(33u16, "あ".to_string());
        btree.insert(66u16, "い".to_string());
        btree.insert(44u16, "あふれちゃう".to_string());

        let _ = remove_file(p);
        assert_eq!(btree.search(&33), Ok("あ".to_string()));
        assert_eq!(btree.search(&44), Ok("あふれちゃう".to_string()));
    }

    fn file_bytes(path: impl AsRef<Path>) -> Vec<u8> {
        let mut f = OpenOptions::new()
            .read(true).write(true)
            .open(path).unwrap();
        let mut buf = Vec::with_capacity(PAGE_SIZE);
        let _ = f.read_to_end(&mut buf);
        buf
    }
}

