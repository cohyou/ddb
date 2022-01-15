use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;

use crate::btree::BTree;

use crate::page::Page;
use crate::node::Node;
use crate::slot::SlotBytes;


impl<K, V> Debug for BTree<K, V>
    where K: Ord + SlotBytes + Debug,
          V: SlotBytes + Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Some(root_page_id) = self.root_page_id {
            self.fmt_internal(f, root_page_id)
        } else {
            writeln!(f, "<empty>", )
        }
    }
}

impl<K, V> BTree<K, V>
    where K: Ord + SlotBytes + Debug,
          V: SlotBytes + Debug,
{
    fn fmt_internal(&self, f: &mut Formatter<'_>, page_id: u16) -> Result<(), Error> {
        let mut page = Page::new(page_id);
        self.storage.borrow_mut().read_page(&mut page);
        let node: Node<K, V> = Node::new(page);
        match node {
            Node::Leaf(leaf) => {
                writeln!(f, "LF{:?}", leaf)
            },
            Node::Branch(branch) => {
                let _ = writeln!(f, "BR{:?}", branch);
                for (_, v) in branch.slotted.slots() {
                    let _ = self.fmt_internal(f, v);
                }
                self.fmt_internal(f, branch.max_page_id())
            },
        }
    }
}