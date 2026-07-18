//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{ 
    flux::{ JsonOutStream, fluxexport::FieldExp, FixedStream, BuffStream, IStream, IFluxExportSource, IFluxImportSource}, 
    silo::{ U8, U32 } 
};
use	std::fs;

//---------------------------------------------------------------------------------------------------------------------------------

struct Point
{
    _X : f64,
    _Y :f64,
}

crate::ImplFluxSource!( Point, _X, _Y);

#[test]
fn	TestJsonOutStream()
{
    let  	prices = crate::Buff![ 12.34_f32, 56.78, 90.12, 34.56, 78.90 ];
    let  	arr = prices.Arr();

    let     pt = Point{ _X: 10.0, _Y: 30.3};

    let  	mut output = String::new();
    {
        let  	mut jsonStream = JsonOutStream::New( &mut output, true);

        jsonStream.KeyField( "point", FieldExp::FluxSource( &pt));
        jsonStream.KeyField( "prices", FieldExp::FluxSource( &arr));
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
    assert_eq!( <&str>::from( stream.BytesAt( U32( 1), U32( 2))), "bc");
    assert_eq!( <&str>::from( stream.BytesAt( U32( 1), U32( 10))), "bc");
    assert_eq!( <&str>::from( stream.BytesAt( U32( 5), U32( 1))), "");
    assert_eq!( <&str>::from( stream.BytesAt( U32( 5), U32( 10))), "");
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
    assert_eq!( <&str>::from( stream.BytesAt( U32( 1), U32( 4))), "ello");
    fs::remove_file( path).unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestFluxSourceDisplayDebug()
{
    let  	pt1 = Point { _X: 10.0, _Y: 30.3 };
    let  	expSource: &dyn IFluxExportSource = &pt1;

    let  	disp = format!( "{}", expSource);
    let  	debug = format!( "{:?}", expSource);

    assert!( disp.contains( "\"_X\": 10"));
    assert!( disp.contains( "\"_Y\": 30.3"));
    assert!( debug.contains( "\n"));

    let  	mut pt2 = Point { _X: 0., _Y: 0. };
    {
        use crate::flux::fluximport::FieldImp;
        let  	mut field = FieldImp::Null;
        pt2.FetchFieldImp( &mut field);
        if let FieldImp::Obj( ref mut cb) = field {
            let  	mut xField = FieldImp::Null;
            assert!( cb( "_X", &mut xField));
            xField.PostF64( 10.0);

            let  	mut yField = FieldImp::Null;
            assert!( cb( "_Y", &mut yField));
            yField.PostF64( 30.3);

            assert!( !cb( "_Z", &mut FieldImp::Null));
        } else {
            panic!( "Expected FieldImp::Obj");
        }
    }
    assert_eq!( pt2._X, 10.0);
    assert_eq!( pt2._Y, 30.3);
}

//---------------------------------------------------------------------------------------------------------------------------------

