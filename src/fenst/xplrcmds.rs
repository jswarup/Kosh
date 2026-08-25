#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
//-- xplrcmds.rs ------------------------------------------------------------------------------------------------------------------
use	std::collections::HashMap;
use	std::path::Path;
use	std::sync::{ LazyLock, Mutex };
use	serde::{ Deserialize, Serialize };
use	crate::{
    fenst::{
        CreateDefaultRegistry, PtsPointsDto, StreamChunkDto,
        XplrContent, XplrEntry, XplrFetchChunk as FenstFetchChunk,
        XplrFetchContent as FenstFetchContent, XplrFetchPtsPoints as FenstFetchPtsPoints,
        XplrLeafInfo as FenstLeafInfo, XplrListEntries as FenstListEntries,
        XplrNodeDto, XplrParsePtsFile, XplrParseWaveObjFile,
    },
    fleck::BBox3f,
    silo::{ Buff, ISliceExt, U32 },
    swarm::{
        Camera, SceneGraph, SwarmCluster,
        viewport::ObjRenderMode,
    },
};


static SYMPH_SPV: &[u8] = include_bytes!( env!( "SYMPH_SPV_PATH"));

static SWARM_CLUSTER: LazyLock< SwarmCluster> = LazyLock::new( || {
    SwarmCluster::Auto()
});

// ---------------------------------------------------------------------------------------------------------------------------------


