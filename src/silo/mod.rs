//-- silo/mod.rs ---------------------------------------------------------------------------------------------------------------------
#[cfg( test)]
mod _tests;
pub mod access;
pub mod arr;
pub mod buff;
pub mod stash;
pub mod stk;
pub mod uint;
pub mod useg;
pub mod cast;
pub mod edge_connect;
pub mod edge_broadcast;
pub use	edge_connect::{ EdgeConnect, IEdgeConnect };
pub use	edge_broadcast::{ EdgeBroadcast, IEdgeBroadcast };
pub use	access::{ IAccess, AccessIter};
pub use	arr::{Arr, IArr};
pub use	buff::Buff;
pub use	crate::Buff;
pub use	stash::{ StashMM, Stash };
pub use	crate::Stash;
pub use	stk::Stk;
pub use	uint::{ U8, U16, U32, U64 };
pub use	useg::USeg;
pub use cast::{ ICastExt, IPtrExt, IConstPtrExt, IPtrRefExt, IConstPtrRefExt, IPtrAtExt, IAllocRawExt, IVoidPtrExt, ISliceExt };

//---------------------------------------------------------------------------------------------------------------------------------
