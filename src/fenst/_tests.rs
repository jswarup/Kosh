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
        if !entry._IsDir {
            seenFile = true;
        }
        if entry._IsDir && seenFile {
            panic!( "Directory '{}' appeared after a file — sort order broken", entry._Name);
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
    assert!( contents._Size > 0);
    assert!( contents._LineCount > 0);
    assert!( contents._Content.contains( "[package]"));
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
    assert_eq!( info._Name, "Cargo.toml");
    assert!( !info._IsDir);
    assert!( info._Size > 0);
    assert_eq!( info._Extension, "toml");
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestGetFileInfoDirectory()
{
    let  	result = XplrLeafInfo( "src".to_string());
    assert!( result.is_ok());

    let  	info = result.unwrap();
    assert!( info._IsDir);
    assert_eq!( info._Name, "src");
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
    assert_eq!( dto._Provider, "file");
    assert!( !dto._Id.is_empty());
}

#[test]
fn	TestReadFileChunk()
{
    use	crate::fenst::XplrFetchChunk;

    let  	result = XplrFetchChunk( "Cargo.toml".to_string(), 0, 30);
    assert!( result.is_ok(), "Failed to read file chunk: {:?}", result.err());

    let  	chunk = result.unwrap();
    assert_eq!( chunk._Offset, 0);
    assert!( chunk._Length > 0);
    assert!( chunk._TotalSize > 0);
    assert!( chunk._Content.contains( "[package]"));
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
fn	TestParsePtsFileStream()
{
    use	crate::fenst::XplrParsePtsFile;
    use	std::fs::File;
    use	std::io::Write;

    let  	tempPath = std::env::temp_dir().join( "test_stream_cloud.pts");
    {
        let  	mut f = File::create( &tempPath).unwrap();
        writeln!( f, "3").unwrap();
        writeln!( f, "1.0 2.0 3.0").unwrap();
        writeln!( f, "4.0 5.0 6.0 100.0").unwrap();
        writeln!( f, "7.0 8.0 9.0 255 128 64").unwrap();
    }

    let  	result = XplrParsePtsFile( tempPath.to_str().unwrap());
    assert!( result.is_ok(), "Failed to parse pts file stream: {:?}", result.err());

    let  	dto = result.unwrap();
    assert_eq!( dto._Count, 3);
    assert_eq!( dto._Points.len(), 3);
    assert_eq!( dto._Points[0], [1.0, 2.0, 3.0]);
    assert_eq!( dto._Points[1], [4.0, 5.0, 6.0]);
    assert_eq!( dto._Points[2], [7.0, 8.0, 9.0]);
    assert_eq!( dto._BboxMin, [1.0, 2.0, 3.0]);
    assert_eq!( dto._BboxMax, [7.0, 8.0, 9.0]);

    let  	_ = std::fs::remove_file( &tempPath);
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestCameraOperations()
{
    use	crate::fenst::Camera;

    let  	mut cam = Camera::New();
    assert_eq!( cam._PanX, 0.0);
    assert_eq!( cam._PanY, 0.0);
    assert_eq!( cam._Zoom, 1.0);

    cam.Pan( 50.0, -30.0);
    assert_eq!( cam._PanX, 50.0);
    assert_eq!( cam._PanY, -30.0);

    cam.Zoom( 1.5);
    assert_eq!( cam._Zoom, 1.5);

    cam.Rotate( 0.1, 0.2);
    assert!( ( cam._RotX - 0.5).abs() < 1e-5);
    assert!( ( cam._RotY - 0.8).abs() < 1e-5);

    let  	( px, py, pz) = cam.Project( 0.0, 0.0, 0.0, 800.0, 600.0);
    assert!( px > 0.0 && px < 800.0);
    assert!( py > 0.0 && py < 600.0);
    assert_eq!( pz, 0.0);

    cam.Reset();
    assert_eq!( cam._PanX, 0.0);
    assert_eq!( cam._PanY, 0.0);
    assert_eq!( cam._Zoom, 1.0);
    assert_eq!( cam._RotX, 0.4);
    assert_eq!( cam._RotY, 0.6);
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSceneGraph()
{
    use	crate::fenst::SceneGraph;
    use	crate::silo::Buff;

    let  	mut points = Buff::New();
    points.Push( [ -10.0, -10.0, -10.0 ]);
    points.Push( [ 10.0, 10.0, 10.0 ]);

    let  	mut scene = SceneGraph::WithPoints( points, [ -10.0, -10.0, -10.0 ], [ 10.0, 10.0, 10.0 ]);
    assert_eq!( scene._Points.len(), 2);

    let  	lines = scene.ProjectBoundingBox( 800.0, 600.0);
    assert_eq!( lines.len(), 12);

    scene.CameraMut().Pan( 10.0, 20.0);
    scene.CameraMut().Zoom( 2.0);
    assert_eq!( scene.Camera()._PanX, 10.0);
    assert_eq!( scene.Camera()._Zoom, 2.0);
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSceneGraphSwarmProjection()
{
    use	crate::fenst::SceneGraph;
    use	crate::silo::Buff;
    use	crate::swarm::SwarmEngine;

    let  	mut points = Buff::New();
    points.Push( [ 0.0, 0.0, 0.0 ]);
    points.Push( [ 5.0, 5.0, 5.0 ]);

    let  	scene = SceneGraph::WithPoints( points, [ -10.0, -10.0, -10.0 ], [ 10.0, 10.0, 10.0 ]);
    let  	engine = SwarmEngine::Auto();

    let  	res = scene.ProjectPointsSwarm( &engine, 800.0, 600.0, 1.0, 0, 243, 255, None);
    assert!( res.is_ok());

    let  	projPoints = res.unwrap();
    assert_eq!( projPoints.len(), 2);
    assert!( projPoints[0].0 > 0.0 && projPoints[0].0 < 800.0);
    assert!( projPoints[0].1 > 0.0 && projPoints[0].1 < 600.0);
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSceneGraphSwarmBoundingBoxProjection()
{
    use	crate::fenst::SceneGraph;
    use	crate::silo::Buff;
    use	crate::swarm::SwarmEngine;

    let  	points = Buff::New();
    let  	scene = SceneGraph::WithPoints( points, [ -10.0, -10.0, -10.0 ], [ 10.0, 10.0, 10.0 ]);
    let  	engine = SwarmEngine::Auto();

    let  	res = scene.ProjectBoundingBoxSwarm( &engine, 800.0, 600.0, None);
    assert!( res.is_ok());

    let  	lines = res.unwrap();
    assert_eq!( lines.len(), 12);

    let  	cpuLines = scene.ProjectBoundingBox( 800.0, 600.0);
    assert_eq!( cpuLines.len(), 12);
    for i in 0..12 {
        assert!( ( lines[i].0.0 - cpuLines[i].0.0).abs() < 1e-2);
        assert!( ( lines[i].0.1 - cpuLines[i].0.1).abs() < 1e-2);
        assert!( ( lines[i].1.0 - cpuLines[i].1.0).abs() < 1e-2);
        assert!( ( lines[i].1.1 - cpuLines[i].1.1).abs() < 1e-2);
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSceneGraphSwarmFullSceneProjection()
{
    use	crate::fenst::SceneGraph;
    use	crate::silo::Buff;
    use	crate::swarm::SwarmEngine;

    let  	mut points = Buff::New();
    points.Push( [ 0.0, 0.0, 0.0 ]);
    points.Push( [ 10.0, 10.0, 10.0 ]);

    let  	scene = SceneGraph::WithPoints( points, [ -20.0, -20.0, -20.0 ], [ 20.0, 20.0, 20.0 ]);
    let  	engine = SwarmEngine::Auto();

    let  	res = scene.ProjectSceneSwarm( &engine, 800.0, 600.0, 1.0, 0, 243, 255, None);
    assert!( res.is_ok());

    let  	frame = res.unwrap();
    assert_eq!( frame._Points.len(), 2);
    assert_eq!( frame._BoxLines.len(), 12);
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestSceneGraphClusterMultiGpuProjection()
{
    use	crate::fenst::SceneGraph;
    use	crate::silo::Buff;
    use	crate::swarm::SwarmCluster;

    let  	mut points = Buff::New();
    for i in 0..100 {
        let  	f = i as f32;
        points.Push( [ f, f * 0.5, f * 0.2 ]);
    }

    let  	scene = SceneGraph::WithPoints( points, [ 0.0, 0.0, 0.0 ], [ 100.0, 50.0, 20.0 ]);
    let  	cluster = SwarmCluster::Auto();

    let  	res = scene.ProjectSceneCluster( &cluster, 800.0, 600.0, 1.0, 0, 243, 255, None);
    assert!( res.is_ok());

    let  	frame = res.unwrap();
    assert_eq!( frame._Points.len(), 100);
    assert_eq!( frame._BoxLines.len(), 12);
}

// ---------------------------------------------------------------------------------------------------------------------------------



