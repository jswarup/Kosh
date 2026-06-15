//-- silo/mod.rs ---------------------------------------------------------------------------------------------------------------------
#[cfg( test)]
mod _tests;
pub mod arr;
pub mod buff;
pub mod instream;
pub mod stash;
pub mod stk;
pub mod uint;
pub mod useg;
pub mod slice;
pub use	arr::Arr;
pub use	slice::{ISlice, IArr};
pub use	buff::Buff;
pub use	instream::InStream;
pub use	stash::Stash;
pub use	stk::Stk;
pub use	uint::{ U8, U16, U32, U64 };
pub use	useg::USeg;

//---------------------------------------------------------------------------------------------------------------------------------
