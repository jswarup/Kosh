//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::{ JsonOutStream, xflux::XField, FixedStream, BuffStream, IStream, IXFluxSource }, silo::{ U8, U32 } };
use	std::fs;

//---------------------------------------------------------------------------------------------------------------------------------

struct Point
{
    _X : f64,
    _Y :f64,
}

crate::ImplIXFluxSource!( Point, _X, _Y);

#[test]
fn	TestJsonOutStream()
{
    let  	prices = crate::Buff![ 12.34_f32, 56.78, 90.12, 34.56, 78.90 ];
    let  	arr = prices.Arr();

    let     pt = Point{ _X: 10.0, _Y: 30.3};

    let  	mut output = String::new();
    {
        let  	mut jsonStream = JsonOutStream::New( &mut output, true);

        jsonStream.KeyField( "point", XField::FluxSource( &pt));
        jsonStream.KeyField( "prices", XField::FluxSource( &arr));
    }

    fs::write( "a.json", output).unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestInStream()
{
    let  	data = "abc";
    let  	mut stream = FixedStream::from( data);
    assert_eq!( stream.Curr(), b'a');
    assert!( stream.Next());
    assert_eq!( stream.Curr(), b'b');
    assert!( stream.Next());
    assert_eq!( stream.Curr(), b'c');
    assert!( !stream.Next());
    assert_eq!( stream.Curr(), 0);
    stream.RollTo( U32( 1));
    assert_eq!( stream.Curr(), b'b');
    let  	rest1 = stream.Bytes( 2);
    assert_eq!( rest1, b"bc");
    assert_eq!( stream.Bytes( 10), b"bc");
    stream.RollTo( U32( 5));
    assert_eq!( stream.Curr(), U8::_0);
    let  	rest5 = stream.Bytes( 1);
    assert_eq!( rest5, b"");
    assert_eq!( stream.Bytes( 10), b"");
    // Test random-access At()
    assert_eq!( stream.At( U32( 0)), b'a');
    assert_eq!( stream.At( U32( 1)), b'b');
    assert_eq!( stream.At( U32( 2)), b'c');
    assert_eq!( stream.At( U32( 5)), U8::_0);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestInStreamFromFile()
{
    let  	path = "test_inbuffstream.txt";
    fs::write( path, b"hello").unwrap();
    let  	mut stream = BuffStream::FromFile( path).unwrap();
    assert_eq!( stream.Curr(), b'h');
    assert!( stream.Next());
    assert_eq!( stream.Curr(), b'e');
    fs::remove_file( path).unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestFluxSourceDisplayDebug()
{
    let  	pt = Point { _X: 10.0, _Y: 30.3 };
    let  	source: &dyn IXFluxSource = &pt;

    let  	disp = format!( "{}", source);
    let  	debug = format!( "{:?}", source);

    assert!( disp.contains( "\"_X\": 10"));
    assert!( disp.contains( "\"_Y\": 30.3"));
    assert!( debug.contains( "\n"));
}

//---------------------------------------------------------------------------------------------------------------------------------

