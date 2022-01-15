mod fmt;
#[cfg(test)] mod test;

use std::cell::RefCell;
use std::fmt::Debug;
use std::path::Path;

use crate::error::Error;
use crate::branch::Branch;
use crate::leaf::Leaf;
use crate::node::Node;
use crate::node::NodeType;
use crate::page::Page;
use crate::slot::Slot;
use crate::slot::SlotBytes;
use crate::slotted::Slotted;
use crate::slotted::pointer::BranchPointer;
use crate::slotted::pointer::LeafPointer;
use crate::slotted::pointer::Pointer;
use crate::storage::Storage;


pub struct BTree<K, V> {
    root_page_id: Option<u16>,
    storage: RefCell<Storage<K, V>>,
}

impl<K: Ord + SlotBytes + std::fmt::Debug, V: SlotBytes + Clone + Debug> BTree<K, V> {
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
        } else {
            Err(Error::NoPage)
        }
    }

    pub fn insert(&mut self, key: K, value: V)
        where
            K: SlotBytes + Clone,
            V: SlotBytes + Clone,
    {
        if let Some(root_page_id) = self.root_page_id {
            let mut breadcrumb = vec![];
            self.insert_internal(root_page_id, key, value, &mut breadcrumb);
        } else {
            let mut leaf = self.create_leaf();
            let slot = Slot::new(key, value);
            let _ = leaf.slotted.insert(&slot);
            self.write_leaf(&mut leaf);
            self.root_page_id = Some(0);
        }
    }

    pub fn delete(&mut self, key: &K) where K: SlotBytes {
        if let Some(root_page_id) = self.root_page_id {
            let node = self.read_node(root_page_id);
            match node {
                Node::Leaf(mut leaf) => {
                    let _ = leaf.slotted.delete(key);
                },
                Node::Branch(_branch) => unimplemented!(),
            }
        }
    }

    fn search_internal<Val: Debug>(&self, page_id: u16, key: &K, breadcrumb: &mut Vec<u16>) -> Result<Val, Error>
        where Val: SlotBytes
    {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);

        match Node::new(page) {
            Node::Leaf(leaf) => {
                leaf.slotted.search(key).ok_or(Error::NotFound)
            },
            Node::Branch(branch) => {
                breadcrumb.push(branch.slotted.page.id);
                for k in branch.slotted.keys() {
                    if key < &k {
                        let child_page_id = branch.slotted.search(&k).unwrap();
                        return self.search_internal(child_page_id, key, breadcrumb);
                    }
                }
                self.search_internal(branch.max_page_id(), key, breadcrumb)
            },
        }
    }
    fn insert_internal<Val: Debug>(&mut self, page_id: u16, key: K, value: Val, breadcrumb: &mut Vec<u16>) 
        where
            K: SlotBytes + Clone,
            Val: SlotBytes + Clone,
    {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);

        match Node::new(page) {
            Node::Leaf(mut leaf) => {
                // let mut node = Slotted::<K, Val>::new(page);
                let slot = Slot::new(key, value);
                match leaf.slotted.insert(&slot) {
                    Ok(_) => {
                        // let mut leaf = Leaf::new(leaf.node);
                        self.write_leaf(&mut leaf);
                    },
                    Err(_) => {
                        self.split(&mut leaf.slotted, slot, breadcrumb);
                    },
                }

            },
            Node::Branch(branch) => {
                breadcrumb.push(branch.slotted.page.id);
                for k in branch.slotted.keys() {
                    if key < k {
                        let child_page_id = branch.slotted.search(&k).unwrap();
                        return self.insert_internal(child_page_id, key, value, breadcrumb);
                    }
                }
                self.insert_internal(branch.max_page_id(), key, value, breadcrumb)
            },
        }
    }

    fn split<Val: Debug, P: Pointer>(&mut self, slotted: &mut Slotted<K, Val, P>, slot: Slot<K, Val>, breadcrumb: &mut Vec<u16>)
        where K: SlotBytes + Clone,
              Val: SlotBytes + Clone, 
    {
        let new_page = self.storage.borrow_mut().allocate_page();

        match Node::create(new_page) {
            Node::Leaf(mut leaf) => {
                let mut keys = slotted.keys();
                keys.push(slot.key.clone());
                keys.sort();

                let mut new_slot_inserted = false;
                for key in keys.split_at(keys.len() / 2).1.iter().rev() {
                    match slotted.search(key) {
                        Some(value) => {
                            let _ = leaf.slotted.insert(&Slot::new(key.clone(), value));
                            let _ = slotted.delete(key);
                        },
                        None => {
                            let _ = leaf.slotted.insert(&slot);
                            new_slot_inserted = true
                        }
                    }
                }

                if !new_slot_inserted {
                    let _ = slotted.insert(&slot);
                }

                // add new branch
                let page = if breadcrumb.is_empty() {
                    self.storage.borrow_mut().allocate_page()
                } else {
                    let page_id = breadcrumb.pop().unwrap();
                    let mut page = Page::new(page_id);
                    self.storage.borrow_mut().read_page(&mut page);
                    page
                };
                let mut parent_branch = Branch::new(Slotted::<K, u16, BranchPointer>::create(page));

                let split_key = keys.split_at(keys.len() / 2).1.iter().next().unwrap();

                if breadcrumb.is_empty() {
                    let _ = parent_branch.slotted.insert(&Slot::new(split_key.clone(), slotted.page.id));

                    // set root page id
                    self.set_root_page_id(parent_branch.slotted.page.id);

                    parent_branch.set_max_page_id(leaf.slotted.page.id);
                } else {
                    let _ = self.insert_internal(parent_branch.slotted.page.id, split_key.clone(), slotted.page.id, breadcrumb);
                }

                self.storage.borrow_mut().write_page(&mut slotted.page);
                self.storage.borrow_mut().write_page(&mut leaf.slotted.page);
                self.storage.borrow_mut().write_page(&mut parent_branch.slotted.page);
            },
            Node::Branch(_branch) => {
                unimplemented!()
            },
        }
    }

    fn create_leaf(&self) -> Leaf<K, V> {
        let page = self.storage.borrow_mut().allocate_page();
        let mut slotted = Slotted::<K, V, LeafPointer>::create(page);
        slotted.set_node_type(NodeType::Leaf);
        Leaf { slotted: slotted }
    }

    fn write_leaf<Val: SlotBytes>(&self, leaf: &mut Leaf<K, Val>) {
        self.storage.borrow_mut().write_page(&mut leaf.slotted.page);
    }

    fn read_node(&self, page_id: u16) -> Node<K, V> {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);
        Node::new(page)
    }

    fn set_root_page_id(&mut self, page_id: u16) {
        self.root_page_id = Some(page_id);
    }
}

