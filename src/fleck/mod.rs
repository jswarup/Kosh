//-- mod.rs ------------------------------------------------------------------------------------------------------------------------

pub mod	point;
pub mod	ptio;
pub mod	vex;
pub mod	waveobjio;

pub use	point::{ Dir3f, Pt3f, WPt2f, WPt3f };
pub use	ptio::{ ParsePts, ParsePtsBytes, ParsePtsStream, PtsCloud, PtsPoint, PtsShard };
pub use	vex::*;
pub use	waveobjio::{
    Face, FaceVertex, ParseWaveObj, ParseWaveObjBytes, ParseWaveObjStream,
    WaveObjModel, WaveObjShard,
};


#[cfg( test)]
mod	_tests;

//---------------------------------------------------------------------------------------------------------------------------------
