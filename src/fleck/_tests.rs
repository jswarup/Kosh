//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	crate::{
    fleck::{ BBox3f, Pt3f, ptio::{ ParsePts, ParsePtsBytes, ParsePtsStream, PtsCloud, PtsShard } },
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
fn	TestWPt3fBasicOps()
{
    use	crate::fleck::WPt3f;

    let  	pt = WPt3f::New( 1.0, 2.0, 3.0);
    assert_eq!( pt._X, 1.0);
    assert_eq!( pt._Y, 2.0);
    assert_eq!( pt._Z, 3.0);
    assert_eq!( pt._W, 1.0);
    assert_eq!( pt.Pos(), [1.0, 2.0, 3.0]);

    let  	ptW = WPt3f::WithW( 4.0, 5.0, 6.0, 2.0);
    assert_eq!( ptW._W, 2.0);

    let  	defaultPt = WPt3f::default();
    assert_eq!( defaultPt.Pos(), [0.0, 0.0, 0.0]);
    assert_eq!( defaultPt._W, 0.0);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestWPt2fBasicOps()
{
    use	crate::fleck::WPt2f;

    let  	pt = WPt2f::New( 0.25, 0.75);
    assert_eq!( pt._U, 0.25);
    assert_eq!( pt._V, 0.75);
    assert_eq!( pt._W, 0.0);
    assert_eq!( pt.Pos(), [0.25, 0.75]);

    let  	ptW = WPt2f::WithW( 0.1, 0.2, 0.3);
    assert_eq!( ptW._W, 0.3);

    let  	defaultPt = WPt2f::default();
    assert_eq!( defaultPt.Pos(), [0.0, 0.0]);
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


//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestVexBasicConstructorsAndAccessors()
{
    use	crate::fleck::vex::{ Vex2f, Vex3d, Vex3f, Vex3i, Vex4f };

    let  	v2 = Vex2f::New2( 3.0, 4.0);
    assert_eq!( v2.X(), 3.0);
    assert_eq!( v2.Y(), 4.0);
    assert_eq!( v2._Data, [3.0, 4.0]);

    let  	mut v3 = Vex3f::New3( 1.0, 2.0, 3.0);
    assert_eq!( v3.X(), 1.0);
    assert_eq!( v3.Y(), 2.0);
    assert_eq!( v3.Z(), 3.0);
    v3.SetX( 10.0);
    v3.SetY( 20.0);
    v3.SetZ( 30.0);
    assert_eq!( v3.AsArray(), &[10.0, 20.0, 30.0]);

    let  	v4 = Vex4f::New4( 1.0, 2.0, 3.0, 4.0);
    assert_eq!( v4.W(), 4.0);

    let  	splat = Vex3f::Splat( 5.0);
    assert_eq!( splat._Data, [5.0, 5.0, 5.0]);

    let  	mapped = v2.Map( |c| c * 2.0);
    assert_eq!( mapped._Data, [6.0, 8.0]);

    let  	zipped = v2.ZipMap( &mapped, |a, b| a + b);
    assert_eq!( zipped._Data, [9.0, 12.0]);

    let  	vInt = Vex3i::New3( 10, -20, 30);
    assert_eq!( vInt.X(), 10);
    assert_eq!( vInt.Y(), -20);
    assert_eq!( vInt.Z(), 30);

    let  	vDouble = Vex3d::New3( 1.5, 2.5, 3.5);
    assert_eq!( vDouble.X(), 1.5);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestVexByValueAndByRefOperators()
{
    use	crate::fleck::vex::Vex3f;

    let  	a = Vex3f::New3( 1.0, 2.0, 3.0);
    let  	b = Vex3f::New3( 4.0, 5.0, 6.0);

    // Val + Val
    let  	sumValVal = a + b;
    assert_eq!( sumValVal._Data, [5.0, 7.0, 9.0]);

    // Ref + Ref
    let  	sumRefRef = &a + &b;
    assert_eq!( sumRefRef._Data, [5.0, 7.0, 9.0]);

    // Ref + Val
    let  	sumRefVal = &a + b;
    assert_eq!( sumRefVal._Data, [5.0, 7.0, 9.0]);

    // Val + Ref
    let  	sumValRef = a + &b;
    assert_eq!( sumValRef._Data, [5.0, 7.0, 9.0]);

    // Subtraction
    let  	diffRefRef = &b - &a;
    assert_eq!( diffRefRef._Data, [3.0, 3.0, 3.0]);
    let  	diffValVal = b - a;
    assert_eq!( diffValVal._Data, [3.0, 3.0, 3.0]);

    // Negation
    let  	negVal = -a;
    assert_eq!( negVal._Data, [-1.0, -2.0, -3.0]);
    let  	negRef = -&a;
    assert_eq!( negRef._Data, [-1.0, -2.0, -3.0]);

    // Hadamard Multiplication
    let  	hadamard = &a * &b;
    assert_eq!( hadamard._Data, [4.0, 10.0, 18.0]);

    // Component-wise Division
    let  	quotient = &b / &a;
    assert_eq!( quotient._Data, [4.0, 2.5, 2.0]);

    // Compound Assignments
    let  	mut acc = a;
    acc += &b;
    assert_eq!( acc._Data, [5.0, 7.0, 9.0]);
    acc -= &b;
    assert_eq!( acc._Data, [1.0, 2.0, 3.0]);
    acc *= &b;
    assert_eq!( acc._Data, [4.0, 10.0, 18.0]);
    acc /= &b;
    assert_eq!( acc._Data, [1.0, 2.0, 3.0]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestVexScalarArithmetic()
{
    use	crate::fleck::vex::{ Vex3d, Vex3f, Vex3i };

    let  	vf = Vex3f::New3( 2.0, 3.0, 4.0);
    let  	sf = 2.5f32;

    // Vector * Scalar
    assert_eq!( ( vf * sf)._Data, [5.0, 7.5, 10.0]);
    assert_eq!( ( &vf * sf)._Data, [5.0, 7.5, 10.0]);
    assert_eq!( ( &vf * &sf)._Data, [5.0, 7.5, 10.0]);
    assert_eq!( ( vf * &sf)._Data, [5.0, 7.5, 10.0]);

    // Scalar * Vector
    assert_eq!( ( sf * vf)._Data, [5.0, 7.5, 10.0]);
    assert_eq!( ( &sf * &vf)._Data, [5.0, 7.5, 10.0]);
    assert_eq!( ( &sf * vf)._Data, [5.0, 7.5, 10.0]);
    assert_eq!( ( sf * &vf)._Data, [5.0, 7.5, 10.0]);

    // Vector / Scalar
    assert_eq!( ( vf / 2.0f32)._Data, [1.0, 1.5, 2.0]);
    assert_eq!( ( &vf / 2.0f32)._Data, [1.0, 1.5, 2.0]);
    assert_eq!( ( &vf / &2.0f32)._Data, [1.0, 1.5, 2.0]);

    // Compound Scalar Assignments
    let  	mut mutV = vf;
    mutV *= 2.0f32;
    assert_eq!( mutV._Data, [4.0, 6.0, 8.0]);
    mutV /= 2.0f32;
    assert_eq!( mutV._Data, [2.0, 3.0, 4.0]);

    // Integer Scalar Multiplication
    let  	vi = Vex3i::New3( 1, -2, 3);
    assert_eq!( ( vi * 3i32)._Data, [3, -6, 9]);
    assert_eq!( ( 3i32 * vi)._Data, [3, -6, 9]);

    // Double Scalar Multiplication
    let  	vd = Vex3d::New3( 1.0, 2.0, 3.0);
    assert_eq!( ( vd * 0.5f64)._Data, [0.5, 1.0, 1.5]);
    assert_eq!( ( 0.5f64 * vd)._Data, [0.5, 1.0, 1.5]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestVexVectorSpaceAndInnerProduct()
{
    use	crate::fleck::vex::{ Cross, Dot, ICrossProduct, IInnerProductSpace, IVectorSpace, Lerp, Vex3f };

    let  	zero = Vex3f::Zero();
    assert!( zero.IsZero());

    let  	u = Vex3f::New3( 1.0, 0.0, 0.0);
    let  	v = Vex3f::New3( 0.0, 1.0, 0.0);
    assert!( !u.IsZero());

    // Dot product
    assert_eq!( u.Dot( &v), 0.0);
    assert_eq!( Dot( &u, &v), 0.0);

    let  	w = Vex3f::New3( 3.0, 4.0, 0.0);
    assert_eq!( w.MagnitudeSquared(), 25.0);
    assert_eq!( w.Magnitude(), 5.0);

    // Normalization
    let  	normW = w.Normalized().unwrap();
    assert_eq!( normW._Data, [0.6, 0.8, 0.0]);
    assert_eq!( normW.Magnitude(), 1.0);

    // Distance
    let  	p1 = Vex3f::New3( 1.0, 2.0, 3.0);
    let  	p2 = Vex3f::New3( 4.0, 6.0, 3.0);
    assert_eq!( p1.DistanceSquared( &p2), 25.0);
    assert_eq!( p1.Distance( &p2), 5.0);

    // Angle
    let  	angleRad = u.Angle( &v);
    let  	piOver2 = std::f32::consts::FRAC_PI_2;
    assert!( ( angleRad - piOver2).abs() < 1e-5);

    // Projection and Rejection
    let  	vecA = Vex3f::New3( 3.0, 3.0, 0.0);
    let  	vecB = Vex3f::New3( 5.0, 0.0, 0.0);
    let  	proj = vecA.Project( &vecB).unwrap();
    assert_eq!( proj._Data, [3.0, 0.0, 0.0]);

    let  	reject = vecA.Reject( &vecB).unwrap();
    assert_eq!( reject._Data, [0.0, 3.0, 0.0]);

    // Reflection (ray hitting a horizontal surface facing up)
    let  	inRay = Vex3f::New3( 1.0, -1.0, 0.0);
    let  	normal = Vex3f::New3( 0.0, 1.0, 0.0);
    let  	reflected = inRay.Reflect( &normal);
    assert_eq!( reflected._Data, [1.0, 1.0, 0.0]);

    // Linear Interpolation (Lerp)
    let  	start = Vex3f::New3( 0.0, 0.0, 0.0);
    let  	end = Vex3f::New3( 10.0, 20.0, 30.0);
    let  	mid = start.Lerp( &end, 0.5);
    assert_eq!( mid._Data, [5.0, 10.0, 15.0]);
    assert_eq!( Lerp( &start, &end, 0.5)._Data, [5.0, 10.0, 15.0]);

    // Cross Product
    let  	cross = u.Cross( &v);
    assert_eq!( cross._Data, [0.0, 0.0, 1.0]);
    assert_eq!( Cross( &u, &v)._Data, [0.0, 0.0, 1.0]);

    // Orthogonality of cross product
    let  	aRnd = Vex3f::New3( 2.5, -3.1, 4.2);
    let  	bRnd = Vex3f::New3( 1.1, 7.8, -0.9);
    let  	cRnd = aRnd.Cross( &bRnd);
    assert!( cRnd.Dot( &aRnd).abs() < 1e-4);
    assert!( cRnd.Dot( &bRnd).abs() < 1e-4);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestVexIndexingAndConversionInterop()
{
    use	crate::fleck::{ Dir3f, Pt3f, WPt2f, WPt3f };
    use	crate::fleck::vex::{ Vex2f, Vex3f, Vex4f };
    use	crate::silo::{ U32, U64 };

    let  	mut v = Vex3f::New3( 10.0, 20.0, 30.0);

    // Indexing
    assert_eq!( v[0], 10.0);
    assert_eq!( v[1], 20.0);
    assert_eq!( v[2], 30.0);

    v[U32( 1)] = 99.0;
    assert_eq!( v[1], 99.0);

    v[U64( 2)] = 123.0;
    assert_eq!( v[2], 123.0);

    // From / Into tuples
    let  	v2: Vex2f = ( 1.0, 2.0).into();
    assert_eq!( v2._Data, [1.0, 2.0]);

    let  	v3: Vex3f = ( 1.0, 2.0, 3.0).into();
    assert_eq!( v3._Data, [1.0, 2.0, 3.0]);

    let  	v4: Vex4f = ( 1.0, 2.0, 3.0, 4.0).into();
    assert_eq!( v4._Data, [1.0, 2.0, 3.0, 4.0]);

    // Iterators
    let  	collected: Vec< f32> = v.into_iter().collect();
    assert_eq!( collected, vec![10.0, 99.0, 123.0]);

    // Pt3f Interop
    let  	pt = Pt3f::New( 1.0, 2.0, 3.0);
    let  	vPt: Vex3f = pt.into();
    assert_eq!( vPt._Data, [1.0, 2.0, 3.0]);
    let  	backPt: Pt3f = vPt.into();
    assert_eq!( backPt, pt);

    // Dir3f Interop
    let  	dir = Dir3f::New( 0.0, 1.0, 0.0);
    let  	vDir: Vex3f = dir.into();
    assert_eq!( vDir._Data, [0.0, 1.0, 0.0]);
    let  	backDir: Dir3f = vDir.into();
    assert_eq!( backDir, dir);

    // WPt3f Interop
    let  	wpt3 = WPt3f::WithW( 1.0, 2.0, 3.0, 0.5);
    let  	vWpt3: Vex4f = wpt3.into();
    assert_eq!( vWpt3._Data, [1.0, 2.0, 3.0, 0.5]);
    let  	backWpt3: WPt3f = vWpt3.into();
    assert_eq!( backWpt3, wpt3);

    // WPt2f Interop
    let  	wpt2 = WPt2f::New( 4.0, 5.0);
    let  	vWpt2: Vex2f = wpt2.into();
    assert_eq!( vWpt2._Data, [4.0, 5.0]);
    let  	backWpt2: WPt2f = vWpt2.into();
    assert_eq!( backWpt2, wpt2);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestVexWithUIntScalarTypes()
{
    use	crate::fleck::vex::{ Dot, IScalar, Vex };
    use	crate::silo::{ U16, U32, U64, U8 };

    // Test IScalar methods on UInt types
    assert_eq!( U32::ZERO, U32( 0));
    assert_eq!( U32::ONE, U32( 1));
    assert_eq!( U32( 16).Sqrt(), U32( 4));
    assert_eq!( U32( 42).Abs(), U32( 42));
    assert_eq!( U32::FromF32( 12.75), U32( 12));
    assert_eq!( U32( 100).ToF64(), 100.0);

    // Test Vex< U32, 3> construction and algebra
    let  	v1 = Vex::< U32, 3>::New( [ U32( 10), U32( 20), U32( 30) ]);
    let  	v2 = Vex::< U32, 3>::New( [ U32( 1), U32( 2), U32( 3) ]);

    let  	sum = v1 + v2;
    assert_eq!( sum._Data, [ U32( 11), U32( 22), U32( 33) ]);

    let  	diff = v1 - v2;
    assert_eq!( diff._Data, [ U32( 9), U32( 18), U32( 27) ]);

    let  	scaled = v1 * U32( 2);
    assert_eq!( scaled._Data, [ U32( 20), U32( 40), U32( 60) ]);

    let  	dot = Dot( &v1, &v2);
    assert_eq!( dot, U32( 10 * 1 + 20 * 2 + 30 * 3));

    // Test Vex< U8, 4>
    let  	vU8_1 = Vex::< U8, 4>::New( [ U8( 10), U8( 20), U8( 30), U8( 40) ]);
    let  	vU8_2 = Vex::< U8, 4>::New( [ U8( 5), U8( 10), U8( 15), U8( 20) ]);
    let  	sumU8 = vU8_1 + vU8_2;
    assert_eq!( sumU8._Data, [ U8( 15), U8( 30), U8( 45), U8( 60) ]);

    // Test Vex< U16, 2>
    let  	vU16_1 = Vex::< U16, 2>::New( [ U16( 300), U16( 400) ]);
    let  	vU16_2 = Vex::< U16, 2>::New( [ U16( 100), U16( 200) ]);
    let  	diffU16 = vU16_1 - vU16_2;
    assert_eq!( diffU16._Data, [ U16( 200), U16( 200) ]);

    // Test Vex< U64, 2>
    let  	vU64_1 = Vex::< U64, 2>::New( [ U64( 100), U64( 200) ]);
    let  	vU64_2 = Vex::< U64, 2>::New( [ U64( 50), U64( 25) ]);
    let  	dotU64 = Dot( &vU64_1, &vU64_2);
    assert_eq!( dotU64, U64( 100 * 50 + 200 * 25));
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBuffVectorSpaceAndInnerProduct()
{
    use	crate::fleck::vex::{ IInnerProductSpace, IVectorSpace };
    use	crate::silo::{ Buff, U32, U8 };

    // 1. Float Vector Space (4D Euclidean space)
    let  	b1 = Buff![ 1.0f32, 2.0, 3.0, 4.0 ];
    let  	b2 = Buff![ 5.0f32, 6.0, 7.0, 8.0 ];

    assert_eq!( b1.Dim(), 4);
    assert!( !b1.IsZero());
    assert!( Buff::< f32>::Zero().IsZero());

    // Addition & Subtraction
    let  	sum = &b1 + &b2;
    assert_eq!( sum, Buff![ 6.0f32, 8.0, 10.0, 12.0 ]);

    let  	diff = &b2 - &b1;
    assert_eq!( diff, Buff![ 4.0f32, 4.0, 4.0, 4.0 ]);

    // Negation
    let  	negB1 = -&b1;
    assert_eq!( negB1, Buff![ -1.0f32, -2.0, -3.0, -4.0 ]);

    // Scalar multiplication & division
    let  	scaled = &b1 * 2.0f32;
    assert_eq!( scaled, Buff![ 2.0f32, 4.0, 6.0, 8.0 ]);

    let  	divScaled = &scaled / 2.0f32;
    assert_eq!( divScaled, b1);

    // In-place operators
    let  	mut bMut = b1.clone();
    bMut += &b2;
    assert_eq!( bMut, Buff![ 6.0f32, 8.0, 10.0, 12.0 ]);
    bMut *= 0.5f32;
    assert_eq!( bMut, Buff![ 3.0f32, 4.0, 5.0, 6.0 ]);

    // Inner product (Dot product)
    // 1*5 + 2*6 + 3*7 + 4*8 = 5 + 12 + 21 + 32 = 70
    let  	dot = b1.Dot( &b2);
    assert_eq!( dot, 70.0f32);

    // Magnitude & Normalization (3D: 3, 4, 0 -> mag = 5)
    let  	b3d = Buff![ 3.0f32, 4.0, 0.0 ];
    assert_eq!( b3d.MagnitudeSquared(), 25.0f32);
    assert_eq!( b3d.Magnitude(), 5.0f32);

    let  	norm = b3d.Normalized().unwrap();
    assert_eq!( norm, Buff![ 0.6f32, 0.8, 0.0 ]);
    assert!( ( norm.Magnitude() - 1.0f32).abs() < 1e-6);

    // Distance
    let  	ptA = Buff![ 0.0f32, 0.0, 0.0 ];
    let  	ptB = Buff![ 0.0f32, 3.0, 4.0 ];
    assert_eq!( ptA.Distance( &ptB), 5.0f32);
    assert_eq!( ptA.DistanceSquared( &ptB), 25.0f32);

    // Lerp
    let  	lerpRes = b1.Lerp( &b2, 0.5f32);
    assert_eq!( lerpRes, Buff![ 3.0f32, 4.0, 5.0, 6.0 ]);

    // Projection & Rejection
    let  	u = Buff![ 1.0f32, 0.0, 0.0 ];
    let  	v = Buff![ 3.0f32, 4.0, 0.0 ];
    let  	proj = v.Project( &u).unwrap();
    assert_eq!( proj, Buff![ 3.0f32, 0.0, 0.0 ]);

    let  	rej = v.Reject( &u).unwrap();
    assert_eq!( rej, Buff![ 0.0f32, 4.0, 0.0 ]);

    // Reflection: (3, 4) reflected across normal (0, 1) -> (3, -4)
    let  	normY = Buff![ 0.0f32, 1.0, 0.0 ];
    let  	refl = v.Reflect( &normY);
    assert_eq!( refl, Buff![ 3.0f32, -4.0, 0.0 ]);

    // 2. Custom UInt Vector Space (5D space with U32)
    let  	vUInt1 = Buff![ U32( 10), U32( 20), U32( 30), U32( 40), U32( 50) ];
    let  	vUInt2 = Buff![ U32( 1), U32( 2), U32( 3), U32( 4), U32( 5) ];

    let  	sumUInt = &vUInt1 + &vUInt2;
    assert_eq!( sumUInt, Buff![ U32( 11), U32( 22), U32( 33), U32( 44), U32( 55) ]);

    let  	scaledUInt = &vUInt1 * U32( 3);
    assert_eq!( scaledUInt, Buff![ U32( 30), U32( 60), U32( 90), U32( 120), U32( 150) ]);

    // Dot product: 10*1 + 20*2 + 30*3 + 40*4 + 50*5 = 10 + 40 + 90 + 160 + 250 = 550
    let  	dotUInt = vUInt1.Dot( &vUInt2);
    assert_eq!( dotUInt, U32( 550));

    // ZeroVec & Splat
    let  	zeroBuff = Buff::< U8>::ZeroVec( 8);
    assert_eq!( zeroBuff.len(), 8);
    assert!( zeroBuff.IsZero());

    let  	splatBuff = Buff::< U8>::Splat( U8( 255), 4);
    assert_eq!( splatBuff, Buff![ U8( 255), U8( 255), U8( 255), U8( 255) ]);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBBox3fBasicOps()
{
    let  	empty = BBox3f::Empty();
    assert!( empty.IsEmpty());

    let  	points = [
        [ -10.0f32, 5.0, 20.0 ],
        [ 30.0, -15.0, 40.0 ],
        [ 0.0, 25.0, -5.0 ],
    ];
    let  	bbox = BBox3f::FromPoints( &points);
    assert!( !bbox.IsEmpty());

    assert_eq!( bbox.Min(), [ -10.0, -15.0, -5.0 ]);
    assert_eq!( bbox.Max(), [ 30.0, 25.0, 40.0 ]);

    let  	center = bbox.Center();
    assert_eq!( center, Pt3f::New( 10.0, 5.0, 17.5));

    let  	extent = bbox.Extent();
    assert_eq!( extent, Pt3f::New( 40.0, 40.0, 45.0));
    assert_eq!( bbox.MaxDim(), 45.0);

    let  	scaleNorm = bbox.ScaleNorm( 240.0);
    assert!(( scaleNorm - ( 240.0 / 45.0)).abs() < 1e-5);

    let  	corners = bbox.Corners();
    assert_eq!( corners.len(), 8);
    assert_eq!( corners[0], Pt3f::New( -10.0, -15.0, -5.0));
    assert_eq!( corners[6], Pt3f::New( 30.0, 25.0, 40.0));

    let  	edges = BBox3f::BoxEdges();
    assert_eq!( edges.len(), 12);
}

//---------------------------------------------------------------------------------------------------------------------------------
