//-- flux/mod.rs ------------------------------------------------------------------------------------------------------------------------

pub mod instream;
pub mod jsonoutstrm;
pub mod outstream;
pub mod xflux;

pub use	instream::InStream;
pub use	jsonoutstrm::JsonOutStream;
pub use	outstream::OutStream;

#[cfg( test)]
mod	_tests;
pub use	xflux::{ IXFluxSink, XField, IXFluxSource };

//---------------------------------------------------------------------------------------------------------------------------------
