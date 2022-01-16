use std::fs::File;
// use std::fs::OpenOptions;
use std::fs::remove_file;

// use std::io::Read;

// use std::path::Path;

use crate::btree::BTree;
use crate::error::Error;
use crate::node::Node;
// use crate::page::PAGE_SIZE;
use crate::slot::Slot;


#[test]
fn test_search_empty() {
    let p = "test_search_empty";
    let btree = BTree::<u16, String>::create(p);
    let error: Result<String, Error> = Err(Error::NoPage);
    let _ = remove_file(p);
    assert_eq!(btree.search(&0), error);
}

// #[test]
// fn test_insert_first_slot() {
//     let key = 123u8;
//     let value = "abc".to_string();
//     let p = "test_insert_first";
//     let mut btree = BTree::create(p);
//     let value_len = value.len();
//     btree.insert(key, value);
//     let mut f = OpenOptions::new()
//         .read(true).write(true)
//         .open(p).unwrap();
//     let mut buf = Vec::with_capacity(PAGE_SIZE);
//     let _ = f.read_to_end(&mut buf);
    
//     let res = file_bytes(p);

//     let slot_len = key.to_le_bytes().len() + value_len + 4;
//     let res = &res[PAGE_SIZE - slot_len..PAGE_SIZE];

//     let _ = remove_file(p);
//     assert_eq!(res, [0, 0, 0, 0, 123, 97, 98, 99]);
// }

// #[test]
// fn test_insert_multi() {
//     let p = "test_insert_multi";
//     let mut btree = BTree::create(p);
//     btree.insert(13u16, "abc".to_string());
//     btree.insert(8976u16, "ありがと".to_string());
//     let res = file_bytes(p);
//     let _ = remove_file(p);
//     assert_eq!(res, [2, 0, 45, 0, 0, 0, 0, 0, 59, 0, 2, 0, 3, 0, 45, 0, 2, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 35, 227, 129, 130, 227, 130, 138, 227, 129, 140, 227, 129, 168, 13, 0, 97, 98, 99]); 
// }

#[test]
fn test_insert_split() {
    let p = "test_insert_split";
    let mut btree = BTree::create(p);
    btree.insert(22u16, "abc".to_string());
    btree.insert(55u16, "defg".to_string());
    btree.insert(33u16, "あ".to_string());
    btree.insert(66u16, "い".to_string());
    btree.insert(11u16, "ぽ".to_string());

    match btree.read_node(btree.root_page_id.unwrap()) {
        Node::Leaf(mut leaf) => {
            let mut breadcrumb = vec![];
            btree.split(&mut leaf.slotted, Slot::new(44u16, "あふれちゃう".to_string()), &mut breadcrumb);
        },
        Node::Branch(_) => panic!(""),
    }


    let _ = remove_file(p);
    // assert_eq!(res, []);
}

#[test]
fn test_search_split() {
    let p = "test_search_split";
    let mut btree = BTree::create(p);
    btree.insert(22u16, "abc".to_string());
    btree.insert(55u16, "defg".to_string());
    btree.insert(33u16, "あ".to_string());
    btree.insert(66u16, "い".to_string());
    btree.insert(44u16, "あふれちゃう".to_string());
    // btree.insert(35u16, "add".to_string());
    println!("{:?}", btree);

    let _ = remove_file(p);
    assert_eq!(btree.search(&33), Ok("あ".to_string()));
    assert_eq!(btree.search(&44), Ok("あふれちゃう".to_string()));
}

#[test]
fn test_insert_meta() {
    let p = "test_insert_meta";
    let mut btree = BTree::<u16, String>::create(p);
    btree.insert(22u16, "abc".to_string());

    println!("{:?}", btree);

    let _ = remove_file(p);
}

