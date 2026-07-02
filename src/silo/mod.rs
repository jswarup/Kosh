//-- silo/mod.rs ---------------------------------------------------------------------------------------------------------------------
#[cfg( test)]
mod _tests;
pub mod access;
pub mod arr;
pub mod buff;
pub mod stash;
pub mod stk;
pub mod uint;
pub mod compre;
pub mod useg;
pub mod cast;
pub use	access::{ IAccess, AccessIter};
pub use	arr::{Arr, IArr};
pub use	buff::Buff;
pub use	stash::Stash;
pub use	stk::Stk;
pub use	uint::{ U8, U16, U32, U64 };
pub use	useg::USeg;
pub use cast::{ ICastExt, IPtrExt, IConstPtrExt };

//---------------------------------------------------------------------------------------------------------------------------------
