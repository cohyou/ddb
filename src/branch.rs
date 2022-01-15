use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

use crate::node::NodeType;
use crate::slotted::Slotted;
use crate::slot::SlotBytes;
use crate::slotted::pointer::BranchPointer;


pub struct Branch<K: Ord + SlotBytes> { pub slotted: Slotted<K, u16, BranchPointer> }

impl<K: Ord + SlotBytes> Branch<K> {
    pub fn new(mut slotted: Slotted<K, u16, BranchPointer>) -> Self {
        slotted.set_node_type(NodeType::Branch);
        Branch { slotted: slotted }
    }

    pub fn set_max_page_id(&mut self, number: u16) {
        self.slotted.page.set_u16_bytes(4, number);
    }

    pub fn max_page_id(&self) -> u16 {
        self.slotted.page.u16_bytes(4)
    }
}

impl<K: Ord + SlotBytes + Debug> Debug for Branch<K> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let _  = write!(f, "({:?}): ", self.slotted.page.id);
        for (k, v) in self.slotted.slots() {
            let _ = write!(f, "{}|<{:?}", v, k);
        }
        write!(f, "|{:?}", self.max_page_id())
    }
}

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// struct Header {
//     number_of_pointer: u16,
//     end_of_free_space: u16,
//     max_pointer: u16,
//     _padding3: u16,
// }

