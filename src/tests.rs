//-- tests.rs ----------------------------------------------------------------------------------------------------------------------


//---------------------------------------------------------------------------------------------------------------------------------

use crate::buff::Buff;

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

#[test]
fn BufZST()
{
    let buff = Buff::new(10, ());
    assert_eq!(buff.len(), 10);
    assert_eq!(buff[5], ());
}

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
