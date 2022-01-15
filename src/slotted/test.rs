use crate::error::Error;
use crate::slotted::Slotted;
use crate::page::Page;
use crate::page::PAGE_SIZE;
use crate::slot::Slot;
use crate::slotted::pointer::LeafPointer;


// const LEN_OF_LEN_OF_POINTER: u16 = 2;
// const LEN_OF_LEN_OF_VALUE: u16 = 2;


type TestSlotted = Slotted::<u16, String, LeafPointer>;

#[test]
fn test_pointers_sorted() {
    let page = Page::new(Default::default());
    let mut node = TestSlotted::create(page);
    let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = node.insert(&Slot::new(7u16, "ありがと".to_string()));
    let _ = node.insert(&Slot::new(5u16, "defg".to_string()));
    let _ = node.insert(&Slot::new(1u16, "ぽ".to_string()));
    let keys = node.keys();
    println!("{:?}", &node.page);
    assert_eq!(keys, [1, 2, 5, 7]);
}

#[test]
fn test_pointers_full() {
    let page = Page::new(Default::default());
    let mut node = TestSlotted::create(page);
    let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = node.insert(&Slot::new(7u16, "ありがと".to_string()));
    let _ = node.insert(&Slot::new(5u16, "defg".to_string()));
    let res = node.insert(&Slot::new(1u16, "pppppp".to_string()));
    // let res = node.insert(&Slot::new(100u16, "あふれちゃう".to_string()));
    let pointers = node.keys();
    println!("{:?}", &node.page);
    assert_eq!(pointers, [2, 5, 7]);
    assert_eq!(res, Err(Error::FullLeaf));
}

#[test]
fn test_search_hit() {
    let page = Page::new(Default::default());
    let mut node = TestSlotted::create(page);
    let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
    assert_eq!(node.search(&2u16), Some("abc".to_string()));
}

#[test]
fn test_search_notfound() {
    let page = Page::new(Default::default());
    let mut node = TestSlotted::create(page);
    let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
    assert_eq!(node.search(&5u16), None);
}

// #[test]
// fn test_search_remove_offset() {
//     let page = Page::new(Default::default());
//     let mut slotted = TestSlotted::create(page);
//     let target_value = "defg";
//     let _ = slotted.insert(&Slot::new(2u16, "abc".to_string()));
//     let _ = slotted.insert(&Slot::new(7u16, "ありがと".to_string()));
//     let _ = slotted.insert(&Slot::new(5u16, target_value.to_string()));
//     assert_eq!(
//         slotted.slot_len(slotted.search_slot_offset(&5).unwrap()),
//         LEN_OF_LEN_OF_POINTER + pointer_size() +
//         LEN_OF_LEN_OF_VALUE + target_value.len()
//     )
// }

#[test]
fn test_delete_notfound() {
    let page = Page::new(Default::default());
    let mut node = TestSlotted::create(page);
    let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
    assert!(node.delete(&5).is_err());
}

#[test]
fn test_delete_one() {
    let page = Page::new(Default::default());
    let mut node = TestSlotted::create(page);
    let _ = node.insert(&Slot::new(2u16, "abc".to_string()));
    assert!(node.delete(&2).is_ok());
    let mut res = [0u8; PAGE_SIZE];
    res[2] = 64;
    assert_eq!(node.page.bytes, res);
}

#[test]
fn test_delete_multi() {
    let mut node1 = TestSlotted::create(Page::new(Default::default()));
    let target_value = "defg";
    let _ = node1.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = node1.insert(&Slot::new(7u16, "ありがと".to_string()));
    let _ = node1.insert(&Slot::new(5u16, target_value.to_string()));
    assert!(node1.delete(&5).is_ok());


    let mut node2 = TestSlotted::create(Page::new(Default::default()));
    let _ = node2.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = node2.insert(&Slot::new(7u16, "ありがと".to_string()));
    assert_eq!(node1.page.bytes, node2.page.bytes);
}

#[test]
fn test_delete_transfer() {
    let mut node1 = TestSlotted::create(Page::new(Default::default()));
    let _ = node1.insert(&Slot::new(13u16, "abc".to_string()));
    let _ = node1.insert(&Slot::new(7u16, "ぽぽ".to_string()));
    assert!(node1.delete(&13).is_ok());

    let mut node2 = TestSlotted::create(Page::new(Default::default()));
    let _ = node2.insert(&Slot::new(7u16, "ぽぽ".to_string()));

    assert_eq!(node1.page.bytes, node2.page.bytes);
}
