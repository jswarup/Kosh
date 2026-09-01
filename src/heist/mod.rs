//-- mod.rs -----------------------------------------------------------------------------------------------------------------------
pub mod atelier;
pub mod atelierinfo;
pub mod choretree;
pub mod corochore;
pub mod maestro;
pub use	atelier::{ Atelier, IAtelier };
pub use	atelierinfo::{ AtelierInfo, JobInfo };
pub use	choretree::{ Chore, IChore, ChoreTarget, IChoreNode, SpawnQuellNode };
pub use	corochore::{ CoroChore, ICoroChore, WorkerFatPtr };
pub use	maestro::{ Maestro, IMaestro };

#[cfg( test)]
mod _tests;

//---------------------------------------------------------------------------------------------------------------------------------
