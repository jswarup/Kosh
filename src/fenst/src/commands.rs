//-- commands.rs ------------------------------------------------------------------------------------------------------------------
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
#[tauri::command]
pub fn	read_directory( path: String) -> Result< Vec< FileEntry>, String>
{
    let  	dirPath = PathBuf::from( &path);
    if !dirPath.exists() {
        return Err( format!( "Path does not exist: {}", path));
    }
    if !dirPath.is_dir() {
        return Err( format!( "Path is not a directory: {}", path));
    }

    let  	readDir = fs::read_dir( &dirPath)
        .map_err( |e| format!( "Failed to read directory: {}", e))?;

    let  	mut dirs: Vec< FileEntry> = Vec::new();
    let  	mut files: Vec< FileEntry> = Vec::new();

    for entry in readDir {
        let  	entry = match entry {
            Ok( e) => e,
            Err( _) => continue,
        };
        let  	filePath = entry.path();
        let  	metadata = match entry.metadata() {
            Ok( m) => m,
            Err( _) => continue,
        };
        let  	fileName = entry.file_name().to_string_lossy().into_owned();

        // Skip hidden files (dotfiles)
        if fileName.starts_with( '.') {
            continue;
        }

        let  	isDir = metadata.is_dir();
        let  	size = if isDir { 0 } else { metadata.len() };
        let  	extension = filePath.extension()
            .map( |e| e.to_string_lossy().into_owned())
            .unwrap_or_default();

        let  	fileEntry = FileEntry {
            name:       fileName,
            path:       filePath.to_string_lossy().into_owned(),
            is_dir:     isDir,
            size,
            extension,
        };

        if isDir {
            dirs.push( fileEntry);
        } else {
            files.push( fileEntry);
        }
    }

    // Sort each group alphabetically (case-insensitive)
    dirs.sort_by( |a, b| a.name.to_lowercase().cmp( &b.name.to_lowercase()));
    files.sort_by( |a, b| a.name.to_lowercase().cmp( &b.name.to_lowercase()));

    // Directories first, then files
    dirs.append( &mut files);
    Ok( dirs)
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Reads the text content of a file, with a size guard.
#[tauri::command]
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
#[tauri::command]
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

/// Shows a native dialog to pick a folder.
#[tauri::command]
pub fn	select_directory() -> Result< Option< String>, String>
{
    let  	fileDialog = rfd::FileDialog::new()
        .set_title( "Select Folder to Open");

    let  	folderPath = fileDialog.pick_folder();

    Ok( folderPath.map( |p| p.to_string_lossy().into_owned()))
}

// ---------------------------------------------------------------------------------------------------------------------------------
