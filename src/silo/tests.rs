//-- tests.rs ----------------------------------------------------------------------------------------------------------------------

use crate::silo::buff::Buff;
use crate::silo::useg::USeg;


//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BuffBasicOps()
{
    let mut buff = Buff::New(10, 42);
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
    let sliceData = [10, 20, 30];
    let buffFromSlice = Buff::from(&sliceData[..]);
    assert_eq!(buffFromSlice.len(), 3);
    assert_eq!(buffFromSlice[0], 10);
    assert_eq!(buffFromSlice[1], 20);
    assert_eq!(buffFromSlice[2], 30);

    // Test creation from a Vec
    let vecData = vec![40, 50];
    let buffFromVec = Buff::from(vecData);
    assert_eq!(buffFromVec.len(), 2);
    assert_eq!(buffFromVec[0], 40);
    assert_eq!(buffFromVec[1], 50);

    // Test creation from an array directly
    let buffFromArr = Buff::from([100, 200, 300, 400]);
    assert_eq!(buffFromArr.len(), 4);
    assert_eq!(buffFromArr[2], 300);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BufZST()
{
    let buff = Buff::New(10, ());
    assert_eq!(buff.len(), 10);
    assert_eq!(buff[5], ());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn BuffSendSync()
{
    let buff = Buff::New(5, 42);
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
    let mut buff = Buff::New(3, 42);
    {
        let mut arr = buff.AsMutArr();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], 42);
        arr[1] = 100;
    }
    assert_eq!(buff[1], 100);

    let arr2 = buff.AsArr();
    assert_eq!(arr2[1], 100);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn ArrDebug()
{
    let mut buff = Buff::New(3, 10);
    buff[1] = 20;
    buff[2] = 30;
    let arr = buff.AsArr();
    assert_eq!(format!("{:?}", arr), "[10, 20, 30]");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn USegBasicOps()
{
    let seg = USeg::New(10, 20);
    assert_eq!(seg.First(), 10);
    assert_eq!(seg.Last(), 20);
    assert_eq!(seg.Size(), 11);
    assert!(!seg.IsEmpty());

    let emptySeg = USeg::New(20, 10);
    assert_eq!(emptySeg.Size(), 0);
    assert!(emptySeg.IsEmpty());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn USegSnip()
{
    let seg = USeg::New(10, 20);

    // Test LSnip
    let lSnipped = seg.LSnip(5);
    assert_eq!(lSnipped.First(), 15);
    assert_eq!(lSnipped.Last(), 20);
    assert_eq!(lSnipped.Size(), 6);

    let lEmpty = seg.LSnip(11);
    assert!(lEmpty.IsEmpty());

    let lOverflow = seg.LSnip(15);
    assert!(lOverflow.IsEmpty());

    // Test RSnip
    let rSnipped = seg.RSnip(4);
    assert_eq!(rSnipped.First(), 10);
    assert_eq!(rSnipped.Last(), 16);
    assert_eq!(rSnipped.Size(), 7);

    let rEmpty = seg.RSnip(11);
    assert!(rEmpty.IsEmpty());

    let rUnderflow = seg.RSnip(20);
    assert!(rUnderflow.IsEmpty());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn USegSpan()
{
    let seg = USeg::New(10, 15);
    
    // Case 1: All values return true
    let mut visited = Vec::new();
    let result = seg.Span(|val| {
        visited.push(val);
        true
    });
    assert!(result);
    assert_eq!(visited, vec![10, 11, 12, 13, 14, 15]);

    // Case 2: One value returns false (early termination)
    let mut visited2 = Vec::new();
    let result2 = seg.Span(|val| {
        visited2.push(val);
        val < 13
    });
    assert!(!result2);
    assert_eq!(visited2, vec![10, 11, 12, 13]);

    // Case 3: Empty segment should vacuously return true
    let emptySeg = USeg::New(20, 10);
    let result3 = emptySeg.Span(|_| {
        panic!("Should not be called!");
    });
    assert!(result3);
}

//---------------------------------------------------------------------------------------------------------------------------------

