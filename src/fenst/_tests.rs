//-- fenst/_tests.rs ---------------------------------------------------------------------------------------------------------------
use	crate::fenst::{ XplrListEntries, XplrFetchContent, XplrLeafInfo };

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestReadDirectory()
{
    // Test reading the project's own src directory (known to exist)
    let  	result = XplrListEntries( "src".to_string());
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
    let  	result = XplrListEntries( "__nonexistent_dir__".to_string());
    assert!( result.is_err());
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestReadFileContents()
{
    // Read this project's Cargo.toml (known to exist)
    let  	result = XplrFetchContent( "Cargo.toml".to_string());
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
    let  	result = XplrFetchContent( "__nonexistent_file__.txt".to_string());
    assert!( result.is_err());
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestGetFileInfo()
{
    let  	result = XplrLeafInfo( "Cargo.toml".to_string());
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
    let  	result = XplrLeafInfo( "src".to_string());
    assert!( result.is_ok());

    let  	info = result.unwrap();
    assert!( info.is_dir);
    assert_eq!( info.name, "src");
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestFsLeaf()
{
    use	crate::fenst::{ Xplr, FsLeaf };

    let  	leaf = FsLeaf::New( "Cargo.toml".to_string());
    assert_eq!( leaf.Name(), "Cargo.toml");
    assert_eq!( leaf.Path(), "Cargo.toml");
    assert!( leaf.IsLeaf());

    let  	asLeaf = leaf.AsLeaf();
    assert!( asLeaf.is_some());
    assert!( leaf.AsBranch().is_none());

    let  	leafRef = asLeaf.unwrap();
    assert!( leafRef.Size() > 0);
    assert_eq!( leafRef.Extension(), "toml");
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestFsBranch()
{
    use	crate::fenst::{ Xplr, FsBranch };

    let  	branch = FsBranch::New( "src".to_string());
    assert_eq!( branch.Name(), "src");
    assert_eq!( branch.Path(), "src");
    assert!( !branch.IsLeaf());

    let  	asBranch = branch.AsBranch();
    assert!( asBranch.is_some());
    assert!( branch.AsLeaf().is_none());

    let  	branchRef = asBranch.unwrap();
    let  	childCount = branchRef.ChildCount();
    assert!( childCount.is_ok());
    assert!( childCount.unwrap().0 > 0);

    let  	children = branchRef.Children();
    assert!( children.is_ok());
    let  	entries = children.unwrap();
    assert!( !entries.is_empty());

    let  	mut seenFile = false;
    for entry in &entries {
        if !entry.IsLeaf() && seenFile {
            panic!( "Directory '{}' appeared after a file in Xplr listing", entry.Name());
        }
        if entry.IsLeaf() {
            seenFile = true;
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestXplrRegistry()
{
    use	crate::fenst::XplrRegistry;

    let  	registry = XplrRegistry::New();
    let  	schemes = registry.Schemes();
    assert!( schemes.contains( &"file".to_string()));

    let  	openRes = registry.OpenRoot( "file://src");
    assert!( openRes.is_ok());

    let  	( scheme, root) = openRes.unwrap();
    assert_eq!( scheme, "file");
    assert_eq!( root.Name(), "src");

    let  	children = root.Children();
    assert!( children.is_ok());
    let  	entries = children.unwrap();
    assert!( !entries.is_empty());

    let  	dto = entries[0].ToDto( &scheme);
    assert_eq!( dto.provider, "file");
    assert!( !dto.id.is_empty());
}

#[test]
fn	TestReadFileChunk()
{
    use	crate::fenst::XplrFetchChunk;

    let  	result = XplrFetchChunk( "Cargo.toml".to_string(), 0, 30);
    assert!( result.is_ok(), "Failed to read file chunk: {:?}", result.err());

    let  	chunk = result.unwrap();
    assert_eq!( chunk.offset, 0);
    assert!( chunk.length > 0);
    assert!( chunk.total_size > 0);
    assert!( chunk.content.contains( "[package]"));
}

#[test]
fn	TestFrescoProvider()
{
    use	crate::fenst::XplrRegistry;

    let  	registry = XplrRegistry::New();
    let  	schemes = registry.Schemes();
    assert!( schemes.contains( &"expr".to_string()));

    let  	openRes = registry.OpenRoot( "expr://demo");
    assert!( openRes.is_ok());

    let  	( scheme, root) = openRes.unwrap();
    assert_eq!( scheme, "expr");
    assert_eq!( root.Name(), "demo");

    let  	children = root.Children();
    assert!( children.is_ok());
    let  	entries = children.unwrap();
    assert!( !entries.is_empty());
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestShardProvider()
{
    use	crate::fenst::XplrRegistry;

    let  	registry = XplrRegistry::New();
    let  	schemes = registry.Schemes();
    assert!( schemes.contains( &"ast".to_string()));

    let  	openRes = registry.OpenRoot( "ast://demo");
    assert!( openRes.is_ok());

    let  	( scheme, root) = openRes.unwrap();
    assert_eq!( scheme, "ast");

    let  	children = root.Children();
    assert!( children.is_ok());
    let  	entries = children.unwrap();
    assert!( !entries.is_empty());
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsFileExtensionDetection()
{
    use	crate::fenst::IsPtsFile;

    assert!( IsPtsFile( "model.pts"));
    assert!( IsPtsFile( "data/pointcloud.pts"));
    assert!( IsPtsFile( "test.pts.json"));
    assert!( !IsPtsFile( "main.rs"));
    assert!( !IsPtsFile( "Cargo.toml"));
}

// ---------------------------------------------------------------------------------------------------------------------------------

