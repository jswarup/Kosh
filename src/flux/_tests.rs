//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::{ JsonOutStream, fluxout::FieldOut, FixedStream, BuffStream, IStream, IFluxOutSource }, silo::{ U8, U32 } };
use	std::fs;

//---------------------------------------------------------------------------------------------------------------------------------

struct Point
{
    _X : f64,
    _Y :f64,
}

crate::ImplIFluxOutSource!( Point, _X, _Y);

#[test]
fn	TestJsonOutStream()
{
    let  	prices = crate::Buff![ 12.34_f32, 56.78, 90.12, 34.56, 78.90 ];
    let  	arr = prices.Arr();

    let     pt = Point{ _X: 10.0, _Y: 30.3};

    let  	mut output = String::new();
    {
        let  	mut jsonStream = JsonOutStream::New( &mut output, true);

        jsonStream.KeyField( "point", FieldOut::FluxSource( &pt));
        jsonStream.KeyField( "prices", FieldOut::FluxSource( &arr));
    }

    fs::write( "a.json", output).unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestInStream()
{
    let  	data = "abc";
    let  	mut stream = FixedStream::from( data);
    
    // Test random-access At()
    assert_eq!( stream.At( U32( 0)), b'a');
    assert_eq!( stream.At( U32( 1)), b'b');
    assert_eq!( stream.At( U32( 2)), b'c');
    assert_eq!( stream.At( U32( 5)), U8::_0);

    // Test stateless BytesAt()
    assert_eq!( stream.BytesAt( U32( 1), U32( 2)), b"bc");
    assert_eq!( stream.BytesAt( U32( 1), U32( 10)), b"bc");
    assert_eq!( stream.BytesAt( U32( 5), U32( 1)), b"");
    assert_eq!( stream.BytesAt( U32( 5), U32( 10)), b"");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestInStreamFromFile()
{
    let  	path = "test_inbuffstream.txt";
    fs::write( path, b"hello").unwrap();
    let  	mut stream = BuffStream::FromFile( path).unwrap();
    assert_eq!( stream.At( U32( 0)), b'h');
    assert_eq!( stream.At( U32( 1)), b'e');
    assert_eq!( stream.BytesAt( U32( 1), U32( 4)), b"ello");
    fs::remove_file( path).unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestFluxSourceDisplayDebug()
{
    let  	pt = Point { _X: 10.0, _Y: 30.3 };
    let  	source: &dyn IFluxOutSource = &pt;

    let  	disp = format!( "{}", source);
    let  	debug = format!( "{:?}", source);

    assert!( disp.contains( "\"_X\": 10"));
    assert!( disp.contains( "\"_Y\": 30.3"));
    assert!( debug.contains( "\n"));
}

//---------------------------------------------------------------------------------------------------------------------------------

