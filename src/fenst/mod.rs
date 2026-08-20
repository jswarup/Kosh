//-- fenst/mod.rs -----------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	std::fs;
use	std::path::PathBuf;
use	std::time::UNIX_EPOCH;
use	serde::Serialize;
use	crate::flux::{ BuffStream, IStream };
use	crate::silo::{ Buff, U32, ISliceExt };
use	crate::swarm::IGpuOp;

// ---------------------------------------------------------------------------------------------------------------------------------

/// A single entry in an explorer listing.
#[derive( Serialize, Clone, Debug)]
#[serde( rename_all = "snake_case")]
pub struct XplrEntry
{
    pub _Name:       String,
    pub _Path:       String,
    pub _IsDir:      bool,
    pub _Size:       u64,
    pub _Extension:  String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Contents of a file with metadata.
#[derive( Serialize, Debug)]
#[serde( rename_all = "snake_case")]
pub struct XplrContent
{
    pub _Path:       String,
    pub _Content:    String,
    pub _Size:       u64,
    pub _LineCount:  usize,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Metadata about a file.
#[derive( Serialize, Debug)]
#[serde( rename_all = "snake_case")]
pub struct XplrLeafInfo
{
    pub _Path:       String,
    pub _Name:       String,
    pub _Size:       u64,
    pub _IsDir:      bool,
    pub _Modified:   u64,
    pub _Extension:  String,
    pub _Readonly:   bool,
}

// ---------------------------------------------------------------------------------------------------------------------------------

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;                       // 10 MB guard

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a directory and returns sorted entries (directories first, then files).
pub fn	XplrListEntries( path: String) -> Result< Buff< XplrEntry>, String>
{
    let  	branch = FsBranch::New( path);
    let  	children = branch.Children()?;
    let  	entries = Buff::Create( children.Size(), |i| {
        let  	child = &children[i.AsUsize()];
        let  	isDir = !child.IsLeaf();
        let  	size = child.AsLeaf().map( |l| l.Size()).unwrap_or( 0);
        let  	extension = child.AsLeaf().map( |l| l.Extension().to_string()).unwrap_or_default();

        XplrEntry {
            _Name:       child.Name().to_string(),
            _Path:       child.Path().to_string(),
            _IsDir:      isDir,
            _Size:       size,
            _Extension:  extension,
        }
    });

    Ok( entries)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads the text content of a file using flux::BuffStream, with a size guard.
pub fn	XplrFetchContent( path: String) -> Result< XplrContent, String>
{
    let  	filePath = PathBuf::from( &path);
    if !filePath.exists() {
        return Err( format!( "File does not exist: {}", path));
    }
    if !filePath.is_file() {
        return Err( format!( "Path is not a file: {}", path));
    }

    let  	metadata = fs::metadata( &filePath)
        .map_err( |e| format!( "Failed to read metadata: {}", e))?;

    let  	size = metadata.len();
    if size > MAX_FILE_SIZE {
        return Err( format!(
            "File too large ({} bytes). Maximum supported size is {} bytes.",
            size, MAX_FILE_SIZE
        ));
    }

    let  	mut stream = BuffStream::FromFile( &filePath)
        .map_err( |e| format!( "Failed to open file stream: {}", e))?;

    let  	bytesArr = stream.BytesAt( U32( 0), U32( size as u32));
    let  	byteSlice = bytesArr.CastSlice();
    let  	content = String::from_utf8_lossy( byteSlice).into_owned();

    let  	lineCount = content.lines().count();

    Ok( XplrContent {
        _Path:       filePath.to_string_lossy().into_owned(),
        _Content:    content,
        _Size:       size,
        _LineCount:  lineCount,
    })
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a windowed chunk of a file using flux::BuffStream and silo::Buff.
pub fn	XplrFetchChunk( path: String, offset: u64, size: usize) -> Result< StreamChunkDto, String>
{
    let  	filePath = PathBuf::from( &path);
    if !filePath.exists() {
        return Err( format!( "File does not exist: {}", path));
    }
    if !filePath.is_file() {
        return Err( format!( "Path is not a file: {}", path));
    }

    let  	metadata = fs::metadata( &filePath)
        .map_err( |e| format!( "Failed to read metadata: {}", e))?;
    let  	totalSize = metadata.len();

    let  	mut stream = BuffStream::FromFile( &filePath)
        .map_err( |e| format!( "Failed to open file stream: {}", e))?;

    let  	offsetU32 = U32( offset as u32);
    let  	countU32 = U32( size as u32);

    let  	bytesArr = stream.BytesAt( offsetU32, countU32);
    let  	byteSlice = bytesArr.CastSlice();
    let  	contentStr = String::from_utf8_lossy( byteSlice).into_owned();

    let  	readLen = byteSlice.len();
    let  	isEof = ( offset + readLen as u64) >= totalSize;

    Ok( StreamChunkDto {
        _Path:       filePath.to_string_lossy().into_owned(),
        _Offset:      offset,
        _Length:      readLen,
        _TotalSize:   totalSize,
        _IsEof:       isEof,
        _Content:     contentStr,
    })
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns metadata about a file or directory.
pub fn	XplrLeafInfo( path: String) -> Result< XplrLeafInfo, String>
{
    let  	filePath = PathBuf::from( &path);
    if !filePath.exists() {
        return Err( format!( "Path does not exist: {}", path));
    }

    let  	metadata = fs::metadata( &filePath)
        .map_err( |e| format!( "Failed to read metadata: {}", e))?;

    let  	modified = metadata.modified()
        .map( |t| t.duration_since( UNIX_EPOCH).unwrap_or_default().as_secs())
        .unwrap_or( 0);

    let  	name = filePath.file_name()
        .map( |n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let  	extension = filePath.extension()
        .map( |e| e.to_string_lossy().into_owned())
        .unwrap_or_default();

    let  	readonly = metadata.permissions().readonly();

    Ok( XplrLeafInfo {
        _Path:       filePath.to_string_lossy().into_owned(),
        _Name:       name,
        _Size:       metadata.len(),
        _IsDir:      metadata.is_dir(),
        _Modified:   modified,
        _Extension:  extension,
        _Readonly:   readonly,
    })
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Checks if a file path or extension corresponds to a .pts file (extension starting with .pts).
pub fn	IsPtsFile( path: &str) -> bool
{
    let  	filePath = PathBuf::from( path);
    let  	ext = filePath.extension()
        .map( |e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let  	fileName = filePath.file_name()
        .map( |n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    ext.starts_with( "pts") || fileName.contains( ".pts")
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Checks if a file path or extension corresponds to a Wavefront .obj file.
pub fn	IsObjFile( path: &str) -> bool
{
    let  	filePath = PathBuf::from( path);
    let  	ext = filePath.extension()
        .map( |e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let  	fileName = filePath.file_name()
        .map( |n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    ext == "obj" || fileName.ends_with( ".obj")
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Fixed binary header for PtsPoints binary serialization.
#[repr(C)]
#[derive( Debug, Clone, Copy, PartialEq)]
pub struct PtsPointsBinaryHeader
{
    pub _Magic:       u32,                                              // 0x50545350 ('PTSP')
    pub _Version:     u32,                                              // Format version: 1
    pub _PointCount:  u32,                                              // Number of 3D points
    pub _Reserved:    u32,                                              // 8-byte alignment padding
    pub _BboxMin:     [f32; 3],                                         // Bounding box minimum
    pub _BboxMax:     [f32; 3],                                         // Bounding box maximum
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Fixed binary header for WaveObjMesh binary serialization.
#[repr(C)]
#[derive( Debug, Clone, Copy, PartialEq)]
pub struct WaveObjMeshBinaryHeader
{
    pub _Magic:         u32,                                            // 0x4D455348 ('MESH')
    pub _Version:       u32,                                            // Format version: 1
    pub _VertexCount:   u32,                                            // Number of vertex points
    pub _FaceCount:     u32,                                            // Number of faces
    pub _TriangleCount: u32,                                            // Number of triangles
    pub _EdgeCount:     u32,                                            // Number of wireframe edges
    pub _NormalCount:   u32,                                            // Number of normal vectors
    pub _Reserved:      u32,                                            // 8-byte alignment padding
    pub _BboxMin:       [f32; 3],                                       // Bounding box minimum
    pub _BboxMax:       [f32; 3],                                       // Bounding box maximum
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// DTO for 3D polygonal mesh data parsed from a Wavefront .obj file.
#[derive( Serialize, Debug, Clone, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct WaveObjMeshDto
{
    pub _Points:        Buff< [f32; 3]>,
    pub _Triangles:     Buff< [u32; 3]>,
    pub _Edges:         Buff< [u32; 2]>,
    pub _Normals:       Buff< [f32; 3]>,
    pub _VertexCount:   usize,
    pub _FaceCount:     usize,
    pub _BboxMin:       [f32; 3],
    pub _BboxMax:       [f32; 3],
}

impl WaveObjMeshDto
{
    /// Serializes the mesh DTO into a compact #[repr(C)] binary payload.
    pub fn	ToBytes( &self) -> Vec< u8>
    {
        let  	header = WaveObjMeshBinaryHeader {
            _Magic:         0x4D455348,
            _Version:       1,
            _VertexCount:   self._Points.len() as u32,
            _FaceCount:     self._FaceCount as u32,
            _TriangleCount: self._Triangles.len() as u32,
            _EdgeCount:     self._Edges.len() as u32,
            _NormalCount:   self._Normals.len() as u32,
            _Reserved:      0,
            _BboxMin:       self._BboxMin,
            _BboxMax:       self._BboxMax,
        };

        let  	headerSlice = std::slice::from_ref( &header);
        let  	headerBytes: &[u8] = headerSlice.CastSlice();

        let  	pointsBytes: &[u8] = self._Points.CastSlice();
        let  	trianglesBytes: &[u8] = self._Triangles.CastSlice();
        let  	edgesBytes: &[u8] = self._Edges.CastSlice();
        let  	normalsBytes: &[u8] = self._Normals.CastSlice();

        let  	totalLen = headerBytes.len() + pointsBytes.len() + trianglesBytes.len() + edgesBytes.len() + normalsBytes.len();
        let  	mut out = Vec::with_capacity( totalLen);

        out.extend_from_slice( headerBytes);
        out.extend_from_slice( pointsBytes);
        out.extend_from_slice( trianglesBytes);
        out.extend_from_slice( edgesBytes);
        out.extend_from_slice( normalsBytes);

        return out;
    }

    /// Deserializes a mesh DTO from a #[repr(C)] binary payload.
    pub fn	FromBytes( bytes: &[u8]) -> Result< Self, String>
    {
        let  	headerSz = std::mem::size_of::< WaveObjMeshBinaryHeader>();
        if bytes.len() < headerSz {
            return Err( "WaveObjMesh binary payload too short for header".to_string());
        }

        let  	headerSlice: &[WaveObjMeshBinaryHeader] = bytes[..headerSz].CastSliceFrom();
        let  	header = headerSlice[0];

        if header._Magic != 0x4D455348 {
            return Err( format!( "Invalid WaveObjMesh magic: 0x{:08X}", header._Magic));
        }

        let  	mut offset = headerSz;

        let  	pointsByteLen = header._VertexCount as usize * std::mem::size_of::< [f32; 3]>();
        if bytes.len() < offset + pointsByteLen {
            return Err( "WaveObjMesh payload truncated in points buffer".to_string());
        }
        let  	pointsSlice: &[[f32; 3]] = bytes[offset..offset + pointsByteLen].CastSliceFrom();
        let  	points = Buff::from(  pointsSlice);
        offset += pointsByteLen;

        let  	trianglesByteLen = header._TriangleCount as usize * std::mem::size_of::< [u32; 3]>();
        if bytes.len() < offset + trianglesByteLen {
            return Err( "WaveObjMesh payload truncated in triangles buffer".to_string());
        }
        let  	trianglesSlice: &[[u32; 3]] = bytes[offset..offset + trianglesByteLen].CastSliceFrom();
        let  	triangles = Buff::from(  trianglesSlice);
        offset += trianglesByteLen;

        let  	edgesByteLen = header._EdgeCount as usize * std::mem::size_of::< [u32; 2]>();
        if bytes.len() < offset + edgesByteLen {
            return Err( "WaveObjMesh payload truncated in edges buffer".to_string());
        }
        let  	edgesSlice: &[[u32; 2]] = bytes[offset..offset + edgesByteLen].CastSliceFrom();
        let  	edges = Buff::from(  edgesSlice);
        offset += edgesByteLen;

        let  	normalsByteLen = header._NormalCount as usize * std::mem::size_of::< [f32; 3]>();
        if bytes.len() < offset + normalsByteLen {
            return Err( "WaveObjMesh payload truncated in normals buffer".to_string());
        }
        let  	normalsSlice: &[[f32; 3]] = bytes[offset..offset + normalsByteLen].CastSliceFrom();
        let  	normals = Buff::from(  normalsSlice);

        return Ok( WaveObjMeshDto {
            _Points:        points,
            _Triangles:     triangles,
            _Edges:         edges,
            _Normals:       normals,
            _VertexCount:   header._VertexCount as usize,
            _FaceCount:     header._FaceCount as usize,
            _BboxMin:       header._BboxMin,
            _BboxMax:       header._BboxMax,
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// DTO for 3D point cloud data generated by the GPU compute shader.
#[derive( Serialize, Debug, Clone, PartialEq)]
#[serde( rename_all = "snake_case")]
pub struct PtsPointsDto
{
    pub _Points:     Buff< [f32; 3]>,
    pub _Count:      usize,
    pub _BboxMin:    [f32; 3],
    pub _BboxMax:    [f32; 3],
}

impl PtsPointsDto
{
    /// Serializes the point cloud DTO into a compact #[repr(C)] binary payload.
    pub fn	ToBytes( &self) -> Vec< u8>
    {
        let  	header = PtsPointsBinaryHeader {
            _Magic:       0x50545350,
            _Version:     1,
            _PointCount:  self._Points.len() as u32,
            _Reserved:    0,
            _BboxMin:     self._BboxMin,
            _BboxMax:     self._BboxMax,
        };

        let  	headerSlice = std::slice::from_ref( &header);
        let  	headerBytes: &[u8] = headerSlice.CastSlice();
        let  	pointsBytes: &[u8] = self._Points.CastSlice();

        let  	mut out = Vec::with_capacity( headerBytes.len() + pointsBytes.len());
        out.extend_from_slice( headerBytes);
        out.extend_from_slice( pointsBytes);

        return out;
    }

    /// Deserializes a point cloud DTO from a #[repr(C)] binary payload.
    pub fn	FromBytes( bytes: &[u8]) -> Result< Self, String>
    {
        let  	headerSz = std::mem::size_of::< PtsPointsBinaryHeader>();
        if bytes.len() < headerSz {
            return Err( "PtsPoints binary payload too short for header".to_string());
        }

        let  	headerSlice: &[PtsPointsBinaryHeader] = bytes[..headerSz].CastSliceFrom();
        let  	header = headerSlice[0];

        if header._Magic != 0x50545350 {
            return Err( format!( "Invalid PtsPoints magic: 0x{:08X}", header._Magic));
        }

        let  	pointsByteLen = header._PointCount as usize * std::mem::size_of::< [f32; 3]>();
        if bytes.len() < headerSz + pointsByteLen {
            return Err( "PtsPoints payload truncated in points buffer".to_string());
        }

        let  	pointsSlice: &[[f32; 3]] = bytes[headerSz..headerSz + pointsByteLen].CastSliceFrom();
        let  	points = Buff::from(  pointsSlice);

        return Ok( PtsPointsDto {
            _Points:     points,
            _Count:      header._PointCount as usize,
            _BboxMin:    header._BboxMin,
            _BboxMax:    header._BboxMax,
        });
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Generates 100 pseudo-random 3D points in [-20, 20]³ using the rust-gpu
/// `pts_pointcloud_cs` compute shader and returns them as a serializable DTO.
pub fn	XplrFetchPtsPoints( spirvBytes: &[u8]) -> Result< PtsPointsDto, String>
{
    let  	numPoints: U32 = U32( 100);
    let  	numPointsUsize = numPoints.AsUsize();

    // Initialize wgpu device and queue
    let  	( device, queue) = wgpu::Device::Init()
        .ok_or_else( || "No GPU adapter found — cannot generate point cloud".to_string())?;

    // Create output buffer: 100 Vec4s (4 floats each = 16 bytes per point)
    let  	vec4Size = std::mem::size_of::< f32>() * 4;
    let  	byteLen = numPointsUsize * vec4Size;
    let  	zeroBuff = Buff::Create( ( byteLen / std::mem::size_of::< f32>()) as u32, |_| 0.0f32);

    let  	gpuOut = device.BufferInit(
        "pts_pointcloud_out",
        zeroBuff.CastSlice(),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );

    // Load pre-compiled SPIR-V shader module
    let  	spirv = std::borrow::Cow::Owned(
        wgpu::util::make_spirv_raw( spirvBytes).into_owned()
    );
    let  	shader = device.create_shader_module( wgpu::ShaderModuleDescriptor {
        label: Some( "pts_pointcloud_shader"),
        source: wgpu::ShaderSource::SpirV( spirv),
    });

    // Create bind group layout, pipeline layout, and compute pipeline
    let  	bindGroupLayout = device.create_bind_group_layout( &wgpu::BindGroupLayoutDescriptor {
        label: Some( "pts_pointcloud_bgl"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    });

    let  	pipelineLayout = device.create_pipeline_layout( &wgpu::PipelineLayoutDescriptor {
        label: Some( "pts_pointcloud_pl"),
        bind_group_layouts: &[Some( &bindGroupLayout)],
        immediate_size: 0,
    });

    let  	pipeline = device.create_compute_pipeline( &wgpu::ComputePipelineDescriptor {
        label: Some( "pts_pointcloud_pipeline"),
        layout: Some( &pipelineLayout),
        module: &shader,
        entry_point: Some( "pts_pointcloud_cs"),
        compilation_options: Default::default(),
        cache: None,
    });

    let  	bindGroup = device.create_bind_group( &wgpu::BindGroupDescriptor {
        label: Some( "pts_pointcloud_bg"),
        layout: &bindGroupLayout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: gpuOut.as_entire_binding(),
            },
        ],
    });

    // Dispatch compute shader: ceil(100 / 64) = 2 workgroups
    let  	workgroups = ( numPoints.AsU32() + 63) / 64;
    let  	mut encoder = device.create_command_encoder( &wgpu::CommandEncoderDescriptor {
        label: Some( "pts_pointcloud_enc"),
    });
    {
        let  	mut pass = encoder.begin_compute_pass( &wgpu::ComputePassDescriptor {
            label: Some( "pts_pointcloud_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline( &pipeline);
        pass.set_bind_group( 0, &bindGroup, &[]);
        pass.dispatch_workgroups( workgroups, 1, 1);
    }
    queue.submit( std::iter::once( encoder.finish()));

    // Read back GPU buffer
    let  	rawBytes = device.ReadBuffer( &queue, &gpuOut, byteLen as u64);
    let  	floatSlice: &[f32] = rawBytes.CastSliceFrom();

    let  	points = Buff::Create( numPoints, |i| {
        let  	base = i.AsUsize() * 4;
        if base + 2 < floatSlice.len() {
            [floatSlice[base], floatSlice[base + 1], floatSlice[base + 2]]
        } else {
            [0.0f32, 0.0, 0.0]
        }
    });

    Ok( PtsPointsDto {
        _Points:     points,
        _Count:      numPointsUsize,
        _BboxMin:    [-20.0, -20.0, -20.0],
        _BboxMax:    [20.0, 20.0, 20.0],
    })
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Parses a .pts point cloud file from disk using flux::BuffStream and fleck::ParsePtsStream.
pub fn	XplrParsePtsFile( path: &str) -> Result< PtsPointsDto, String>
{
    let  	filePath = PathBuf::from( path);
    if !filePath.exists() {
        return Err( format!( "File does not exist: {}", path));
    }
    if !filePath.is_file() {
        return Err( format!( "Path is not a file: {}", path));
    }

    let  	mut stream = BuffStream::FromFile( &filePath)
        .map_err( |e| format!( "Failed to open file stream for {}: {}", path, e))?;

    stream.ReadAll()
        .map_err( |e| format!( "Failed to read stream for {}: {}", path, e))?;

    let  	cloud = crate::fleck::ParsePtsStream( &mut stream)
        .map_err( |e| format!( "Failed to parse .pts stream for {}: {}", path, e))?;

    Ok( cloud.ToDto())
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Parses a Wavefront .obj 3D mesh file from disk using flux::BuffStream and fleck::ParseWaveObjStream.
pub fn	XplrParseWaveObjFile( path: &str) -> Result< WaveObjMeshDto, String>
{
    let  	filePath = PathBuf::from( path);
    if !filePath.exists() {
        return Err( format!( "File does not exist: {}", path));
    }
    if !filePath.is_file() {
        return Err( format!( "Path is not a file: {}", path));
    }

    let  	mut stream = BuffStream::FromFile( &filePath)
        .map_err( |e| format!( "Failed to open file stream for {}: {}", path, e))?;

    stream.ReadAll()
        .map_err( |e| format!( "Failed to read stream for {}: {}", path, e))?;

    let  	model = crate::fleck::ParseWaveObjStream( &mut stream)
        .map_err( |e| format!( "Failed to parse .obj stream for {}: {}", path, e))?;

    Ok( model.ToMeshDto())
}

// ---------------------------------------------------------------------------------------------------------------------------------


pub mod xplr;
pub mod fsxplr;
pub mod frescoxplr;
pub mod shardxplr;
pub mod provider;
pub mod scene;
pub mod app;
pub mod xplrcmds;

pub use	xplr::{ Xplr, LeafXplr, BranchXplr, XplrNodeDto, StreamChunkDto };
pub use	fsxplr::{ FsLeaf, FsBranch };
pub use	frescoxplr::{ FrescoLeaf, FrescoBranch, FrescoProvider };
pub use	shardxplr::{ ShardLeaf, ShardBranch, ShardProvider };
pub use	provider::{ XplrProvider, FsProvider, XplrRegistry, SharedXplrRegistry, CreateDefaultRegistry };
pub use	scene::{ Camera, SceneGraph, SceneDisplayFrame };
pub use	app::run;

// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests;

// ---------------------------------------------------------------------------------------------------------------------------------
