use crate::node::NodeType;
use crate::slotted::Slotted;
use crate::slot::SlotBytes;
use crate::slotted::pointer::BranchPointer;

pub struct Branch<K: Ord + SlotBytes> { pub node: Slotted<K, u16, BranchPointer> }

impl<K: Ord + SlotBytes> Branch<K> {
    pub fn new(mut node: Slotted<K, u16, BranchPointer>) -> Self {
        node.set_node_type(NodeType::Branch);
        Branch { node: node }
    }

    pub fn set_max_page_id(&mut self, number: u16) {
        self.node.page.set_u16_bytes(4, number);
    }

    pub fn max_page_id(&self) -> u16 {
        self.node.page.u16_bytes(4)
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Header {
    number_of_pointer: u16,
    end_of_free_space: u16,
    max_pointer: u16,
    _padding3: u16,
}

