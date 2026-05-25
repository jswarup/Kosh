//-- silo/mod.rs ---------------------------------------------------------------------------------------------------------------------

pub mod uint32;
pub mod useg;
pub mod arr;
pub mod atm;
pub mod buff;
pub mod stk;

#[cfg(test)]
mod _tests;

pub struct Silo;

use crate::silo::buff::Buff;
use crate::silo::atm::Atm;
use crate::silo::stk::Stk;
use crate::silo::uint32::UInt32;

//---------------------------------------------------------------------------------------------------------------------------------

/// ExportImportOps demonstrates stack export/import operations.
#[allow(dead_code)]
pub fn ExportImportOps() {
    // Source stack with initial values 1..=5
    let mut src_buff = Buff::CreateD(10, |_| 0u32);
    let mut src_atm = Atm::New(0u32);
    let mut src_arr = src_buff.AsMutArr();
    let mut src_stack = Stk::Create(&mut src_atm, &mut src_arr);
    for i in 1..=5u32 {
        let mut val = i;
        src_stack.Push(&mut val);
    }
    assert_eq!(src_stack.Size(), 5);

    // Destination stack initially empty
    let mut dst_buff = Buff::CreateD(10, |_| 0u32);
    let mut dst_atm = Atm::New(0u32);
    let mut dst_arr = dst_buff.AsMutArr();
    let mut dst_stack = Stk::Create(&mut dst_atm, &mut dst_arr);
    assert_eq!(dst_stack.Size(), 0);

    // Export from source to destination (move all 5 elements)
    let moved = src_stack.Export(&mut dst_stack, 5);
    assert_eq!(moved, 5);
    assert_eq!(src_stack.Size(), 0);
    assert_eq!(dst_stack.Size(), 5);

    // Verify order in destination stack (LIFO 5..=1)
    for expected in (1..=5u32).rev() {
        let mut out = 0u32;
        dst_stack.Pop(&mut out);
        assert_eq!(out, expected);
    }
    assert_eq!(dst_stack.Size(), 0);

    // Refill source stack for Import test
    for i in 10..=14u32 {
        let mut v = i;
        src_stack.Push(&mut v);
    }
    assert_eq!(src_stack.Size(), 5);

    // Import from source into destination (move all 5 elements)
    let imported = dst_stack.Import(&mut src_stack, 5);
    assert_eq!(imported, 5);
    assert_eq!(dst_stack.Size(), 5);

    // Verify imported order (LIFO, should be 14..=10)
    for expected in (10..=14u32).rev() {
        let mut out = 0u32;
        dst_stack.Pop(&mut out);
        assert_eq!(out, expected);
    }
    assert_eq!(dst_stack.Size(), 0);
}

//---------------------------------------------------------------------------------------------------------------------------------
