//-- mod.rs ------------------------------------------------------------------------------------------------------------------------

pub mod	point;
pub mod	ptio;
pub mod	waveobjio;

pub use	point::{ Dir3f, Pt3f, Pt4f };
pub use	ptio::{ ParsePts, ParsePtsBytes, ParsePtsStream, PtsCloud, PtsPoint, PtsShard };
pub use	waveobjio::{
    Face, FaceVertex, ParseWaveObj, ParseWaveObjBytes, ParseWaveObjStream,
    TexCoord, WaveObjModel, WaveObjShard,
};


#[cfg( test)]
mod	_tests;

//---------------------------------------------------------------------------------------------------------------------------------
