//-- fenst/_tests.rs ---------------------------------------------------------------------------------------------------------------
use	crate::fenst::{ read_directory, read_file_contents, get_file_info };

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestReadDirectory()
{
    // Test reading the project's own src directory (known to exist)
    let  	result = read_directory( "src".to_string());
    assert!( result.is_ok(), "Failed to read 'src' directory: {:?}", result.err());

    let  	entries = result.unwrap();
    assert!( !entries.is_empty(), "src directory should not be empty");

    // Verify directories come before files
    let  	mut seenFile = false;
    for entry in &entries {
        if !entry.is_dir {
            seenFile = true;
        }
        if entry.is_dir && seenFile {
            panic!( "Directory '{}' appeared after a file — sort order broken", entry.name);
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestReadDirectoryNonExistent()
{
    let  	result = read_directory( "__nonexistent_dir__".to_string());
    assert!( result.is_err());
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestReadFileContents()
{
    // Read this project's Cargo.toml (known to exist)
    let  	result = read_file_contents( "Cargo.toml".to_string());
    assert!( result.is_ok(), "Failed to read Cargo.toml: {:?}", result.err());

    let  	contents = result.unwrap();
    assert!( contents.size > 0);
    assert!( contents.line_count > 0);
    assert!( contents.content.contains( "[package]"));
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestReadFileContentsNonExistent()
{
    let  	result = read_file_contents( "__nonexistent_file__.txt".to_string());
    assert!( result.is_err());
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestGetFileInfo()
{
    let  	result = get_file_info( "Cargo.toml".to_string());
    assert!( result.is_ok(), "Failed to get file info: {:?}", result.err());

    let  	info = result.unwrap();
    assert_eq!( info.name, "Cargo.toml");
    assert!( !info.is_dir);
    assert!( info.size > 0);
    assert_eq!( info.extension, "toml");
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestGetFileInfoDirectory()
{
    let  	result = get_file_info( "src".to_string());
    assert!( result.is_ok());

    let  	info = result.unwrap();
    assert!( info.is_dir);
    assert_eq!( info.name, "src");
}

// ---------------------------------------------------------------------------------------------------------------------------------
