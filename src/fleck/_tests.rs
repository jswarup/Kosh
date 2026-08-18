//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	crate::{
    fleck::ptio::{ ParsePts, ParsePtsBytes, ParsePtsStream, PtsCloud, PtsShard },
    flux::instream::FixedStream,
    shard::Parser,
    silo::{ IAccess, U32, U8 },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsBasic3D()
{
    let  	ptsData = "10.0 20.0 30.0\n40.5 -50.25 60.125\n0.0 0.0 0.0\n";
    let  	res = ParsePts( ptsData);
    assert!( res.is_ok());

    let  	cloud = res.unwrap();
    assert_eq!( cloud.Count().AsUsize(), 3);
    assert!( !cloud.IsEmpty());

    let  	arr = cloud.Points().Arr();
    let  	p0 = arr.At( U32( 0));
    assert_eq!( p0._Pos._X, 10.0);
    assert_eq!( p0._Pos._Y, 20.0);
    assert_eq!( p0._Pos._Z, 30.0);
    assert_eq!( p0._Intensity, None);
    assert_eq!( p0._Color, None);

    let  	p1 = arr.At( U32( 1));
    assert_eq!( p1._Pos._X, 40.5);
    assert_eq!( p1._Pos._Y, -50.25);
    assert_eq!( p1._Pos._Z, 60.125);

    let  	( minBbox, maxBbox) = cloud.BoundingBox();
    assert_eq!( minBbox, [0.0, -50.25, 0.0]);
    assert_eq!( maxBbox, [40.5, 20.0, 60.125]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsWithHeaderAndIntensity()
{
    let  	ptsData = "# Point cloud with header and intensity\n2\n1.0 2.0 3.0 0.75\n-4.0 -5.0 -6.0 0.25\n";
    let  	res = ParsePts( ptsData);
    assert!( res.is_ok());

    let  	cloud = res.unwrap();
    assert_eq!( cloud._HeaderCount, Some( U32( 2)));
    assert_eq!( cloud.Count().AsUsize(), 2);

    let  	arr = cloud.Points().Arr();
    let  	p0 = arr.At( U32( 0));
    assert_eq!( p0._Pos._X, 1.0);
    assert_eq!( p0._Pos._Y, 2.0);
    assert_eq!( p0._Pos._Z, 3.0);
    assert_eq!( p0._Intensity, Some( 0.75));

    let  	p1 = arr.At( U32( 1));
    assert_eq!( p1._Pos._X, -4.0);
    assert_eq!( p1._Pos._Y, -5.0);
    assert_eq!( p1._Pos._Z, -6.0);
    assert_eq!( p1._Intensity, Some( 0.25));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsWithColorRGB()
{
    let  	ptsData = "10.0 20.0 30.0 255 128 64\n0.0 0.0 0.0 0 255 0\n";
    let  	res = ParsePts( ptsData);
    assert!( res.is_ok());

    let  	cloud = res.unwrap();
    assert_eq!( cloud.Count().AsUsize(), 2);

    let  	arr = cloud.Points().Arr();
    let  	p0 = arr.At( U32( 0));
    assert_eq!( p0._Pos._X, 10.0);
    assert_eq!( p0._Pos._Y, 20.0);
    assert_eq!( p0._Pos._Z, 30.0);
    let  	color0 = p0._Color.unwrap();
    assert_eq!( color0._R, U8( 255));
    assert_eq!( color0._G, U8( 128));
    assert_eq!( color0._B, U8( 64));

    let  	p1 = arr.At( U32( 1));
    let  	color1 = p1._Color.unwrap();
    assert_eq!( color1._R, U8( 0));
    assert_eq!( color1._G, U8( 255));
    assert_eq!( color1._B, U8( 0));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsWithIntensityAndColor()
{
    let  	ptsData = "1.0 2.0 3.0 0.9 255 200 150\n";
    let  	res = ParsePts( ptsData);
    assert!( res.is_ok());

    let  	cloud = res.unwrap();
    assert_eq!( cloud.Count().AsUsize(), 1);

    let  	arr = cloud.Points().Arr();
    let  	p0 = arr.At( U32( 0));
    assert_eq!( p0._Pos._X, 1.0);
    assert_eq!( p0._Pos._Y, 2.0);
    assert_eq!( p0._Pos._Z, 3.0);
    assert_eq!( p0._Intensity, Some( 0.9));
    let  	color0 = p0._Color.unwrap();
    assert_eq!( color0._R, U8( 255));
    assert_eq!( color0._G, U8( 200));
    assert_eq!( color0._B, U8( 150));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsScientificNotationAndComments()
{
    let  	ptsData = "// Header comment\n# Another comment\n\n1.25e-2 -3.5E+1 4.0e0\n  \t  10.0   20.0   30.0  # inline comment\n";
    let  	res = ParsePts( ptsData);
    assert!( res.is_ok());

    let  	cloud = res.unwrap();
    assert_eq!( cloud.Count().AsUsize(), 2);

    let  	arr = cloud.Points().Arr();
    let  	p0 = arr.At( U32( 0));
    assert_eq!( p0._Pos._X, 0.0125);
    assert_eq!( p0._Pos._Y, -35.0);
    assert_eq!( p0._Pos._Z, 4.0);

    let  	p1 = arr.At( U32( 1));
    assert_eq!( p1._Pos._X, 10.0);
    assert_eq!( p1._Pos._Y, 20.0);
    assert_eq!( p1._Pos._Z, 30.0);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsToDtoConversion()
{
    let  	ptsData = "0.0 0.0 0.0\n100.0 200.0 300.0\n";
    let  	cloud = ParsePts( ptsData).unwrap();
    let  	dto = cloud.ToDto();

    assert_eq!( dto._Count, 2);
    assert_eq!( dto._Points.len(), 2);
    assert_eq!( dto._Points[0], [0.0, 0.0, 0.0]);
    assert_eq!( dto._Points[1], [100.0, 200.0, 300.0]);
    assert_eq!( dto._BboxMin, [0.0, 0.0, 0.0]);
    assert_eq!( dto._BboxMax, [100.0, 200.0, 300.0]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsParseBytesAndStream()
{
    let  	bytes = b"10.0 20.0 30.0\n";
    let  	cloudFromBytes = ParsePtsBytes( bytes).unwrap();
    assert_eq!( cloudFromBytes.Count().AsUsize(), 1);

    let  	mut stream = FixedStream::from( "10.0 20.0 30.0\n");
    let  	cloudFromStream = ParsePtsStream( &mut stream).unwrap();
    assert_eq!( cloudFromStream.Count().AsUsize(), 1);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsShardGrammarDirect()
{
    let  	ptsData = "5.0 15.0 25.0\n";
    let  	mut stream = FixedStream::from( ptsData);
    let  	mut parser = Parser::New( &mut stream);
    let  	mut cloud = PtsCloud::New();
    let  	shard = PtsShard { _Cloud: &mut cloud };

    let  	res = parser.ParseGrammar( &shard, U32( 0));
    assert!( res.is_some());
    assert_eq!( cloud.Count().AsUsize(), 1);

    let  	arr = cloud.Points().Arr();
    let  	p0 = arr.At( U32( 0));
    assert_eq!( p0._Pos._X, 5.0);
    assert_eq!( p0._Pos._Y, 15.0);
    assert_eq!( p0._Pos._Z, 25.0);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWaveObjBasicCube()
{
    use	crate::fleck::ParseWaveObj;

    let  	objData = r#"
# Wavefront OBJ Cube
o Cube
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 1.0 1.0 0.0
v 0.0 1.0 0.0
v 0.0 0.0 1.0
v 1.0 0.0 1.0
v 1.0 1.0 1.0
v 0.0 1.0 1.0

# 6 quad faces
f 1 2 3 4
f 5 6 7 8
f 1 2 6 5
f 2 3 7 6
f 3 4 8 7
f 4 1 5 8
"#;

    let  	res = ParseWaveObj( objData);
    assert!( res.is_ok());

    let  	model = res.unwrap();
    assert_eq!( model.VertexCount().AsUsize(), 8);
    assert_eq!( model.FaceCount().AsUsize(), 6);
    assert_eq!( model._Objects.Size().AsUsize(), 1);
    assert_eq!( model._Objects.Arr().At( U32( 0)), &"Cube".to_string());

    let  	( bMin, bMax) = model.BoundingBox();
    assert_eq!( bMin, [0.0, 0.0, 0.0]);
    assert_eq!( bMax, [1.0, 1.0, 1.0]);

    let  	triangles = model.Triangulate();
    assert_eq!( triangles.Size().AsUsize(), 12); // 6 quads = 12 triangles
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWaveObjWithNormalsAndTexCoords()
{
    use	crate::fleck::ParseWaveObj;

    let  	objData = r#"
# Vertices, TexCoords, Normals, and Faces with v/vt/vn
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0

vt 0.0 0.0
vt 1.0 0.0
vt 0.0 1.0

vn 0.0 0.0 1.0

f 1/1/1 2/2/1 3/3/1
"#;

    let  	res = ParseWaveObj( objData);
    assert!( res.is_ok());

    let  	model = res.unwrap();
    assert_eq!( model.VertexCount().AsUsize(), 3);
    assert_eq!( model.TexCoordCount().AsUsize(), 3);
    assert_eq!( model.NormalCount().AsUsize(), 1);
    assert_eq!( model.FaceCount().AsUsize(), 1);

    let  	facesArr = model._Faces.Arr();
    let  	face0 = facesArr.At( U32( 0));
    assert_eq!( face0.Len(), 3);

    let  	v0 = face0._Vertices.Arr().At( U32( 0));
    assert_eq!( v0._VertexIdx, 1);
    assert_eq!( v0._TexCoordIdx, Some( 1));
    assert_eq!( v0._NormalIdx, Some( 1));

    let  	v1 = face0._Vertices.Arr().At( U32( 1));
    assert_eq!( v1._VertexIdx, 2);
    assert_eq!( v1._TexCoordIdx, Some( 2));
    assert_eq!( v1._NormalIdx, Some( 1));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWaveObjFaceFormats()
{
    use	crate::fleck::ParseWaveObj;

    let  	objData = r#"
v 0.0 0.0 0.0
v 1.0 0.0 0.0
v 0.0 1.0 0.0
v 1.0 1.0 0.0

vt 0.0 0.0
vt 1.0 1.0

vn 0.0 0.0 1.0

# v only
f 1 2 3
# v/vt
f 1/1 2/2 3/1
# v//vn
f 1//1 2//1 3//1
# v/vt/vn
f 1/1/1 2/2/1 4/2/1 3/1/1
"#;

    let  	model = ParseWaveObj( objData).unwrap();
    assert_eq!( model.FaceCount().AsUsize(), 4);

    let  	faces = model._Faces.Arr();

    // Face 0: v only
    let  	f0 = faces.At( U32( 0));
    assert_eq!( f0._Vertices.Arr().At( U32( 0))._VertexIdx, 1);
    assert_eq!( f0._Vertices.Arr().At( U32( 0))._TexCoordIdx, None);
    assert_eq!( f0._Vertices.Arr().At( U32( 0))._NormalIdx, None);

    // Face 1: v/vt
    let  	f1 = faces.At( U32( 1));
    assert_eq!( f1._Vertices.Arr().At( U32( 0))._VertexIdx, 1);
    assert_eq!( f1._Vertices.Arr().At( U32( 0))._TexCoordIdx, Some( 1));
    assert_eq!( f1._Vertices.Arr().At( U32( 0))._NormalIdx, None);

    // Face 2: v//vn
    let  	f2 = faces.At( U32( 2));
    assert_eq!( f2._Vertices.Arr().At( U32( 0))._VertexIdx, 1);
    assert_eq!( f2._Vertices.Arr().At( U32( 0))._TexCoordIdx, None);
    assert_eq!( f2._Vertices.Arr().At( U32( 0))._NormalIdx, Some( 1));

    // Face 3: quad v/vt/vn
    let  	f3 = faces.At( U32( 3));
    assert_eq!( f3.Len(), 4);
    assert_eq!( f3._Vertices.Arr().At( U32( 3))._VertexIdx, 3);
    assert_eq!( f3._Vertices.Arr().At( U32( 3))._TexCoordIdx, Some( 1));
    assert_eq!( f3._Vertices.Arr().At( U32( 3))._NormalIdx, Some( 1));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWaveObjNegativeIndexing()
{
    use	crate::fleck::ParseWaveObj;

    let  	objData = r#"
v 10.0 20.0 30.0
v 40.0 50.0 60.0
v 70.0 80.0 90.0
f -3 -2 -1
"#;

    let  	model = ParseWaveObj( objData).unwrap();
    assert_eq!( model.VertexCount().AsUsize(), 3);
    assert_eq!( model.FaceCount().AsUsize(), 1);

    let  	f0 = model._Faces.Arr().At( U32( 0));
    assert_eq!( f0._Vertices.Arr().At( U32( 0))._VertexIdx, 1);
    assert_eq!( f0._Vertices.Arr().At( U32( 1))._VertexIdx, 2);
    assert_eq!( f0._Vertices.Arr().At( U32( 2))._VertexIdx, 3);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWaveObjToDtoAndTriangulate()
{
    use	crate::fleck::{ ParseWaveObjBytes, ParseWaveObjStream };
    use	crate::flux::instream::FixedStream;

    let  	objData = b"v 0.0 0.0 0.0\nv 10.0 0.0 0.0\nv 10.0 10.0 0.0\nv 0.0 10.0 0.0\nf 1 2 3 4\n";
    let  	model = ParseWaveObjBytes( objData).unwrap();

    let  	dto = model.ToDto();
    assert_eq!( dto._Count, 4);
    assert_eq!( dto._BboxMin, [0.0, 0.0, 0.0]);
    assert_eq!( dto._BboxMax, [10.0, 10.0, 0.0]);

    let  	triangles = model.Triangulate();
    assert_eq!( triangles.Size().AsUsize(), 2);

    let  	mut stream = FixedStream::from( "v 5.0 5.0 5.0\n");
    let  	streamModel = ParseWaveObjStream( &mut stream).unwrap();
    assert_eq!( streamModel.VertexCount().AsUsize(), 1);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWaveObjMetadataAndMaterials()
{
    use	crate::fleck::ParseWaveObj;

    let  	objData = r#"
mtllib materials.mtl
o MainModel
g GroupA
usemtl Metal_Shiny
v 1.0 2.0 3.0
v 4.0 5.0 6.0
v 7.0 8.0 9.0
f 1 2 3
"#;

    let  	model = ParseWaveObj( objData).unwrap();
    assert_eq!( model._MtlLibs.Size().AsUsize(), 1);
    assert_eq!( model._MtlLibs.Arr().At( U32( 0)), &"materials.mtl".to_string());
    assert_eq!( model._Objects.Arr().At( U32( 0)), &"MainModel".to_string());
    assert_eq!( model._Groups.Arr().At( U32( 0)), &"GroupA".to_string());
    assert_eq!( model._UseMtls.Arr().At( U32( 0)), &"Metal_Shiny".to_string());
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPt3fBasicOps()
{
    use	crate::fleck::Pt3f;

    let  	pt = Pt3f::New( 1.5, 2.5, 3.5);
    assert_eq!( pt._X, 1.5);
    assert_eq!( pt._Y, 2.5);
    assert_eq!( pt._Z, 3.5);
    assert_eq!( pt.Pos(), [1.5, 2.5, 3.5]);

    let  	defaultPt = Pt3f::default();
    assert_eq!( defaultPt.Pos(), [0.0, 0.0, 0.0]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPt4fBasicOps()
{
    use	crate::fleck::Pt4f;

    let  	pt = Pt4f::New( 1.0, 2.0, 3.0);
    assert_eq!( pt._X, 1.0);
    assert_eq!( pt._Y, 2.0);
    assert_eq!( pt._Z, 3.0);
    assert_eq!( pt._W, 1.0);
    assert_eq!( pt.Pos(), [1.0, 2.0, 3.0]);

    let  	ptW = Pt4f::WithW( 4.0, 5.0, 6.0, 2.0);
    assert_eq!( ptW._W, 2.0);

    let  	defaultPt = Pt4f::default();
    assert_eq!( defaultPt.Pos(), [0.0, 0.0, 0.0]);
    assert_eq!( defaultPt._W, 0.0);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestDir3fBasicOps()
{
    use	crate::fleck::Dir3f;

    let  	dir = Dir3f::New( 0.0, 0.0, 1.0);
    assert_eq!( dir._X, 0.0);
    assert_eq!( dir._Y, 0.0);
    assert_eq!( dir._Z, 1.0);
    assert_eq!( dir.Vec(), [0.0, 0.0, 1.0]);

    let  	defaultDir = Dir3f::default();
    assert_eq!( defaultDir.Vec(), [0.0, 0.0, 0.0]);
}

//---------------------------------------------------------------------------------------------------------------------------------
