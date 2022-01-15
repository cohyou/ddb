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
            let mut page = Page::new(root_page_id);
            self.storage.borrow_mut().read_page(&mut page);
            let node: Node<K, V> = Node::new(page);
            writeln!(f, "node: {:?}", node)
        } else {
            writeln!(f, "<empty>", )
        }
    }
}
