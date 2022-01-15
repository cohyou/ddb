use crate::branch::Branch;
use crate::leaf::Leaf;
use crate::page::Page;
use crate::slot::SlotBytes;
use crate::slotted::Slotted;
use crate::slotted::pointer::BranchPointer;
use crate::slotted::pointer::LeafPointer;


pub enum Node<K: Ord + SlotBytes, V> {
    Leaf(Leaf<K, V>),
    Branch(Branch<K>),
}

impl<K: Ord + SlotBytes, V> Node<K, V> {
    pub fn new(page: Page) -> Self {
        match NodeType::new(&page) {
            NodeType::Leaf => {
                let slotted = Slotted::<K, V, LeafPointer>::create(page);
                let leaf = Leaf::new(slotted);
                Node::Leaf(leaf)
            },
            NodeType::Branch => {
                let slotted = Slotted::<K, u16, BranchPointer>::create(page);
                let branch = Branch::new(slotted);
                Node::Branch(branch)
            }
        }
    }
}

pub enum NodeType { Leaf, Branch, }

impl NodeType {
    fn new(page: &Page) -> Self {
        if page.i16_bytes(0) >= 0 {
            NodeType::Leaf
        } else {
            NodeType::Branch
        }
    }
}