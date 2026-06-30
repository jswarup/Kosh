//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{ segue::parser::IForge, silo::Arr };
use	crate::{
    flux::InStream,
    segue::{ Charset, shard::Shard },
    silo::{ U8, U32 },
    heist::Atelier
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

struct DummyForge {
    _Offset: U32,
}

impl<'a> IForge<'a> for DummyForge {
    fn	Parent( &self) -> Option< &'a dyn IForge<'a>> { None }
    fn	Offset( &self) -> U32 { self._Offset }
}

#[test]
fn	TestParserBasic()
{
    use	crate::segue::{ Parser, IGrammar, Charset };
    
    let  	data = [U8( b'h'), U8( b'e'), U8( b'l'), U8( b'l'), U8( b'o'), U8( b' '), U8( b'p'), U8( b'a'), U8( b'r'), U8( b's'), U8( b'e'), U8( b'r')];
    let  	arr = Arr::from( &data[..]);
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