#[test]
fn test_read_meta() {
    let p = "sample/test_read_meta";
    let mut btree = BTree::<u16, String>::create(p);
    btree.insert(22u16, "abc".to_string());
    btree.insert(55u16, "defg".to_string());
    btree.insert(33u16, "あ".to_string());
    btree.insert(66u16, "い".to_string());
    btree.insert(44u16, "あふれちゃう".to_string());
    println!("{:?}", btree);

    let btree = BTree::<u16, String>::create(p);
    println!("{:?}", btree);

    let _ = remove_file(p);
}

#[test]
fn test_split_multi() {
    let p = "sample/test_split_multi";
    if File::open(p).is_err() {
        let mut btree = BTree::<u16, String>::create(p);
        btree.insert(22u16, "abc".to_string());
        btree.insert(55u16, "defg".to_string());
        btree.insert(33u16, "あ".to_string());
        btree.insert(66u16, "い".to_string());
        btree.insert(44u16, "あふれちゃう".to_string());
        btree.insert(35u16, "add".to_string());    
    }

    let mut btree = BTree::<u16, String>::create(p);
    println!("{:?}", btree);
    if btree.search(&58).is_err() {
        btree.insert(58u16, "i am 58".to_string());
    }

    assert_eq!(btree.search(&33), Ok("あ".to_string()));
}

#[test]
fn test_split_nested() {
    let p = "sample/test_split_nested";
    if File::open(p).is_err() {
        let mut btree = BTree::<u16, String>::create(p);
        btree.insert(22u16, "abc".to_string());
        btree.insert(55u16, "defg".to_string());
        btree.insert(33u16, "あ".to_string());
        btree.insert(66u16, "い".to_string());
        btree.insert(44u16, "あふれちゃう".to_string());
        btree.insert(35u16, "add".to_string());    
        btree.insert(58u16, "i am 58".to_string());
        btree.insert(100, "こんどはどうだ".to_string());
        btree.insert(16, "sixteen".to_string());
    }

    let mut btree = BTree::<u16, String>::create(p);
    println!("{:?}", btree);
    if btree.search(&18).is_err() {
        btree.insert(18, "新成人".to_string());
        println!("{:?}", btree);
    }

    // let _ = remove_file(p);
    assert_eq!(btree.search(&33), Ok("あ".to_string()));
}

#[test]
fn test_split_branch() {
    let p = "sample/test_split_branch";
    if File::open(p).is_err() {
        let mut btree = BTree::<u16, String>::create(p);
        btree.insert(22, "abc".to_string());
        btree.insert(55, "defg".to_string());
        btree.insert(33, "あ".to_string());
        btree.insert(66, "い".to_string());
        btree.insert(44, "あふれちゃう".to_string());
        btree.insert(35, "add".to_string());    
        btree.insert(58, "i am 58".to_string());
        btree.insert(100, "こんどはどうだ".to_string());
        btree.insert(16, "sixteen".to_string());
        btree.insert(18, "新成人".to_string());
        btree.insert(99, "ナインティナ".to_string());
        btree.insert(77, "lucky seven!!".to_string());
        btree.insert(41, "いつまでやるんよ".to_string());
        btree.insert(7, "七転".to_string());
        btree.insert(8, "八倒".to_string());
        btree.insert(25, "around thirty".to_string());
        btree.insert(64, "8bit".to_string());
        btree.insert(13, "金曜日".to_string());
        btree.insert(50, "50:50".to_string());
    }

    let mut btree = BTree::<u16, String>::create(p);
    println!("{:?}", btree);
    if btree.search(&28).is_err() {
        btree.insert(28, "I am perfect number.".to_string());
        println!("{:?}", btree);
    }

    let _ = remove_file(p);
    assert_eq!(btree.search(&28), Ok("I am perfect number.".to_string()));
}

// #[allow(dead_code)]
// fn file_bytes(path: impl AsRef<Path>) -> Vec<u8> {
//     let mut f = OpenOptions::new()
//         .read(true).write(true)
//         .open(path).unwrap();
//     let mut buf = Vec::with_capacity(PAGE_SIZE);
//     let _ = f.read_to_end(&mut buf);
//     buf
// }
