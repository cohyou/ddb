use crate::error::Error;
use crate::slotted::Slotted;
use crate::page::Page;
use crate::page::PAGE_SIZE;
use crate::slot::Slot;
use crate::slotted::pointer::LeafPointer;


type TestSlotted = Slotted::<u16, String, LeafPointer>;

#[test]
fn test_insert_one() {
    let page = Page::new(Default::default());
    let mut slotted = TestSlotted::create(page);
    let _ = slotted.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = slotted.insert(&Slot::new(7u16, "ありがと".to_string()));
    let _ = slotted.insert(&Slot::new(5u16, "defg".to_string()));
    let _ = slotted.insert(&Slot::new(1u16, "ぽ".to_string()));
    println!("{:?}", &slotted.page);
    println!("{:?}", &slotted);
}

#[test]
fn test_pointers_sorted() {
    let page = Page::new(Default::default());
    let mut slotted = TestSlotted::create(page);
    let _ = slotted.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = slotted.insert(&Slot::new(7u16, "ありがと".to_string()));
    let _ = slotted.insert(&Slot::new(5u16, "defg".to_string()));
    let _ = slotted.insert(&Slot::new(1u16, "ぽ".to_string()));
    let keys = slotted.keys();
    println!("{:?}", &slotted.page);
    assert_eq!(keys, [1, 2, 5, 7]);
}

#[test]
fn test_pointers_full() {
    let page = Page::new(Default::default());
    let mut slotted = TestSlotted::create(page);
    let _ = slotted.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = slotted.insert(&Slot::new(7u16, "ありがと".to_string()));
    let _ = slotted.insert(&Slot::new(5u16, "defg".to_string()));
    let res = slotted.insert(&Slot::new(1u16, "pppppp".to_string()));
    let pointers = slotted.keys();
    println!("{:?}", &slotted.page);
    println!("{:?}", &slotted);
    assert_eq!(pointers, [2, 5, 7]);
    assert_eq!(res, Err(Error::FullLeaf));
}

#[test]
fn test_search_hit() {
    let page = Page::new(Default::default());
    let mut slotted = TestSlotted::create(page);
    let _ = slotted.insert(&Slot::new(2u16, "abc".to_string()));
    assert_eq!(slotted.search(&2u16), Some("abc".to_string()));
}

#[test]
fn test_search_notfound() {
    let page = Page::new(Default::default());
    let mut slotted = TestSlotted::create(page);
    let _ = slotted.insert(&Slot::new(2u16, "abc".to_string()));
    assert_eq!(slotted.search(&5u16), None);
}

#[test]
fn test_delete_notfound() {
    let page = Page::new(Default::default());
    let mut slotted = TestSlotted::create(page);
    let _ = slotted.insert(&Slot::new(2u16, "abc".to_string()));
    assert!(slotted.delete(&5).is_err());
}

#[test]
fn test_delete_one() {
    let page = Page::new(Default::default());
    let mut slotted = TestSlotted::create(page);
    let _ = slotted.insert(&Slot::new(2u16, "abc".to_string()));
    assert!(slotted.delete(&2).is_ok());
    let mut res = [0u8; PAGE_SIZE];
    res[2] = 64;
    assert_eq!(slotted.page.bytes, res);
}

#[test]
fn test_delete_multi() {
    let mut slotted1 = TestSlotted::create(Page::new(Default::default()));
    let target_value = "defg";
    let _ = slotted1.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = slotted1.insert(&Slot::new(7u16, "ありがと".to_string()));
    let _ = slotted1.insert(&Slot::new(5u16, target_value.to_string()));
    assert!(slotted1.delete(&5).is_ok());


    let mut slotted2 = TestSlotted::create(Page::new(Default::default()));
    let _ = slotted2.insert(&Slot::new(2u16, "abc".to_string()));
    let _ = slotted2.insert(&Slot::new(7u16, "ありがと".to_string()));
    assert_eq!(slotted1.page.bytes, slotted2.page.bytes);
}

#[test]
fn test_delete_transfer() {
    let mut slotted1 = TestSlotted::create(Page::new(Default::default()));
    let _ = slotted1.insert(&Slot::new(13u16, "abc".to_string()));
    let _ = slotted1.insert(&Slot::new(7u16, "ぽぽ".to_string()));
    assert!(slotted1.delete(&13).is_ok());

    let mut slotted2 = TestSlotted::create(Page::new(Default::default()));
    let _ = slotted2.insert(&Slot::new(7u16, "ぽぽ".to_string()));
    println!("{:?}", slotted2);

    assert_eq!(slotted1.page.bytes, slotted2.page.bytes);
}
