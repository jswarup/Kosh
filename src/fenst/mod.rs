//-- fenst/mod.rs -----------------------------------------------------------------------------------------------------------------
#![allow( non_snake_case, non_camel_case_types, non_upper_case_globals)]
use	std::fs;
use	std::path::PathBuf;
use	std::time::UNIX_EPOCH;
use	serde::Serialize;

// ---------------------------------------------------------------------------------------------------------------------------------

/// A single entry in a directory listing.
#[derive( Serialize, Clone, Debug)]
pub struct FileEntry
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
pub struct FileContents
{
    pub path:       String,
    pub content:    String,
    pub size:       u64,
    pub line_count: usize,
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Metadata about a file.
#[derive( Serialize, Debug)]
pub struct FileInfo
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
pub fn	read_directory( path: String) -> Result< Vec< FileEntry>, String>
{
    let  	branch = FsBranch::New( path);
    let  	children = branch.Children()?;
    let  	mut entries: Vec< FileEntry> = Vec::new();

    for child in children {
        let  	isDir = !child.IsLeaf();
        let  	size = child.AsLeaf().map( |l| l.Size()).unwrap_or( 0);
        let  	extension = child.AsLeaf().map( |l| l.Extension().to_string()).unwrap_or_default();

        entries.push( FileEntry {
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

/// Reads the text content of a file, with a size guard.
pub fn	read_file_contents( path: String) -> Result< FileContents, String>
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

    let  	content = fs::read_to_string( &filePath)
        .map_err( |e| format!( "Failed to read file: {}", e))?;

    let  	lineCount = content.lines().count();

    Ok( FileContents {
        path:       filePath.to_string_lossy().into_owned(),
        content,
        size,
        line_count: lineCount,
    })
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Returns metadata about a file or directory.
pub fn	get_file_info( path: String) -> Result< FileInfo, String>
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

    Ok( FileInfo {
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

pub use	xplr::{ Xplr, LeafXplr, BranchXplr };
pub use	fsxplr::{ FsLeaf, FsBranch };

// ---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests;

// ---------------------------------------------------------------------------------------------------------------------------------