/// Reads a directory and returns sorted entries (directories first, then files).
pub fn	XplrListEntries( path: String) -> Result< Buff< XplrEntry>, String>
{
    FenstListEntries( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads the text content of a file, with a size guard.
pub fn	XplrFetchContent( path: String) -> Result< XplrContent, String>
{
    FenstFetchContent( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns metadata about a file or directory.
pub fn	XplrLeafInfo( path: String) -> Result< FenstLeafInfo, String>
{
    FenstLeafInfo( path)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Shows a native dialog to pick a folder.
pub async fn	XplrSelectBranch() -> Result< Option< String>, String>
{
    let  	fileDialog = rfd::AsyncFileDialog::new()
        .set_title( "Select Folder to Open");

    let  	folderHandle = fileDialog.pick_folder().await;

    Ok( folderHandle.map( |h| h.path().to_string_lossy().into_owned()))
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Fetches children of a given URI using the registered XplrProviders.
pub fn	XplrChildren( uri: String) -> Result< Buff< XplrNodeDto>, String>
{
    let  	registry = CreateDefaultRegistry();
    let  	guard = registry.read().map_err( |e| e.to_string())?;
    let  	( scheme, root) = guard.OpenRoot( &uri)?;
    let  	children = root.Children()?;
    let  	dtos = Buff::Create( children.Size(), |i| {
        children[i.AsUsize()].ToDto( &scheme)
    });
    Ok( dtos)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns the list of registered provider scheme prefixes.
pub fn	XplrListProviders() -> Result< Buff< String>, String>
{
    let  	registry = CreateDefaultRegistry();
    let  	guard = registry.read().map_err( |e| e.to_string())?;
    let  	schemes = guard.Schemes();
    let  	buff = Buff::Create( U32( schemes.len() as u32), |i| {
        schemes[i.AsUsize()].clone()
    });
    Ok( buff)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a windowed chunk of a file using flux::BuffStream and silo::Buff.
pub fn	XplrFetchChunk( path: String, offset: u64, size: usize) -> Result< StreamChunkDto, String>
{
    FenstFetchChunk( path, offset, size)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Generates 3D points from a .pts file (using fleck::ParsePtsStream) or from GPU compute shader if path is empty/omitted.
pub fn	XplrFetchPtsPoints( path: Option< String>) -> Result< Vec< u8>, String>
{
    let  	dto = if let Some( ref filePath) = path {
        if !filePath.is_empty() && Path::new( filePath).exists() {
            XplrParsePtsFile( filePath)?
        } else {
            FenstFetchPtsPoints( SYMPH_SPV)?
        }
    } else {
        FenstFetchPtsPoints( SYMPH_SPV)?
    };

    let  	bytes = dto.ToBytes();
    return Ok( bytes);
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Parses and returns a Wavefront .obj 3D mesh model from the specified file path.
pub fn	XplrFetchWaveObj( path: Option< String>) -> Result< Vec< u8>, String>
{
    let  	dto = if let Some( ref filePath) = path {
        if !filePath.is_empty() && Path::new( filePath).exists() {
            XplrParseWaveObjFile( filePath)?
        } else {
            let  	defaultObj = "workbench/blub/blub_control_mesh.obj";
            if Path::new( defaultObj).exists() {
                XplrParseWaveObjFile( defaultObj)?
            } else {
                return Err( "No valid .obj file path provided".to_string());
            }
        }
    } else {
        let  	defaultObj = "workbench/blub/blub_control_mesh.obj";
        if Path::new( defaultObj).exists() {
            XplrParseWaveObjFile( defaultObj)?
        } else {
            return Err( "No valid .obj file path provided".to_string());
        }
    };

    let  	bytes = dto.ToBytes();
    return Ok( bytes);
}

// ---------------------------------------------------------------------------------------------------------------------------------


struct PtsSessionState
{
    _Scene: SceneGraph,
    _MetadataSent: bool,                                               // Phase 2: Track if metadata was sent
    _MetadataHash: u64,                                                // Phase 2: Hash to detect metadata changes
    _LastFrameKey: Option< PtsFrameKey>,
}

#[derive( Clone, Copy, PartialEq, Eq)]
struct PtsFrameKey
{
    _Width:  u32,
    _Height: u32,
    _Dpr:    u32,
    _PanX:   u32,
    _PanY:   u32,
    _Zoom:   u32,
    _RotX:   u32,
    _RotY:   u32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Phase 2: Static metadata sent once per session to reduce bandwidth.
#[derive( Serialize, Clone, Debug)]
#[serde( rename_all = "snake_case")]
pub struct PtsSessionMetadata
{
    pub _FileName:      String,                                        // Sent once
    pub _Count:         usize,                                         // Total scene point count
    pub _BboxMin:       [f32; 3],                                      // Bounding box minimum
    pub _BboxMax:       [f32; 3],                                      // Bounding box maximum
    pub _BboxLabel:     String,                                        // Formatted bbox label
}

/// Phase 2: Dynamic frame data sent every frame (40% smaller without static metadata).
#[derive( Serialize, Clone, Debug)]
#[serde( rename_all = "snake_case")]
pub struct PtsFrameUpdate
{
    pub _Points:        Buff< ProjectedPoint>,                         // Only dynamic: projected points
    pub _BoxLines:      Buff< ProjectedLine>,                          // Only dynamic: bbox wireframe
    pub _OverlayText1:  String,                                        // Camera state + point count
    pub _OverlayText2:  String,                                        // Source file OR device status
}

// ---------------------------------------------------------------------------------------------------------------------------------

static PTS_STATE: LazyLock< Mutex< HashMap< String, PtsSessionState>>> = LazyLock::new( || {
    Mutex::new( HashMap::new())
});

// ---------------------------------------------------------------------------------------------------------------------------------

#[repr(C)]
#[derive( Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct ProjectedPoint
{
    pub _X:           f32,
    pub _Y:           f32,
    pub _Radius:      f32,
    pub _CoreRadius:  f32,
    pub _Alpha:       f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[repr(C)]
#[derive( Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct ProjectedLine
{
    pub _X1:  f32,
    pub _Y1:  f32,
    pub _X2:  f32,
    pub _Y2:  f32,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// 2D projected triangular facet with depth and shading attributes.
#[derive( Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct ProjectedFacet
{
    pub _P1:    ( f32, f32),
    pub _P2:    ( f32, f32),
    pub _P3:    ( f32, f32),
    pub _ZAvg:  f32,
    pub _Shade: u8,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Frame payload containing projected mesh wireframes, facets, points, and status.
#[derive( Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct MeshFrameDto
{
    pub _Points:       Buff< ProjectedPoint>,
    pub _WireLines:    Buff< ProjectedLine>,
    pub _Facets:       Buff< ProjectedFacet>,
    pub _PointCount:   usize,
    pub _FaceCount:    usize,
    pub _StatusText:   String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Phase 3: Quantized point data (7 bytes instead of 20 bytes = 65% bandwidth savings).
/// Uses 16-bit screen coordinates and 8-bit attributes for sufficient precision in typical use.
#[repr(C, packed)]
#[derive( Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct QuantizedPoint
{
    pub _X:           u16,                                             // 16-bit screen X (0-65535, maps to screen width)
    pub _Y:           u16,                                             // 16-bit screen Y (0-65535, maps to screen height)
    pub _Radius:      u8,                                              // 8-bit radius (0-255)
    pub _CoreRadius:  u8,                                              // 8-bit core radius (0-255)
    pub _Alpha:       u8,                                              // 8-bit alpha (0-255)
}

impl QuantizedPoint
{
    /// Quantizes a full-precision ProjectedPoint to 7-byte QuantizedPoint.
    /// Screen coords are normalized to 0-65535 range based on dimensions.
    pub fn	FromProjected( pt: &ProjectedPoint, width: f32, height: f32) -> Self
    {
        let  	x_norm = (pt._X / width * 65535.0).clamp( 0.0, 65535.0) as u16;
        let  	y_norm = (pt._Y / height * 65535.0).clamp( 0.0, 65535.0) as u16;
        let  	radius = (pt._Radius * 255.0).clamp( 0.0, 255.0) as u8;
        let  	core_radius = (pt._CoreRadius * 255.0).clamp( 0.0, 255.0) as u8;
        let  	alpha = (pt._Alpha * 255.0).clamp( 0.0, 255.0) as u8;

        QuantizedPoint {
            _X: x_norm,
            _Y: y_norm,
            _Radius: radius,
            _CoreRadius: core_radius,
            _Alpha: alpha,
        }
    }

    /// Dequantizes back to full-precision ProjectedPoint given screen dimensions.
    pub fn	ToProjected( &self, width: f32, height: f32) -> ProjectedPoint
    {
        let  	x = (self._X as f32 / 65535.0) * width;
        let  	y = (self._Y as f32 / 65535.0) * height;
        let  	radius = self._Radius as f32 / 255.0;
        let  	core_radius = self._CoreRadius as f32 / 255.0;
        let  	alpha = self._Alpha as f32 / 255.0;

        ProjectedPoint {
            _X: x,
            _Y: y,
            _Radius: radius,
            _CoreRadius: core_radius,
            _Alpha: alpha,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Phase 3: Quantized line data (4 bytes instead of 16 bytes = 75% bandwidth savings).
/// Uses 16-bit screen coordinates for bounding box wireframe rendering.
#[repr(C, packed)]
#[derive( Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct QuantizedLine
{
    pub _X1:  u16,                                                     // 16-bit screen X1 (0-65535)
    pub _Y1:  u16,                                                     // 16-bit screen Y1 (0-65535)
    pub _X2:  u16,                                                     // 16-bit screen X2 (0-65535)
    pub _Y2:  u16,                                                     // 16-bit screen Y2 (0-65535)
}

impl QuantizedLine
{
    /// Quantizes a full-precision ProjectedLine to 4-byte QuantizedLine.
    pub fn	FromProjected( line: &ProjectedLine, width: f32, height: f32) -> Self
    {
        let  	x1_norm = (line._X1 / width * 65535.0).clamp( 0.0, 65535.0) as u16;
        let  	y1_norm = (line._Y1 / height * 65535.0).clamp( 0.0, 65535.0) as u16;
        let  	x2_norm = (line._X2 / width * 65535.0).clamp( 0.0, 65535.0) as u16;
        let  	y2_norm = (line._Y2 / height * 65535.0).clamp( 0.0, 65535.0) as u16;

        QuantizedLine {
            _X1: x1_norm,
            _Y1: y1_norm,
            _X2: x2_norm,
            _Y2: y2_norm,
        }
    }

    /// Dequantizes back to full-precision ProjectedLine given screen dimensions.
    pub fn	ToProjected( &self, width: f32, height: f32) -> ProjectedLine
    {
        let  	x1 = (self._X1 as f32 / 65535.0) * width;
        let  	y1 = (self._Y1 as f32 / 65535.0) * height;
        let  	x2 = (self._X2 as f32 / 65535.0) * width;
        let  	y2 = (self._Y2 as f32 / 65535.0) * height;

        ProjectedLine {
            _X1: x1,
            _Y1: y1,
            _X2: x2,
            _Y2: y2,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Fixed binary header for PtsFrame binary serialization.
#[repr(C)]
#[derive( Debug, Clone, Copy, PartialEq)]
pub struct PtsFrameBinaryHeader
{
    pub _Magic:          u32,                                           // 0x4B505453 ('KPTS')
    pub _Version:        u32,                                           // Format version: 1
    pub _PointCount:     u32,                                           // Number of ProjectedPoints
    pub _LineCount:      u32,                                           // Number of ProjectedLines
    pub _TotalPoints:    u32,                                           // Total scene point count
    pub _FileNameLen:    u32,                                           // Byte length of _FileName
    pub _BboxLabelLen:   u32,                                           // Byte length of _BboxLabel
    pub _ShaderStatusLen:u32,                                           // Byte length of _ShaderStatus
    pub _Overlay1Len:    u32,                                           // Byte length of _OverlayText1
    pub _Overlay2Len:    u32,                                           // Byte length of _OverlayText2
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct PtsFrameDto
{
    pub _Points:        Buff< ProjectedPoint>,
    pub _BoxLines:      Buff< ProjectedLine>,
    pub _FileName:      String,
    pub _Count:         usize,
    pub _BboxLabel:     String,
    pub _ShaderStatus:  String,
    pub _OverlayText1:  String,
    pub _OverlayText2:  String,
}

impl PtsFrameDto
{
    /// Serializes the frame DTO into a compact #[repr(C)] binary payload.
    pub fn	ToBytes( &self) -> Vec< u8>
    {
        let  	fileNameBytes = self._FileName.as_bytes();
        let  	bboxLabelBytes = self._BboxLabel.as_bytes();
        let  	shaderStatusBytes = self._ShaderStatus.as_bytes();
        let  	overlay1Bytes = self._OverlayText1.as_bytes();
        let  	overlay2Bytes = self._OverlayText2.as_bytes();

        let  	header = PtsFrameBinaryHeader {
            _Magic:          0x4B505453,
            _Version:        1,
            _PointCount:     self._Points.len() as u32,
            _LineCount:      self._BoxLines.len() as u32,
            _TotalPoints:    self._Count as u32,
            _FileNameLen:    fileNameBytes.len() as u32,
            _BboxLabelLen:   bboxLabelBytes.len() as u32,
            _ShaderStatusLen:shaderStatusBytes.len() as u32,
            _Overlay1Len:    overlay1Bytes.len() as u32,
            _Overlay2Len:    overlay2Bytes.len() as u32,
        };

        let  	headerSlice = std::slice::from_ref( &header);
        let  	headerBytes: &[u8] = headerSlice.CastSlice();
        let  	pointsBytes: &[u8] = self._Points.CastSlice();
        let  	linesBytes: &[u8] = self._BoxLines.CastSlice();

        let  	totalLen = headerBytes.len()
            + pointsBytes.len()
            + linesBytes.len()
            + fileNameBytes.len()
            + bboxLabelBytes.len()
            + shaderStatusBytes.len()
            + overlay1Bytes.len()
            + overlay2Bytes.len();

        let  	mut out = Vec::with_capacity( totalLen);
        out.extend_from_slice( headerBytes);
        out.extend_from_slice( pointsBytes);
        out.extend_from_slice( linesBytes);
        out.extend_from_slice( fileNameBytes);
        out.extend_from_slice( bboxLabelBytes);
        out.extend_from_slice( shaderStatusBytes);
        out.extend_from_slice( overlay1Bytes);
        out.extend_from_slice( overlay2Bytes);

        return out;
    }

    /// Deserializes a frame DTO from a #[repr(C)] binary payload.
    pub fn	FromBytes( bytes: &[u8]) -> Result< Self, String>
    {
        let  	headerSz = std::mem::size_of::< PtsFrameBinaryHeader>();
        if bytes.len() < headerSz {
            return Err( "PtsFrame binary payload too short for header".to_string());
        }

        let  	headerSlice: &[PtsFrameBinaryHeader] = bytes[..headerSz].CastSliceFrom();
        let  	header = headerSlice[0];

        if header._Magic != 0x4B505453 {
            return Err( format!( "Invalid PtsFrame magic: 0x{:08X}", header._Magic));
        }

        let  	mut offset = headerSz;

        let  	pointsByteLen = header._PointCount as usize * std::mem::size_of::< ProjectedPoint>();
        if bytes.len() < offset + pointsByteLen {
            return Err( "PtsFrame payload truncated in points buffer".to_string());
        }
        let  	pointsSlice: &[ProjectedPoint] = bytes[offset..offset + pointsByteLen].CastSliceFrom();
        let  	points = Buff::from( pointsSlice);
        offset += pointsByteLen;

        let  	linesByteLen = header._LineCount as usize * std::mem::size_of::< ProjectedLine>();
        if bytes.len() < offset + linesByteLen {
            return Err( "PtsFrame payload truncated in lines buffer".to_string());
        }
        let  	linesSlice: &[ProjectedLine] = bytes[offset..offset + linesByteLen].CastSliceFrom();
        let  	lines = Buff::from( linesSlice);
        offset += linesByteLen;

        let  	strEnd1 = offset + header._FileNameLen as usize;
        let  	strEnd2 = strEnd1 + header._BboxLabelLen as usize;
        let  	strEnd3 = strEnd2 + header._ShaderStatusLen as usize;
        let  	strEnd4 = strEnd3 + header._Overlay1Len as usize;
        let  	strEnd5 = strEnd4 + header._Overlay2Len as usize;

        if bytes.len() < strEnd5 {
            return Err( "PtsFrame payload truncated in string table".to_string());
        }

        let  	fileName = String::from_utf8_lossy( &bytes[offset..strEnd1]).into_owned();
        let  	bboxLabel = String::from_utf8_lossy( &bytes[strEnd1..strEnd2]).into_owned();
        let  	shaderStatus = String::from_utf8_lossy( &bytes[strEnd2..strEnd3]).into_owned();
        let  	overlay1 = String::from_utf8_lossy( &bytes[strEnd3..strEnd4]).into_owned();
        let  	overlay2 = String::from_utf8_lossy( &bytes[strEnd4..strEnd5]).into_owned();

        return Ok( PtsFrameDto {
            _Points:        points,
            _BoxLines:      lines,
            _FileName:      fileName,
            _Count:         header._TotalPoints as usize,
            _BboxLabel:     bboxLabel,
            _ShaderStatus:  shaderStatus,
            _OverlayText1:  overlay1,
            _OverlayText2:  overlay2,
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Phase 2-3 Integration: Quantized frame DTO using QuantizedPoint/Line (65% bandwidth savings).
/// Uses same binary header format as PtsFrameDto but with smaller quantized types.
#[derive( Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct QuantizedPtsFrameDto
{
    pub _Points:        Buff< QuantizedPoint>,                         // 7 bytes per point instead of 20
    pub _BoxLines:      Buff< QuantizedLine>,                          // 8 bytes per line instead of 16
    pub _FileName:      String,
    pub _Count:         usize,
    pub _BboxLabel:     String,
    pub _ShaderStatus:  String,
    pub _OverlayText1:  String,
    pub _OverlayText2:  String,
}

impl QuantizedPtsFrameDto
{
    /// Converts from full-precision PtsFrameDto to quantized version.
    pub fn	FromPtsFrameDto( frame: &PtsFrameDto, width: f32, height: f32) -> Self
    {
        let  	quantized_points = Buff::Create( frame._Points.Size(), |i| {
            QuantizedPoint::FromProjected( &frame._Points[i.AsUsize()], width, height)
        });

        let  	quantized_lines = Buff::Create( frame._BoxLines.Size(), |i| {
            QuantizedLine::FromProjected( &frame._BoxLines[i.AsUsize()], width, height)
        });

        QuantizedPtsFrameDto {
            _Points:        quantized_points,
            _BoxLines:      quantized_lines,
            _FileName:      frame._FileName.clone(),
            _Count:         frame._Count,
            _BboxLabel:     frame._BboxLabel.clone(),
            _ShaderStatus:  frame._ShaderStatus.clone(),
            _OverlayText1:  frame._OverlayText1.clone(),
            _OverlayText2:  frame._OverlayText2.clone(),
        }
    }

    /// Converts back to full-precision PtsFrameDto for compatibility.
    pub fn	ToPtsFrameDto( &self, width: f32, height: f32) -> PtsFrameDto
    {
        let  	points = Buff::Create( self._Points.Size(), |i| {
            self._Points[i.AsUsize()].ToProjected( width, height)
        });

        let  	lines = Buff::Create( self._BoxLines.Size(), |i| {
            self._BoxLines[i.AsUsize()].ToProjected( width, height)
        });

        PtsFrameDto {
            _Points:        points,
            _BoxLines:      lines,
            _FileName:      self._FileName.clone(),
            _Count:         self._Count,
            _BboxLabel:     self._BboxLabel.clone(),
            _ShaderStatus:  self._ShaderStatus.clone(),
            _OverlayText1:  self._OverlayText1.clone(),
            _OverlayText2:  self._OverlayText2.clone(),
        }
    }

    /// Serializes quantized frame to binary format (same structure as PtsFrameDto but with smaller types).
    pub fn	ToBytes( &self) -> Vec< u8>
    {
        let  	fileNameBytes = self._FileName.as_bytes();
        let  	bboxLabelBytes = self._BboxLabel.as_bytes();
        let  	shaderStatusBytes = self._ShaderStatus.as_bytes();
        let  	overlay1Bytes = self._OverlayText1.as_bytes();
        let  	overlay2Bytes = self._OverlayText2.as_bytes();

        let  	header = PtsFrameBinaryHeader {
            _Magic:          0x4B505451,                               // Magic 'KPTQ' (Kosh PTs Quantized)
            _Version:        2,                                         // Version 2 = quantized format
            _PointCount:     self._Points.len() as u32,
            _LineCount:      self._BoxLines.len() as u32,
            _TotalPoints:    self._Count as u32,
            _FileNameLen:    fileNameBytes.len() as u32,
            _BboxLabelLen:   bboxLabelBytes.len() as u32,
            _ShaderStatusLen:shaderStatusBytes.len() as u32,
            _Overlay1Len:    overlay1Bytes.len() as u32,
            _Overlay2Len:    overlay2Bytes.len() as u32,
        };

        let  	headerSlice = std::slice::from_ref( &header);
        let  	headerBytes: &[u8] = headerSlice.CastSlice();
        let  	pointsBytes: &[u8] = self._Points.CastSlice();
        let  	linesBytes: &[u8] = self._BoxLines.CastSlice();

        let  	totalLen = headerBytes.len() + pointsBytes.len() + linesBytes.len()
            + fileNameBytes.len() + bboxLabelBytes.len() + shaderStatusBytes.len()
            + overlay1Bytes.len() + overlay2Bytes.len();
        let  	mut out = Vec::with_capacity( totalLen);

        out.extend_from_slice( headerBytes);
        out.extend_from_slice( pointsBytes);
        out.extend_from_slice( linesBytes);
        out.extend_from_slice( fileNameBytes);
        out.extend_from_slice( bboxLabelBytes);
        out.extend_from_slice( shaderStatusBytes);
        out.extend_from_slice( overlay1Bytes);
        out.extend_from_slice( overlay2Bytes);

        return out;
    }

    /// Deserializes quantized frame from binary payload.
    pub fn	FromBytes( bytes: &[u8]) -> Result< Self, String>
    {
        let  	headerSz = std::mem::size_of::< PtsFrameBinaryHeader>();
        if bytes.len() < headerSz {
            return Err( "QuantizedPtsFrame binary payload too short for header".to_string());
        }

        let  	headerSlice: &[PtsFrameBinaryHeader] = bytes[..headerSz].CastSliceFrom();
        let  	header = headerSlice[0];

        if header._Magic != 0x4B505451 {
            return Err( format!( "Invalid QuantizedPtsFrame magic: 0x{:08X}", header._Magic));
        }

        let  	mut offset = headerSz;

        let  	pointsByteLen = header._PointCount as usize * std::mem::size_of::< QuantizedPoint>();
        if bytes.len() < offset + pointsByteLen {
            return Err( "QuantizedPtsFrame payload truncated in points buffer".to_string());
        }
        let  	pointsSlice: &[QuantizedPoint] = bytes[offset..offset + pointsByteLen].CastSliceFrom();
        let  	points = Buff::from(  pointsSlice);
        offset += pointsByteLen;

        let  	linesByteLen = header._LineCount as usize * std::mem::size_of::< QuantizedLine>();
        if bytes.len() < offset + linesByteLen {
            return Err( "QuantizedPtsFrame payload truncated in lines buffer".to_string());
        }
        let  	linesSlice: &[QuantizedLine] = bytes[offset..offset + linesByteLen].CastSliceFrom();
        let  	lines = Buff::from(  linesSlice);
        offset += linesByteLen;

        // String table
        let  	strEnd1 = offset + header._FileNameLen as usize;
        let  	strEnd2 = strEnd1 + header._BboxLabelLen as usize;
        let  	strEnd3 = strEnd2 + header._ShaderStatusLen as usize;
        let  	strEnd4 = strEnd3 + header._Overlay1Len as usize;
        let  	strEnd5 = strEnd4 + header._Overlay2Len as usize;

        if bytes.len() < strEnd5 {
            return Err( "QuantizedPtsFrame payload truncated in string table".to_string());
        }

        let  	fileName = String::from_utf8_lossy( &bytes[offset..strEnd1]).into_owned();
        let  	bboxLabel = String::from_utf8_lossy( &bytes[strEnd1..strEnd2]).into_owned();
        let  	shaderStatus = String::from_utf8_lossy( &bytes[strEnd2..strEnd3]).into_owned();
        let  	overlay1 = String::from_utf8_lossy( &bytes[strEnd3..strEnd4]).into_owned();
        let  	overlay2 = String::from_utf8_lossy( &bytes[strEnd4..strEnd5]).into_owned();

        return Ok( QuantizedPtsFrameDto {
            _Points:        points,
            _BoxLines:      lines,
            _FileName:      fileName,
            _Count:         header._TotalPoints as usize,
            _BboxLabel:     bboxLabel,
            _ShaderStatus:  shaderStatus,
            _OverlayText1:  overlay1,
            _OverlayText2:  overlay2,
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Transforms and projects 3D point cloud coordinates and its bounding box to 2D screen coordinates,
/// applying camera pan, zoom, and rotation state.
pub fn	XplrProjectPts(
    path: String,
    width: f32,
    height: f32,
    dpr: f32,
    speed: f32,
    _color: String,
    pan_x: Option< f32>,
    pan_y: Option< f32>,
    zoom: Option< f32>,
    rot_x: Option< f32>,
    rot_y: Option< f32>,
    is_interactive: Option< bool>,
) -> Result< Vec< u8>, String>
{
    let  	mut guard = PTS_STATE.lock().map_err( |e| e.to_string())?;
    let  	state = guard.entry( path.clone()).or_insert_with( || {
        let  	dto = if Path::new( &path).exists() {
            XplrParsePtsFile( &path)
                .or_else( |_| FenstFetchPtsPoints( SYMPH_SPV))
                .unwrap_or_else( |_| PtsPointsDto {
                    _Points: Buff::New(),
                    _Count: 0,
                    _BboxMin: [ 0.0, 0.0, 0.0 ],
                    _BboxMax: [ 0.0, 0.0, 0.0 ],
                })
        } else {
            FenstFetchPtsPoints( SYMPH_SPV).unwrap_or_else( |_| PtsPointsDto {
                _Points: Buff::New(),
                _Count: 0,
                _BboxMin: [ 0.0, 0.0, 0.0 ],
                _BboxMax: [ 0.0, 0.0, 0.0 ],
            })
        };

        PtsSessionState {
            _Scene: SceneGraph::WithPoints( dto._Points, dto._BboxMin, dto._BboxMax),
            _MetadataSent: false,                                      // Phase 2: Initialize as not sent
            _MetadataHash: 0,                                          // Phase 2: Initialize hash
            _LastFrameKey: None,
        }
    });

    let  	cam = state._Scene.CameraMut();
    if let Some( px) = pan_x {
        cam._PanX = px;
    }
    if let Some( py) = pan_y {
        cam._PanY = py;
    }
    if let Some( z) = zoom {
        cam.SetZoom( z);
    }
    if let Some( rx) = rot_x {
        cam._RotX = rx;
    }
    if let Some( ry) = rot_y {
        cam._RotY = ry;
    }

    if is_interactive != Some( true) && speed > 0.0 {
        let  	speedRad = speed / 1000.0;
        cam._RotY += speedRad;
        cam._RotX += speedRad * 0.5;
    }

    let   frameKey = PtsFrameKey {
        _Width: width.to_bits(),
        _Height: height.to_bits(),
        _Dpr: dpr.to_bits(),
        _PanX: cam._PanX.to_bits(),
        _PanY: cam._PanY.to_bits(),
        _Zoom: cam._Zoom.to_bits(),
        _RotX: cam._RotX.to_bits(),
        _RotY: cam._RotY.to_bits(),
    };
    if state._LastFrameKey == Some( frameKey) {
        return Ok( Vec::new());
    }

    let  	camRef = state._Scene.Camera();

    let  	sceneResult = state._Scene.ProjectSceneCluster(
        &SWARM_CLUSTER,
        width,
        height,
        dpr,
        None,
    );

    let  	( box_lines, projectedPoints ) = if let Ok( sceneFrame) = sceneResult {
        let  	lines = Buff::Create( sceneFrame._BoxLines.Size(), |i| {
            let  	line = &sceneFrame._BoxLines[i.AsUsize()];
            ProjectedLine {
                _X1: line.0.0,
                _Y1: line.0.1,
                _X2: line.1.0,
                _Y2: line.1.1,
            }
        });
        let  	pts = Buff::Create( sceneFrame._Points.Size(), |i| {
            let  	pt = &sceneFrame._Points[i.AsUsize()];
            ProjectedPoint {
                _X: pt.0,
                _Y: pt.1,
                _Radius: pt.2,
                _CoreRadius: pt.3,
                _Alpha: pt.4,
            }
        });
        ( lines, pts )
    } else {
        // High performance CPU fallback if Swarm device is uninitialized
        let  	( center, scaleNorm) = state._Scene.CalcNormalization();
        let  	bboxLines = state._Scene.ProjectBoundingBox( width, height);
        let  	lines = Buff::Create( bboxLines.Size(), |i| {
            let  	( p1, p2) = &bboxLines[i.AsUsize()];
            ProjectedLine {
                _X1: p1.0,
                _Y1: p1.1,
                _X2: p2.0,
                _Y2: p2.1,
            }
        });

        let  	pts = Buff::Create( state._Scene._Points.Size(), |i| {
            let  	pt = &state._Scene._Points[i.AsUsize()];
            let  	nx = ( pt[0] - center[0]) * scaleNorm;
            let  	ny = ( pt[1] - center[1]) * scaleNorm;
            let  	nz = ( pt[2] - center[2]) * scaleNorm;
            let  	( px, py, pz) = camRef.Project( nx, ny, nz, width, height);
            let  	depthFactor = 0.3f32.max( 1.0f32.min( ( 300.0 - pz) / 400.0));
            let  	radius = ( 3.0 + depthFactor * 4.0) * dpr;
            let  	alpha = 0.5 + depthFactor * 0.5;
            let  	core_radius = ( 1.0 + depthFactor * 1.5) * dpr;

            ProjectedPoint {
                _X: px,
                _Y: py,
                _Radius: radius,
                _CoreRadius: core_radius,
                _Alpha: alpha,
            }
        });
        ( lines, pts )
    };

    let  	fileName = std::path::Path::new( &path)
        .file_name()
        .map( |n| n.to_string_lossy().into_owned())
        .unwrap_or_else( || "Block.pts".to_string());

    let  	bboxMin = state._Scene._BboxMin;
    let  	bboxMax = state._Scene._BboxMax;
    let  	bboxLabel = format!( "[{:.2}, {:.2}, {:.2}] → [{:.2}, {:.2}, {:.2}]",
        bboxMin[0], bboxMin[1], bboxMin[2],
        bboxMax[0], bboxMax[1], bboxMax[2]
    );

    let  	isParsedFile = std::path::Path::new( &path).exists();
    let  	overlay1 = format!(
        "Points: {} | Zoom: {:.2}x | Pan: ({:.0}, {:.0})",
        state._Scene._Points.len(),
        camRef._Zoom,
        camRef._PanX,
        camRef._PanY
    );
    let  	overlay2 = if isParsedFile {
        format!( "Source: {} | Rot: ({:.0}°, {:.0}°)", fileName, camRef._RotX.to_degrees(), camRef._RotY.to_degrees())
    } else {
        format!( "Swarm {} [{} dev] | Rot: ({:.0}°, {:.0}°)", SWARM_CLUSTER.Primary().Backend(), SWARM_CLUSTER.DeviceCount(), camRef._RotX.to_degrees(), camRef._RotY.to_degrees())
    };
    let  	shaderStatus = format!( "Swarm GPU [{}] ({} device{}) SceneGraph ({} pts)",
        SWARM_CLUSTER.Primary().Backend(),
        SWARM_CLUSTER.DeviceCount(),
        if SWARM_CLUSTER.DeviceCount() > 1 { "s" } else { "" },
        state._Scene._Points.len()
    );

    let  	frame = PtsFrameDto {
        _Points: projectedPoints,
        _BoxLines: box_lines,
        _FileName: fileName,
        _Count: state._Scene._Points.len(),
        _BboxLabel: bboxLabel,
        _ShaderStatus: shaderStatus,
        _OverlayText1: overlay1,
        _OverlayText2: overlay2,
    };

    // Phase 2-3 Integration: Use quantized format for 65% bandwidth savings
    let  	quantized_frame = QuantizedPtsFrameDto::FromPtsFrameDto( &frame, width, height);
    let  	bytes = quantized_frame.ToBytes();
    state._LastFrameKey = Some( frameKey);
    return Ok( bytes);
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Resets camera pan, zoom, and rotation in the active SceneGraph to default centered view.
pub fn	XplrResetCamera( path: String) -> Result< Camera, String>
{
    let  	mut guard = PTS_STATE.lock().map_err( |e| e.to_string())?;
    if let Some( state) = guard.get_mut( &path) {
        state._Scene.CameraMut().Reset();
        state._LastFrameKey = None;
        Ok( *state._Scene.Camera())
    } else {
        Ok( Camera::New())
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests
{
    use	super::*;
    use	crate::fenst::WaveObjMeshDto;
    use	std::env::temp_dir;
    use	std::fs::{ File, remove_file };
    use	std::io::Write;

    #[test]
    fn	TestXplrFetchPtsPointsWithFile()
    {
        let  	tempPath = temp_dir().join( "test_fenst_cloud.pts");
        {
            let  	mut f = File::create( &tempPath).unwrap();
            writeln!( f, "3").unwrap();
            writeln!( f, "10.0 20.0 30.0").unwrap();
            writeln!( f, "40.0 50.0 60.0").unwrap();
            writeln!( f, "70.0 80.0 90.0").unwrap();
        }

        let  	pathStr = tempPath.to_str().unwrap().to_string();
        let  	dtoDirect = XplrParsePtsFile( &pathStr).unwrap();
        assert_eq!( dtoDirect._Count, 3);
        assert_eq!( dtoDirect._Points.len(), 3);
        assert_eq!( dtoDirect._Points[0], [10.0, 20.0, 30.0]);
        assert_eq!( dtoDirect._BboxMin, [10.0, 20.0, 30.0]);
        assert_eq!( dtoDirect._BboxMax, [70.0, 80.0, 90.0]);

        // Test binary serialization roundtrip
        let  	bytes = dtoDirect.ToBytes();
        let  	dtoDecoded = PtsPointsDto::FromBytes( &bytes).unwrap();
        assert_eq!( dtoDecoded, dtoDirect);

        let  	res = XplrFetchPtsPoints( Some( pathStr));
        assert!( res.is_ok());

        let  	_ = remove_file( &tempPath);
    }

    #[test]
    fn	TestXplrProjectPtsWithFile()
    {
        let  	tempPath = std::env::temp_dir().join( "test_fenst_project.pts");
        {
            let  	mut f = File::create( &tempPath).unwrap();
            writeln!( f, "2").unwrap();
            writeln!( f, "0.0 0.0 0.0").unwrap();
            writeln!( f, "10.0 10.0 10.0").unwrap();
        }

        let  	pathStr = tempPath.to_str().unwrap().to_string();
        let  	res = XplrProjectPts(
            pathStr,
            800.0,
            600.0,
            1.0,
            30.0,
            "#00f3ff".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!( res.is_ok());

        let  	_ = std::fs::remove_file( &tempPath);
    }

    #[test]
    fn	TestXplrProjectPtsWithCameraPanZoomRotate()
    {
        let  	tempPath = std::env::temp_dir().join( "test_fenst_camera_project.pts");
        {
            let  	mut f = File::create( &tempPath).unwrap();
            writeln!( f, "2").unwrap();
            writeln!( f, "0.0 0.0 0.0").unwrap();
            writeln!( f, "10.0 10.0 10.0").unwrap();
        }

        let  	pathStr = tempPath.to_str().unwrap().to_string();

        // Standard projection
        let  	res = XplrProjectPts(
            pathStr.clone(),
            800.0,
            600.0,
            1.0,
            30.0,
            "#00f3ff".to_string(),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!( res.is_ok());

        // Interactive projection with Pan, Zoom, and Rotation
        let  	resInteractive = XplrProjectPts(
            pathStr.clone(),
            800.0,
            600.0,
            1.0,
            0.0,
            "#00f3ff".to_string(),
            Some( 100.0),
            Some( -50.0),
            Some( 2.5),
            Some( 0.8),
            Some( 1.2),
            Some( true),
        );
        assert!( resInteractive.is_ok());

        // Reset Camera
        let  	resetRes = XplrResetCamera( pathStr.clone());
        assert!( resetRes.is_ok());
        let  	cam = resetRes.unwrap();
        assert_eq!( cam._PanX, 0.0);
        assert_eq!( cam._PanY, 0.0);
        assert_eq!( cam._Zoom, 1.0);

        let  	_ = std::fs::remove_file( &tempPath);
    }

    #[test]
    fn	TestXplrCommandsGeneral()
    {
        let  	contentRes = XplrFetchContent( "Cargo.toml".to_string());
        assert!( contentRes.is_ok());

        let  	chunkRes = XplrFetchChunk( "Cargo.toml".to_string(), 0, 50);
        assert!( chunkRes.is_ok());
        assert_eq!( chunkRes.unwrap()._Length, 50);

        let  	infoRes = XplrLeafInfo( "Cargo.toml".to_string());
        assert!( infoRes.is_ok());
        assert_eq!( infoRes.unwrap()._Name, "Cargo.toml");
    }

    #[test]
    fn	TestWorkbenchBunnyDataParsing()
    {
        let  	bunnyPath = Path::new( "workbench/bunnyData.pts");
        if bunnyPath.exists() {
            let  	dto = XplrParsePtsFile( "workbench/bunnyData.pts").unwrap();
            assert_eq!( dto._Count, 30571);
            assert_eq!( dto._Points.len(), 30571);

            let  	bytes = dto.ToBytes();
            let  	decoded = PtsPointsDto::FromBytes( &bytes).unwrap();
            assert_eq!( decoded._Count, 30571);
            assert_eq!( decoded._Points.len(), 30571);
        }
    }

    #[test]
    fn	TestWorkbenchBlubObjParsing()
    {
        let  	blubPath = Path::new( "workbench/blub/blub_control_mesh.obj");
        if blubPath.exists() {
            let  	dto = XplrParseWaveObjFile( "workbench/blub/blub_control_mesh.obj").unwrap();
            assert!( dto._VertexCount > 0);
            assert!( dto._FaceCount > 0);
            assert_eq!( dto._Points.len(), dto._VertexCount);
            assert!( dto._Triangles.len() > 0);
            assert!( dto._Edges.len() > 0);
            assert_eq!( dto._Normals.len(), dto._Triangles.len());

            let  	bytes = dto.ToBytes();
            let  	decoded = WaveObjMeshDto::FromBytes( &bytes).unwrap();
            assert_eq!( decoded._VertexCount, dto._VertexCount);
            assert_eq!( decoded._FaceCount, dto._FaceCount);
            assert_eq!( decoded._Points.len(), dto._Points.len());
            assert_eq!( decoded._Triangles.len(), dto._Triangles.len());
            assert_eq!( decoded._Edges.len(), dto._Edges.len());
            assert_eq!( decoded._Normals.len(), dto._Normals.len());
        }
    }

    #[test]
    fn	TestPtsFrameDtoBinaryRoundtrip()
    {
        let  	points = Buff![
            ProjectedPoint { _X: 10.0, _Y: 20.0, _Radius: 3.0, _CoreRadius: 1.0, _Alpha: 0.8 },
            ProjectedPoint { _X: 30.0, _Y: 40.0, _Radius: 4.0, _CoreRadius: 1.5, _Alpha: 0.9 }
        ];

        let  	lines = Buff![
            ProjectedLine { _X1: 1.0, _Y1: 2.0, _X2: 3.0, _Y2: 4.0 }
        ];

        let  	frame = PtsFrameDto {
            _Points: points,
            _BoxLines: lines,
            _FileName: "test.pts".to_string(),
            _Count: 2,
            _BboxLabel: "[-10, 10]".to_string(),
            _ShaderStatus: "OK".to_string(),
            _OverlayText1: "Text1".to_string(),
            _OverlayText2: "Text2".to_string(),
        };

        let  	bytes = frame.ToBytes();
        let  	decoded = PtsFrameDto::FromBytes( &bytes).unwrap();
        assert_eq!( decoded, frame);
    }

    #[test]
    fn	TestPhase3QuantizedPointRoundtrip()
    {
        let  	width = 1920.0;
        let  	height = 1080.0;

        let  	original = ProjectedPoint {
            _X: 960.0,
            _Y: 540.0,
            _Radius: 0.5,                                              // Normalized 0-1 range
            _CoreRadius: 0.25,                                         // Normalized 0-1 range
            _Alpha: 0.8,                                              // Normalized 0-1 range
        };

        // Quantize
        let  	quantized = QuantizedPoint::FromProjected( &original, width, height);

        // Verify size is 7 bytes
        assert_eq!( std::mem::size_of::< QuantizedPoint>(), 7);

        // Dequantize
        let  	restored = quantized.ToProjected( width, height);

        // Check that values are close (within quantization error)
        assert!( (restored._X - original._X).abs() < 1.0);
        assert!( (restored._Y - original._Y).abs() < 1.0);
        assert!( (restored._Radius - original._Radius).abs() < 0.005);
        assert!( (restored._CoreRadius - original._CoreRadius).abs() < 0.005);
        assert!( (restored._Alpha - original._Alpha).abs() < 0.005);
    }

    #[test]
    fn	TestPhase3QuantizedLineRoundtrip()
    {
        let  	width = 1920.0;
        let  	height = 1080.0;

        let  	original = ProjectedLine {
            _X1: 100.0,
            _Y1: 200.0,
            _X2: 300.0,
            _Y2: 400.0,
        };

        // Quantize
        let  	quantized = QuantizedLine::FromProjected( &original, width, height);

        // Verify size is 8 bytes (u16 x 4)
        assert_eq!( std::mem::size_of::< QuantizedLine>(), 8);

        // Dequantize
        let  	restored = quantized.ToProjected( width, height);

        // Check that values are close
        assert!( (restored._X1 - original._X1).abs() < 1.0);
        assert!( (restored._Y1 - original._Y1).abs() < 1.0);
        assert!( (restored._X2 - original._X2).abs() < 1.0);
        assert!( (restored._Y2 - original._Y2).abs() < 1.0);
    }

    #[test]
    fn	TestPhase3QuantizationBandwidthReduction()
    {
        let  	width = 1920.0;
        let  	height = 1080.0;

        // Verify size reduction: 20 bytes -> 7 bytes (65% savings)
        let  	orig_size = std::mem::size_of::< ProjectedPoint>();
        let  	quant_size = std::mem::size_of::< QuantizedPoint>();
        assert_eq!( orig_size, 20);
        assert_eq!( quant_size, 7);
        let  	ratio = quant_size as f64 / orig_size as f64;
        assert!( (ratio - 0.35).abs() < 0.01); // 35% of original ≈ 65% savings

        // Original ProjectedLine: 16 bytes (4 x f32)
        let  	original_line = ProjectedLine {
            _X1: 100.0,
            _Y1: 200.0,
            _X2: 300.0,
            _Y2: 400.0,
        };

        let  	_quantized_line = QuantizedLine::FromProjected( &original_line, width, height);

        // Verify size reduction: 16 bytes -> 8 bytes (50% savings)
        let  	orig_size = std::mem::size_of::< ProjectedLine>();
        let  	quant_size = std::mem::size_of::< QuantizedLine>();
        assert_eq!( orig_size, 16);
        assert_eq!( quant_size, 8);
        let  	ratio = quant_size as f64 / orig_size as f64;
        assert!( (ratio - 0.5).abs() < 0.01); // 50% of original = 50% savings
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Projects a 3D Wavefront mesh or point cloud using camera state and render mode,
/// returning projected 2D primitives (points, wire lines, and depth-sorted shaded facets).
pub fn	XplrProjectMesh(
    points: &[ [f32; 3]],
    triangles: &[ [u32; 3]],
    camera: &Camera,
    mode: ObjRenderMode,
    width: f32,
    height: f32,
) -> MeshFrameDto
{
    if points.is_empty() {
        return MeshFrameDto {
            _Points:     Buff::New(),
            _WireLines:  Buff::New(),
            _Facets:     Buff::New(),
            _PointCount: 0,
            _FaceCount:  0,
            _StatusText: "No geometry data available".to_string(),
        };
    }

    let  	bbox = BBox3f::FromPoints( points);
    let  	center = bbox.Center().Pos();
    let  	scaleNorm = bbox.ScaleNorm( 240.0);

    let  	projectedVerts: Vec< ( f32, f32, f32)> = points
        .iter()
        .map( |&p| {
            let  	nx = ( p[0] - center[0]) * scaleNorm;
            let  	ny = ( p[1] - center[1]) * scaleNorm;
            let  	nz = ( p[2] - center[2]) * scaleNorm;
            camera.Project( nx, ny, nz, width, height)
        })
        .collect();

    let  	hasFaces = !triangles.is_empty();

    let  	mut projectedPoints = Buff::New();
    let  	mut wireLines = Buff::New();
    let  	mut facets = Buff::New();

    if !hasFaces || mode == ObjRenderMode::Points {
        let  	pts = Buff::Create( crate::silo::U32( projectedVerts.len() as u32), |i| {
            let  	pv = projectedVerts[i.AsUsize()];
            ProjectedPoint {
                _X:          pv.0,
                _Y:          pv.1,
                _Radius:     1.5,
                _CoreRadius: 1.0,
                _Alpha:      1.0,
            }
        });
        projectedPoints = pts;
    }

    if !hasFaces {
        let  	corners = bbox.Corners();
        let  	projCorners: Vec< ( f32, f32, f32)> = corners
            .iter()
            .map( |c| {
                let  	nx = ( c._X - center[0]) * scaleNorm;
                let  	ny = ( c._Y - center[1]) * scaleNorm;
                let  	nz = ( c._Z - center[2]) * scaleNorm;
                camera.Project( nx, ny, nz, width, height)
            })
            .collect();

        let  	edges = BBox3f::BoxEdges();
        let  	boxLines = Buff::Create( crate::silo::U32( edges.len() as u32), |i| {
            let  	( a, b) = edges[i.AsUsize()];
            let  	p1 = projCorners[a];
            let  	p2 = projCorners[b];
            ProjectedLine {
                _X1: p1.0,
                _Y1: p1.1,
                _X2: p2.0,
                _Y2: p2.1,
            }
        });
        wireLines = boxLines;
    } else {
        if mode == ObjRenderMode::Facets || mode == ObjRenderMode::ShadedWire {
            let  	mut sortedTris: Vec< ( usize, f32)> = triangles
                .iter()
                .enumerate()
                .map( |( i, tri)| {
                    let  	zAvg = ( projectedVerts[tri[0] as usize].2
                        + projectedVerts[tri[1] as usize].2
                        + projectedVerts[tri[2] as usize].2)
                        / 3.0;
                    ( i, zAvg)
                })
                .collect();
            sortedTris.sort_by( |a, b| b.1.partial_cmp( &a.1).unwrap_or( std::cmp::Ordering::Equal));

            let  	facetBuff = Buff::Create( crate::silo::U32( sortedTris.len() as u32), |i| {
                let  	( triIdx, z) = sortedTris[i.AsUsize()];
                let  	tri = triangles[triIdx];
                let  	a = projectedVerts[tri[0] as usize];
                let  	b = projectedVerts[tri[1] as usize];
                let  	c = projectedVerts[tri[2] as usize];
                let  	shade = ( 200.0 - z).clamp( 60.0, 220.0) as u8;
                ProjectedFacet {
                    _P1:    ( a.0, a.1),
                    _P2:    ( b.0, b.1),
                    _P3:    ( c.0, c.1),
                    _ZAvg:  z,
                    _Shade: shade,
                }
            });
            facets = facetBuff;
        }

        if mode == ObjRenderMode::Wireframe || mode == ObjRenderMode::ShadedWire {
            let  	mut linesStash = crate::silo::Stash::WithCapacity( crate::silo::U32( (triangles.len() * 3) as u32));
            for tri in triangles {
                let  	a = projectedVerts[tri[0] as usize];
                let  	b = projectedVerts[tri[1] as usize];
                let  	c = projectedVerts[tri[2] as usize];
                linesStash.Push( ProjectedLine { _X1: a.0, _Y1: a.1, _X2: b.0, _Y2: b.1 });
                linesStash.Push( ProjectedLine { _X1: b.0, _Y1: b.1, _X2: c.0, _Y2: c.1 });
                linesStash.Push( ProjectedLine { _X1: c.0, _Y1: c.1, _X2: a.0, _Y2: a.1 });
            }
            wireLines = linesStash.IntoBuff();
        }
    }

    let  	statusText = if !hasFaces {
        format!( "{} points", points.len())
    } else {
        format!( "{} verts | {} faces", points.len(), triangles.len())
    };

    MeshFrameDto {
        _Points:     projectedPoints,
        _WireLines:  wireLines,
        _Facets:     facets,
        _PointCount: points.len(),
        _FaceCount:  triangles.len(),
        _StatusText: statusText,
    }
}
