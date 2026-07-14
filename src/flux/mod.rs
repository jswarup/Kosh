//-- flux/mod.rs ------------------------------------------------------------------------------------------------------------------------

pub mod instream;
pub mod jsonoutstrm;
pub mod outstream;
pub mod xflux;

pub use	instream::{IStream, FixedStream, BuffStream};
pub use	jsonoutstrm::JsonOutStream;
pub use	outstream::OutStream;

#[cfg( test)]
mod	_tests;
pub use	xflux::{ IXFluxSink, XField, IXFluxSource };

//---------------------------------------------------------------------------------------------------------------------------------
pub mod zflux;
