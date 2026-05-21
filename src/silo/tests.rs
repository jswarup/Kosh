//-- tests.rs ----------------------------------------------------------------------------------------------------------------------

use crate::silo::buff::Buff;
use crate::silo::useg::USeg;


//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BuffBasicOps()
{
    let mut buff = Buff::new(10, 42);
    assert_eq!(buff.len(), 10);
    assert_eq!(buff[0], 42);
    assert_eq!(buff[1], 42);
    assert_eq!(buff[2], 42);

    buff[1] = 100;
    assert_eq!(buff[1], 100);

    // Test slice methods made available via Deref
    assert_eq!(buff.first(), Some(&42));
    assert_eq!(buff.last(), Some(&42));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BuffFrom()
{
    // Test creation from a slice
    let slice_data = [10, 20, 30];
    let buff_from_slice = Buff::from(&slice_data[..]);
    assert_eq!(buff_from_slice.len(), 3);
    assert_eq!(buff_from_slice[0], 10);
    assert_eq!(buff_from_slice[1], 20);
    assert_eq!(buff_from_slice[2], 30);

    // Test creation from a Vec
    let vec_data = vec![40, 50];
    let buff_from_vec = Buff::from(vec_data);
    assert_eq!(buff_from_vec.len(), 2);
    assert_eq!(buff_from_vec[0], 40);
    assert_eq!(buff_from_vec[1], 50);

    // Test creation from an array directly
    let buff_from_arr = Buff::from([100, 200, 300, 400]);
    assert_eq!(buff_from_arr.len(), 4);
    assert_eq!(buff_from_arr[2], 300);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BufZST()
{
    let buff = Buff::new(10, ());
    assert_eq!(buff.len(), 10);
    assert_eq!(buff[5], ());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BuffSendSync()
{
    let buff = Buff::new(5, 42);
    let handle = std::thread::spawn(move ||
    {
        assert_eq!(buff.len(), 5);
        assert_eq!(buff[0], 42);
    });

    handle.join().unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn ArrBasicOps()
{
    let mut buff = Buff::new(3, 42);
    {
        let mut arr = buff.as_mut_arr();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], 42);
        arr[1] = 100;
    }
    assert_eq!(buff[1], 100);

    let arr2 = buff.as_arr();
    assert_eq!(arr2[1], 100);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn ArrDebug()
{
    let mut buff = Buff::new(3, 10);
    buff[1] = 20;
    buff[2] = 30;
    let arr = buff.as_arr();
    assert_eq!(format!("{:?}", arr), "[10, 20, 30]");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn USegBasicOps()
{
    let seg = USeg::new(10, 20);
    assert_eq!(seg.first(), 10);
    assert_eq!(seg.last(), 20);
    assert_eq!(seg.len(), 11);
    assert!(!seg.is_empty());

    let empty_seg = USeg::new(20, 10);
    assert_eq!(empty_seg.len(), 0);
    assert!(empty_seg.is_empty());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn USegSnip()
{
    let seg = USeg::new(10, 20);

    // Test LSnip
    let l_snipped = seg.LSnip(5);
    assert_eq!(l_snipped.first(), 15);
    assert_eq!(l_snipped.last(), 20);
    assert_eq!(l_snipped.len(), 6);

    let l_empty = seg.LSnip(11);
    assert!(l_empty.is_empty());

    let l_overflow = seg.LSnip(15);
    assert!(l_overflow.is_empty());

    // Test RSnip
    let r_snipped = seg.RSnip(4);
    assert_eq!(r_snipped.first(), 10);
    assert_eq!(r_snipped.last(), 16);
    assert_eq!(r_snipped.len(), 7);

    let r_empty = seg.RSnip(11);
    assert!(r_empty.is_empty());

    let r_underflow = seg.RSnip(20);
    assert!(r_underflow.is_empty());
}

//---------------------------------------------------------------------------------------------------------------------------------

