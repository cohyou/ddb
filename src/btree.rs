use crate::{
    error::Error,
    leaf::Leaf,
    branch::Branch,
};

#[repr(C)]
#[derive(Debug)]
pub struct BTree {
    root: Branch,
    branches: Vec<Branch>,
    leaves: Vec<Leaf>,
}

impl BTree {
    pub fn new() -> Self {
        let mut root = Branch::new();
        root.set_max_pointer(1);

        let mut leaf = Leaf::new();
        let _ = leaf.page.load();

        BTree { root: root, branches: vec![], leaves: vec![leaf] }
    }

    pub fn add(&mut self, key: u16, value: String) {
        let leaf_index = self.get_target_leaf_index(key) as usize;
        if self.leaves[leaf_index].can_add(value.len() as u16) {
            let _ = self.leaves[leaf_index].add(key, value);
            let _ = self.leaves[leaf_index].page.save();
        } else {
            println!("can not add");
            self.add_branch(key);
        }
    }

    pub fn search(&self, key: u16) -> Result<String, Error> {
        let leaf_index = self.search_internal(key)?;
        let leaf = self.leaf(leaf_index);

        if let Some(leaf) = leaf {
            leaf.search(key)
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn list(&self) -> Vec<u16> {
        let leaf = &self.leaves[0];
        println!("leaf.pointers: {:?}", leaf.list());
        vec![]
    }

    fn search_internal(&self, key: u16) -> Result<u16, Error>  {
        let next_pointer = self.root.search(key)?;
        Ok(next_pointer)
    }

    fn get_target_branch_mut<'a>(&'a mut self, _key: u16) -> &'a mut Branch {
        &mut self.root
    }

    fn get_target_leaf_index(&self, _key: u16) -> u16 {
        0
    }

    fn add_branch(&mut self, key: u16) {        
        let branch = self.get_target_branch_mut(key);
        branch.add()
    }

    fn leaf<'a>(&'a self, pointer: u16) -> Option<&'a Leaf> {
        let leaf_index = pointer - 1;
        self.leaves.get(leaf_index as usize)
    }
}

#[test]
fn test() {
    let btree = BTree::new();
    assert_eq!(btree.root.max_pointer(), 1);
}

#[test]
fn test2() {
    let mut btree = BTree::new();
    assert_eq!(btree.root.max_pointer(), 1);

    btree.add(13, "abc".to_string());
    btree.add(2000, "defg".to_string());
    btree.add(200, "こんにちは".to_string());
    btree.add(8976, "ありがと".to_string());
    btree.add(6, "ぽ".to_string());

    let res = btree.search(13);
    println!("search 13: {:?}", res);
    let res = btree.search(2000);
    println!("search 2000: {:?}", res);
    let res = btree.search(200);
    println!("search 200: {:?}", res);
    let res = btree.search(8976);
    println!("search 8976: {:?}", res);
    let res = btree.search(6);
    println!("search 6: {:?}", res);

    println!("leaf: {:?}", btree);
}
