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
pub struct XplrEntry
{
    pub name:       String,
    pub path:       String,
    pub is_dir:     bool,
    pub size:       u64,
    pub extension:  String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Contents of a file with metadata.
#[derive( Serialize, Debug)]
pub struct XplrContent
{
    pub path:       String,
    pub content:    String,
    pub size:       u64,
    pub line_count: usize,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Metadata about a file.
#[derive( Serialize, Debug)]
pub struct XplrLeafInfo
{
    pub path:       String,
    pub name:       String,
    pub size:       u64,
    pub is_dir:     bool,
    pub modified:   u64,
    pub extension:  String,
    pub readonly:   bool,
}

// ---------------------------------------------------------------------------------------------------------------------------------

const MAX_FILE_SIZE: u64 = 10 * 1024 * 1024;                       // 10 MB guard

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads a directory and returns sorted entries (directories first, then files).
pub fn	XplrListEntries( path: String) -> Result< Buff< XplrEntry>, String>
{
    let  	branch = FsBranch::New( path);
    let  	children = branch.Children()?;
    let  	mut entries: Buff< XplrEntry> = Buff::NewEmpty();

    for child in children {
        let  	isDir = !child.IsLeaf();
        let  	size = child.AsLeaf().map( |l| l.Size()).unwrap_or( 0);
        let  	extension = child.AsLeaf().map( |l| l.Extension().to_string()).unwrap_or_default();

        entries.Push( XplrEntry {
            name:       child.Name().to_string(),
            path:       child.Path().to_string(),
            is_dir:     isDir,
            size,
            extension,
        });
    }

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
        path:       filePath.to_string_lossy().into_owned(),
        content,
        size,
        line_count: lineCount,
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
        path:       filePath.to_string_lossy().into_owned(),
        offset,
        length:     readLen,
        total_size: totalSize,
        is_eof:     isEof,
        content:    contentStr,
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
        path:       filePath.to_string_lossy().into_owned(),
        name,
        size:       metadata.len(),
        is_dir:     metadata.is_dir(),
        modified,
        extension,
        readonly,
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

/// DTO for 3D point cloud data generated by the GPU compute shader.
#[derive( Serialize, Debug)]
pub struct PtsPointsDto
{
    pub points:     Buff< [f32; 3]>,
    pub count:      usize,
    pub bbox_min:   [f32; 3],
    pub bbox_max:   [f32; 3],
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
        count:      points.len(),
        points,
        bbox_min:   [-20.0, -20.0, -20.0],
        bbox_max:   [20.0, 20.0, 20.0],
    })
}

// ---------------------------------------------------------------------------------------------------------------------------------


pub mod xplr;
pub mod fsxplr;
pub mod frescoxplr;
pub mod shardxplr;
pub mod provider;

pub use	xplr::{ Xplr, LeafXplr, BranchXplr, XplrNodeDto, StreamChunkDto };
pub use	fsxplr::{ FsLeaf, FsBranch };
pub use	frescoxplr::{ FrescoLeaf, FrescoBranch, FrescoProvider };
pub use	shardxplr::{ ShardLeaf, ShardBranch, ShardProvider };
pub use	provider::{ XplrProvider, FsProvider, XplrRegistry, SharedXplrRegistry, CreateDefaultRegistry };

// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests;

// ---------------------------------------------------------------------------------------------------------------------------------
