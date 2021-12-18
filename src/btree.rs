use crate::{
    error::Error,
    leaf::Leaf,
    branch::Branch,
};

#[repr(C)]
#[derive(Debug)]
pub struct BTree {
    root: Branch,
    leaf: Leaf,
}

impl BTree {
    pub fn new() -> Self {
        let mut root = Branch::new();
        root.set_max_pointer(1);

        let mut leaf = Leaf::new();
        let _ = leaf.page.load();

        BTree { root: root, leaf: leaf }
    }

    pub fn add(&mut self, key: u16, value: String) {
        if self.leaf.can_add(value.len() as u16) {
            self.leaf.add(key, value);
            let _ = self.leaf.page.save();
        } else {
            println!("can not add");
        }
    }

    pub fn search(&self, searching_key: u16) -> Result<String, Error> {
        let next_pointer = self.root.max_pointer();
        let leaf = self.leaf(next_pointer);
        leaf.search(searching_key)
    }

    fn leaf<'a>(&'a self, _pointer: u16) -> &'a Leaf {
        &self.leaf
    }
}

#[test]
fn test() {
    let btree = BTree::new();
    assert_eq!(btree.root.max_pointer(), 1);
}