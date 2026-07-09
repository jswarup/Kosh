//-- mod.rs -----------------------------------------------------------------------------------------------------------------------

pub mod shard;
pub mod parshard;
pub mod catshard;
pub mod repeatshard;
pub mod actionshard;
pub mod charset;
pub mod parser;
pub use charset::Charset;
pub use parser::{ Parser, IGrammar };
#[cfg( test)]
mod _tests;

//---------------------------------------------------------------------------------------------------------------------------------
