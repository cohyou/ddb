use crate::node::Slotted;
use crate::node::NodeType;
use crate::slot::SlotBytes;

pub struct Leaf<K: Ord + SlotBytes, V> { pub node: Slotted<K, V> }

impl<K: Ord + SlotBytes, V> Leaf<K, V> {
    pub fn new(mut node: Slotted<K, V>) -> Self {
        node.set_node_type(NodeType::Leaf);
        Leaf { node: node }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Header {
    number_of_pointer: u16,
    end_of_free_space: u16,
    _padding2: u16,
    _padding3: u16,
}

#[repr(C)]
#[derive(Debug, PartialEq, Clone, Copy)]
struct Pointer(pub u16);
