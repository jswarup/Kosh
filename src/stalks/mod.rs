//-- stalks/mod.rs --------------------------------------------------------------------------------------------------------------------
#[cfg( test)]
mod _tests;
pub mod atm;
pub mod work;
pub use	atm::{ Atm, Spinlock };
pub use	work::{ DynIWork, DynIWorker, IWork, IWorker, IntoWorkPtr, JobFn, WorkPtr, Worker };

//---------------------------------------------------------------------------------------------------------------------------------
