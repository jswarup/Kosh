//-- fenst/mod.rs -----------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	std::fs;
use	std::path::PathBuf;
use	std::time::UNIX_EPOCH;
use	serde::Serialize;
use	crate::flux::{ BuffStream, IStream };
use	crate::silo::{ U32, ISliceExt };

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
pub fn	XplrListEntries( path: String) -> Result< Vec< XplrEntry>, String>
{
    let  	branch = FsBranch::New( path);
    let  	children = branch.Children()?;
    let  	mut entries: Vec< XplrEntry> = Vec::new();

    for child in children {
        let  	isDir = !child.IsLeaf();
        let  	size = child.AsLeaf().map( |l| l.Size()).unwrap_or( 0);
        let  	extension = child.AsLeaf().map( |l| l.Extension().to_string()).unwrap_or_default();

        entries.push( XplrEntry {
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
