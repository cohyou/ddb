use crate::node::Node;
use crate::node::NodeType;
use crate::slot::SlotBytes;


pub struct Branch<K: Ord + SlotBytes, V> { pub node: Node<K, V> }

impl<K: Ord + SlotBytes, V> Branch<K, V> {
    pub fn new(mut node: Node<K, V>) -> Self {
        node.set_node_type(NodeType::Branch);
        Branch { node: node }
    }

    pub fn set_max_page_id(&mut self, number: u16) {
        self.node.page.set_u16_bytes(6, number);
    }

    pub fn max_page_id(&self) -> u16 {
        self.node.page.u16_bytes(6)
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

#[repr(C)]
#[derive(Debug)]
struct Pointer(pub u16);
