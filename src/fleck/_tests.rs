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
    assert_eq!( p0.x, 10.0);
    assert_eq!( p0.y, 20.0);
    assert_eq!( p0.z, 30.0);
    assert_eq!( p0.intensity, None);
    assert_eq!( p0.r, None);

    let  	p1 = arr.At( U32( 1));
    assert_eq!( p1.x, 40.5);
    assert_eq!( p1.y, -50.25);
    assert_eq!( p1.z, 60.125);

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
    assert_eq!( p0.x, 1.0);
    assert_eq!( p0.y, 2.0);
    assert_eq!( p0.z, 3.0);
    assert_eq!( p0.intensity, Some( 0.75));

    let  	p1 = arr.At( U32( 1));
    assert_eq!( p1.x, -4.0);
    assert_eq!( p1.y, -5.0);
    assert_eq!( p1.z, -6.0);
    assert_eq!( p1.intensity, Some( 0.25));
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
    assert_eq!( p0.x, 10.0);
    assert_eq!( p0.y, 20.0);
    assert_eq!( p0.z, 30.0);
    assert_eq!( p0.r, Some( U8( 255)));
    assert_eq!( p0.g, Some( U8( 128)));
    assert_eq!( p0.b, Some( U8( 64)));

    let  	p1 = arr.At( U32( 1));
    assert_eq!( p1.r, Some( U8( 0)));
    assert_eq!( p1.g, Some( U8( 255)));
    assert_eq!( p1.b, Some( U8( 0)));
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
    assert_eq!( p0.x, 1.0);
    assert_eq!( p0.y, 2.0);
    assert_eq!( p0.z, 3.0);
    assert_eq!( p0.intensity, Some( 0.9));
    assert_eq!( p0.r, Some( U8( 255)));
    assert_eq!( p0.g, Some( U8( 200)));
    assert_eq!( p0.b, Some( U8( 150)));
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
    assert_eq!( p0.x, 0.0125);
    assert_eq!( p0.y, -35.0);
    assert_eq!( p0.z, 4.0);

    let  	p1 = arr.At( U32( 1));
    assert_eq!( p1.x, 10.0);
    assert_eq!( p1.y, 20.0);
    assert_eq!( p1.z, 30.0);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestPtsToDtoConversion()
{
    let  	ptsData = "0.0 0.0 0.0\n100.0 200.0 300.0\n";
    let  	cloud = ParsePts( ptsData).unwrap();
    let  	dto = cloud.ToDto();

    assert_eq!( dto.count, 2);
    assert_eq!( dto.points.len(), 2);
    assert_eq!( dto.points[0], [0.0, 0.0, 0.0]);
    assert_eq!( dto.points[1], [100.0, 200.0, 300.0]);
    assert_eq!( dto.bbox_min, [0.0, 0.0, 0.0]);
    assert_eq!( dto.bbox_max, [100.0, 200.0, 300.0]);
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
    assert_eq!( p0.x, 5.0);
    assert_eq!( p0.y, 15.0);
    assert_eq!( p0.z, 25.0);
}

//---------------------------------------------------------------------------------------------------------------------------------
