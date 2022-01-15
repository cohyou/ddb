use std::fmt::Debug;
// use std::fmt::Error;
// use std::fmt::Formatter;

use crate::branch::Branch;
use crate::leaf::Leaf;
use crate::page::Page;
use crate::slot::SlotBytes;
use crate::slotted::Slotted;
use crate::slotted::pointer::BranchPointer;
use crate::slotted::pointer::LeafPointer;

#[derive(Debug)]
pub enum Node<K: Ord + SlotBytes, V: SlotBytes> {
    Leaf(Leaf<K, V>),
    Branch(Branch<K>),
}

impl<K: Ord + SlotBytes + Debug, V: SlotBytes + Debug> Node<K, V> {
    pub fn new(page: Page) -> Self {
        match NodeType::new(&page) {
            NodeType::Leaf => {
                let slotted = Slotted::<K, V, LeafPointer>::new(page);
                let leaf = Leaf::new(slotted);                
                Node::Leaf(leaf)
            },
            NodeType::Branch => {
                let slotted = Slotted::<K, u16, BranchPointer>::new(page);
                let branch = Branch::new(slotted);
                Node::Branch(branch)
            }
        }
    }

    pub fn create(page: Page) -> Self {
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

// impl<K: Ord + SlotBytes + Debug, V: Debug> Node<K, V> {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
//         match self {
//             Node::Leaf(leaf) => writeln!(f, "{:?}", leaf),
//             Node::Branch(branch) => writeln!(f, "{:?}", branch),
//         }
//     }
// }