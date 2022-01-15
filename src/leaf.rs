use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

use crate::node::NodeType;
use crate::slotted::Slotted;
use crate::slot::SlotBytes;
use crate::slotted::pointer::LeafPointer;


// #[derive(Debug)]
pub struct Leaf<K: Ord + SlotBytes, V: SlotBytes> { pub slotted: Slotted<K, V, LeafPointer> }

impl<K: Ord + SlotBytes, V: SlotBytes> Leaf<K, V> {
    pub fn new(mut slotted: Slotted<K, V, LeafPointer>) -> Self {
        slotted.set_node_type(NodeType::Leaf);
        Leaf { slotted: slotted }
    }
}

impl<K: Ord + SlotBytes + Debug, V: SlotBytes + Debug> Debug for Leaf<K, V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.debug_list()
            .entries(&self.slotted.slots())
            .finish()
    }
}

// #[repr(C)]
// #[derive(Debug, Clone, Copy)]
// struct Header {
//     number_of_pointer: u16,
//     end_of_free_space: u16,
//     _padding2: u16,
//     _padding3: u16,
// }

