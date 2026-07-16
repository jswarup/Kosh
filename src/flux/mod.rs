//-- flux/mod.rs ------------------------------------------------------------------------------------------------------------------------

pub mod instream;
pub mod jsonoutstrm;
pub mod outstream;
pub mod fluxexport;
pub mod fluxbasics;

pub use	instream::{IStream, FixedStream, BuffStream};
pub use	jsonoutstrm::JsonOutStream;
pub use	outstream::OutStream;

#[cfg( test)]
mod	_tests;
pub use	fluxexport::{ IFluxExportSink, FieldExp, IFluxExportSource };

//---------------------------------------------------------------------------------------------------------------------------------
pub mod fluximport;
pub use fluximport::{ IFluxImportSource, IFluxImportSink, FieldImp };
