//-- flux/mod.rs ------------------------------------------------------------------------------------------------------------------------

pub mod instream;
pub mod jsonoutstrm;
pub mod outstream;
pub mod fluxout;

pub use	instream::{IStream, FixedStream, BuffStream};
pub use	jsonoutstrm::JsonOutStream;
pub use	outstream::OutStream;

#[cfg( test)]
mod	_tests;
pub use	fluxout::{ IFluxOutSink, FieldOut, IFluxOutSource };

//---------------------------------------------------------------------------------------------------------------------------------
pub mod fluxin;

