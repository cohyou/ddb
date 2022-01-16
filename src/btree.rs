mod fmt;
#[cfg(test)] mod test;

use std::cell::RefCell;
use std::fmt::Debug;
use std::path::Path;

use crate::error::Error;
use crate::branch::Branch;
use crate::leaf::Leaf;
use crate::meta::Meta;
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

impl<K, V> BTree<K, V>
    where K: Ord + SlotBytes + Debug,
          V: SlotBytes + Clone + Debug,
{
    pub fn create(file_path: impl AsRef<Path>) -> Self {
        let mut storage = Storage::from_path(file_path);

        let root_page_id = if storage.next_page_id > 0 {
            let mut meta_page = Page::new(0);
            storage.read_page(&mut meta_page);
            let meta = Meta::new(meta_page);    
            Some(meta.root_page_id())
        } else {
            Default::default()
        };

        BTree {
            root_page_id: root_page_id,
            storage: RefCell::new(storage),
        }
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
            let root_page_id = 1;
            self.root_page_id = Some(root_page_id);

            let meta_page = self.storage.borrow_mut().allocate_page();
            let mut meta = Meta::new(meta_page);
            meta.set_root_page_id(root_page_id);
            self.storage.borrow_mut().write_page(&mut meta.page);

            let mut leaf = self.create_leaf();
            let slot = Slot::new(key, value);
            let _ = leaf.slotted.insert(&slot);
            self.write_leaf(&mut leaf);
        }
    }

    pub fn delete(&mut self, key: &K) where K: SlotBytes {
        if let Some(root_page_id) = self.root_page_id {
            self.delete_internal(root_page_id, key);
        }
    }

    fn delete_internal(&mut self, page_id: u16, key: &K) {
        println!("delete_internal: page_id: {:?} key: {:?}", page_id, key);
        let node = self.read_node(page_id);
        match node {
            Node::Leaf(mut leaf) => {
                let _ = leaf.slotted.delete(key);
            },
            Node::Branch(branch) => {
                for k in branch.slotted.keys() {
                    if key < &k {
                        let child_page_id = branch.slotted.search(&k).unwrap();
                        return self.delete_internal(child_page_id, key);
                    }
                }
                self.delete_internal(branch.max_page_id(), key);
            },
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
        // println!("insert_internal: page_id: {:?} key: {:?} value: {:?} breadcrumb: {:?}", &page_id, &key, &value, &breadcrumb);
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);

        match Node::new(page) {
            Node::Leaf(mut leaf) => {
                let slot = Slot::new(key, value);
                match leaf.slotted.insert(&slot) {
                    Ok(_) => {
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

    fn split<Val, Ptr>(&mut self, slotted: &mut Slotted<K, Val, Ptr>, slot: Slot<K, Val>, breadcrumb: &mut Vec<u16>)
        where K: SlotBytes + Clone,
              Val: SlotBytes + Clone + Debug,
              Ptr: Pointer + Debug,
    {
        // println!("split: slotted: {:?} slot: {:?} breadcrumb: {:?}", &slotted.slots(), &slot, &breadcrumb);

        let new_page = self.storage.borrow_mut().allocate_page();
        let mut new_slotted = Slotted::<K, Val, Ptr>::create(new_page);
        new_slotted.set_node_type(NodeType::new(&slotted.page));

        let mut keys = slotted.keys();

        self.transfer_slots(&mut keys, slotted, &mut new_slotted, &slot);

        let mut parent_branch = self.parent_branch(&mut new_slotted, breadcrumb);

        self.update_parent_branch(&keys, slotted, &mut new_slotted, breadcrumb, &mut parent_branch);

        self.write_splitted_pages(slotted, &mut new_slotted, &mut parent_branch);
    }

    fn transfer_slots<Val, Ptr>(&mut self,
        keys: &mut Vec<K>,
        old_slotted: &mut Slotted<K, Val, Ptr>, 
        new_slotted: &mut Slotted<K, Val, Ptr>, 
        slot: &Slot<K, Val>
    )
        where K: SlotBytes + Clone,
              Val: SlotBytes + Clone + Debug,
              Ptr: Pointer + Debug,
    {
        keys.push(slot.key.clone());
        keys.sort();
        let mut new_slot_inserted = false;
        for key in keys.split_at(keys.len() / 2).1.iter().rev() {
            match old_slotted.search(key) {
                Some(value) => {
                    let _ = new_slotted.insert(&Slot::new(key.clone(), value));
                    let _ = old_slotted.delete(key);
                },
                None => {
                    let _ = new_slotted.insert(&slot);
                    new_slot_inserted = true
                }
            }
        }

        // transfer max_page_id
        if NodeType::new(&old_slotted.page) == NodeType::Branch {
            let max_page_id = old_slotted.page.u16_bytes(4);
            // 0 is treated as invalid page_id
            old_slotted.page.set_u16_bytes(4, 0);
            new_slotted.page.set_u16_bytes(4, max_page_id);
        }

        if !new_slot_inserted {
            let _ = old_slotted.insert(&slot);
        }

        // println!("splitted! old: {:?} new: {:?}", &old_slotted, &new_slotted);
    }

    fn parent_branch<Val, Ptr>(&mut self,
        new_slotted: &mut Slotted<K, Val, Ptr>, 
        breadcrumb: &mut Vec<u16>
    ) -> Branch<K>
        where K: SlotBytes + Clone,
            Val: SlotBytes + Clone + Debug,
            Ptr: Pointer + Debug,
    {
        let parent_branch = if breadcrumb.is_empty() {
            // add new branch
            let page = self.storage.borrow_mut().allocate_page();
            let parent_slotted = Slotted::<K, u16, BranchPointer>::create(page);
            let mut branch = Branch::new(parent_slotted);
            branch.set_max_page_id(new_slotted.page.id);
            branch
        } else {
            let page_id = breadcrumb[breadcrumb.len() - 1];
            let mut page = Page::new(page_id);
            self.storage.borrow_mut().read_page(&mut page);
            let parent_slotted = Slotted::<K, u16, BranchPointer>::new(page);
            Branch::new(parent_slotted)
        };
        // println!("parent_branch: {:?}", &parent_branch);
        parent_branch
    }

    fn update_parent_branch<Val, Ptr>(&mut self,
        keys: &Vec<K>,
        old_slotted: &mut Slotted<K, Val, Ptr>, 
        new_slotted: &mut Slotted<K, Val, Ptr>, 
        breadcrumb: &mut Vec<u16>, 
        parent_branch: &mut Branch<K>
    )
        where K: SlotBytes + Clone,
            Val: SlotBytes + Clone + Debug,
            Ptr: Pointer + Debug,
    {
        let mut keys_iter = keys.split_at(keys.len() / 2).1.iter();
        let split_key = keys_iter.next().unwrap();

        if breadcrumb.is_empty() {
            let _ = parent_branch.slotted.insert(&Slot::new(split_key.clone(), old_slotted.page.id));

            // set root page id
            self.set_root_page_id(parent_branch.slotted.page.id);
        } else {
            breadcrumb.pop();
            let _ = self.insert_page_id_into_branch(parent_branch, split_key.clone(), old_slotted.page.id, breadcrumb);
            // println!("slotted.page.id: {:?} parent_branch.max_page_id: {:?}", old_slotted.page.id, parent_branch.max_page_id());
            if old_slotted.page.id == parent_branch.max_page_id() {
                parent_branch.set_max_page_id(new_slotted.page.id);
            } else {
                let slots = parent_branch.slotted.slots();
                let rewriting_key = slots.iter().rfind(|(_k, v)| v == &old_slotted.page.id).unwrap();
                // println!("rewriting_key: {:?}", rewriting_key);

                let _ = parent_branch.slotted.delete(&rewriting_key.0);
                let _ = parent_branch.slotted.insert(&Slot::new(rewriting_key.0.clone(), new_slotted.page.id));
            }
        }
    }

    fn write_splitted_pages<Val, Ptr>(&mut self, 
        old_slotted: &mut Slotted<K, Val, Ptr>, 
        new_slotted: &mut Slotted<K, Val, Ptr>, 
        parent_branch: &mut Branch<K>
    )
        where K: SlotBytes + Clone,
            Val: SlotBytes + Clone + Debug,
            Ptr: Pointer + Debug,
    {
        self.storage.borrow_mut().write_page(&mut old_slotted.page);
        self.storage.borrow_mut().write_page(&mut new_slotted.page);
        self.storage.borrow_mut().write_page(&mut parent_branch.slotted.page);
    }

    fn insert_page_id_into_branch(&mut self, branch: &mut Branch<K>, key: K, value: u16, breadcrumb: &mut Vec<u16>) 
        where K: SlotBytes + Clone,
    {
        // println!("insert_page_id_into_branch: branch: {:?} key: {:?} value: {:?}", branch, key, value);
        let slot = Slot::new(key, value);
        match branch.slotted.insert(&slot) {
            Ok(_) => {
                self.storage.borrow_mut().write_page(&mut branch.slotted.page);
            },
            Err(_) => {
                self.split(&mut branch.slotted, slot, breadcrumb);
            },
        }
    }

    fn create_leaf(&self) -> Leaf<K, V> {
        let page = self.storage.borrow_mut().allocate_page();
        let mut slotted = Slotted::<K, V, LeafPointer>::create(page);
        slotted.set_node_type(NodeType::Leaf);
        Leaf { slotted: slotted }
    }

    fn write_leaf<Val: SlotBytes + Debug>(&self, leaf: &mut Leaf<K, Val>) {
        self.storage.borrow_mut().write_page(&mut leaf.slotted.page);
    }

    fn read_node(&self, page_id: u16) -> Node<K, V> {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);
        Node::new(page)
    }

    fn set_root_page_id(&mut self, page_id: u16) {
        self.root_page_id = Some(page_id);
        let mut page = Page::new(0);
        self.storage.borrow_mut().read_page(&mut page);
        let mut meta = Meta::new(page);
        meta.set_root_page_id(self.root_page_id.unwrap());
        self.storage.borrow_mut().write_page(&mut meta.page);
    }
}

