#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Po {
    po5: u128,
    po1: u16,
    po2: u16,
    po3: u32,
    po4: u64,
    
}

use std::io::Write;
fn w() -> std::io::Result<()> {
    use std::fs::File;
    let mut f = File::create("f")?;
    // use std::io::BufWriter;
    // let mut writer = BufWriter::new(f);
    f.write_all(b"aqs23   153sw74defvo89")
}

#[test]
fn test() {
    let _ = w();
    // let _po = Po {po1: 0, po2: 0, po3: 0};
    let arr: [u8; 32] = [0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31];
    let popo_ref = arr.as_ptr() as *const Po;
    unsafe {
        let popo = *popo_ref;
        println!("{:x?}", popo);
    }

    // let mut a = *b"abcde";
    // // a.copy_within(0..2, 3);
    // // a.copy_within(0..3, 2);
    // a.copy_within(2..5, 0);
    // println!("a:{:x?}", a);

    assert_eq!(200u8 as i8, -10);

    let bytes = [0b00101010,0b00101010];
    assert_eq!(a(bytes), NodeType::Leaf);
}

#[derive(PartialEq, Debug)]
enum NodeType {
    Leaf,
    Branch,
}

fn a(v: [u8; 2]) -> NodeType {
    let i = i16::from_le_bytes(v);
    if i < 0 {
        NodeType::Leaf
    } else {
        NodeType::Branch
    }
}