//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist::Atelier,
    segue::{ Charset, InStream, Shard, JsonOutStream, xflux::XField },
    silo::{ IAccess, U8, U32 }
};

 
//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestShardFromCharAndString()
{
    let  	_nodeTree = crate::ShardTree!( ( "cShard" < ( 'b' < [ "a-z" ] < "aShard"[ || {
                        print!( "{} ", 50);
                    }] )));
    let  	atelier = Atelier::New( U32( 4));
    let  	_mainMaestro = atelier.MainMaestro();
    atelier.DoLaunch();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestCharsetOps()
{
    // 1. Check ToString formatting of special/escaped chars
    let  	mut cs1 = Charset::New();
    cs1.SetChar( b' ');
    cs1.SetChar( b'-');
    cs1.SetChar( b'\\');
    println!( "cs1: {}", cs1);
    // 2. Check Compare values
    let  	mut cs2 = Charset::New();
    cs2.SetChar( b'a');
    let  	mut cs3 = Charset::New();
    cs3.SetChar( b'b');
    println!( "Compare cs2 to cs3: {}", cs2.Compare( &cs3));
    println!( "Compare cs3 to cs2: {}", cs3.Compare( &cs2));
    // 3. Check negation formatting
    let  	cs4 = Charset::Word().Negative();
    println!( "cs4 (NonWord): {}", cs4);
    let  	cs5 = Charset::Digit().Negative();
    println!( "cs5 (NonDigit): {}", cs5);
}

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
    
    std::fs::write( "a.json", output).unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestInStream()
{
    let  	data = [U8( b'a'), U8( b'b'), U8( b'c')];
    let  	mut stream = InStream::FromArr( (&data).into());
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
    std::fs::write( path, b"hello").unwrap();
    let  	mut stream = InStream::FromFile( path).unwrap();
    assert_eq!( stream.Curr(), U8( b'h'));
    assert!( stream.Next());
    assert_eq!( stream.Curr(), U8( b'e'));
    std::fs::remove_file( path).unwrap();
}



//---------------------------------------------------------------------------------------------------------------------------------

struct DummyForge {
    _Offset: U32,
}

impl<'a> crate::segue::IForge<'a> for DummyForge {
    fn	Parent( &self) -> Option< &'a dyn crate::segue::IForge<'a>> { None }
    fn	Offset( &self) -> U32 { self._Offset }
}

#[test]
fn	TestParserBasic()
{
    use	crate::segue::{ Parser, IGrammar, Charset };
    
    let  	data = [U8( b'h'), U8( b'e'), U8( b'l'), U8( b'l'), U8( b'o'), U8( b' '), U8( b'p'), U8( b'a'), U8( b'r'), U8( b's'), U8( b'e'), U8( b'r')];
    let  	arr = crate::silo::Arr::from( &data[..]);
    let  	mut stream = InStream::FromArr( arr);
    let  	forge = DummyForge { _Offset: U32( 0) };
    let  	mut parser = Parser::New( &mut stream, &forge);

    // Test char grammar
    assert!( 'h'.Match( &mut parser));
    assert!( 'e'.Match( &mut parser));

    // Test &str grammar
    assert!( "llo ".Match( &mut parser));

    // Test charset grammar
    let  	mut cs = Charset::New();
    cs.SetChar( b'p');
    assert!( cs.Match( &mut parser));
    
    // Test failing match (should rollback)
    assert!( !"fail".Match( &mut parser));
    
    // Test continuing after fail
    assert!( "arser".Match( &mut parser));
}

//---------------------------------------------------------------------------------------------------------------------------------
