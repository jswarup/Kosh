//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::{ JsonOutStream, xflux::XField, InStream }, silo::{ U8, U32, IAccess } };
use	std::fs;

//---------------------------------------------------------------------------------------------------------------------------------

struct Point 
{
    _X : f64,
    _Y :f64,
}

crate::ImplIXFluxable!( Point, _X, _Y);

#[test] 
fn	TestJsonOutStream()
{
    let  	prices = crate::Buff![ 12.34_f32, 56.78, 90.12, 34.56, 78.90 ];
    let  	arr = prices.Arr();
    
    let     pt = Point{ _X: 10.0, _Y: 30.3};
    
    let  	mut output = String::new();
    {
        let  	mut jsonStream = JsonOutStream::New( &mut output, true);

        jsonStream.KeyField( "point", XField::Fluxable( &pt));
        jsonStream.KeyField( "prices", XField::Fluxable( &arr));
    }
    
    fs::write( "a.json", output).unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestInStream()
{
    let  	data = "abc";
    let  	mut stream = InStream::FromStr( &data);
    assert_eq!( stream.Curr(), U8( b'a'));
    assert!( stream.Next());
    assert_eq!( stream.Curr(), U8( b'b'));
    assert!( stream.Next());
    assert_eq!( stream.Curr(), U8( b'c'));
    assert!( !stream.Next());
    assert_eq!( stream.Curr(), U8::_0);
    stream.RollTo( U32( 1));
    assert_eq!( stream.Curr(), U8( b'b'));
    let  	rest1 = stream.Rest();
    assert_eq!( rest1.Size(), 2);
    assert_eq!( *rest1.At( 0), U8( b'b'));
    assert_eq!( *rest1.At( 1), U8( b'c'));
    assert_eq!( stream.RemainingBytes(), b"bc");
    stream.RollTo( U32( 5));
    assert_eq!( stream.Curr(), U8::_0);
    let  	rest5 = stream.Rest();
    assert_eq!( rest5.Size(), 0);
    assert_eq!( stream.RemainingBytes(), b"");
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestInStreamFromFile()
{
    let  	path = "test_inbuffstream.txt";
    fs::write( path, b"hello").unwrap();
    let  	mut stream = InStream::FromFile( path).unwrap();
    assert_eq!( stream.Curr(), U8( b'h'));
    assert!( stream.Next());
    assert_eq!( stream.Curr(), U8( b'e'));
    fs::remove_file( path).unwrap();
}
