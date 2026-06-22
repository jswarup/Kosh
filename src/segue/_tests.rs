//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist::Atelier,
    segue::
    { Charset, InStream, Shard, JsonListener, JsonOutStream, JsonValue },
    silo::{ Arr, Buff, IAccess, U8, U32 }
};

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestShardBuds()
{
    let  	aShard = Shard::New( |_m| {
        print!( "{} ", 10);
    });
    let  	bShard = Shard::New( |_m| {
        print!( "{} ", 20);
    });
    let  	cShard = Shard::New( |_m| {
        print!( "{} ", 40);
    });
    let  	_nodeTree = crate::ShardTree!( ( cShard
            < ( bShard
                | aShard
                | ( |_m| {
                    print!( "{} ", 50);
                })))
    );
    let  	atelier = Atelier::New( U32( 4));
    let  	_mainMaestro = atelier.MainMaestro();
    atelier.DoLaunch();
}

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

#[test]
fn	TestJsonOutStream()
{
    let  	mut prices = vec![ 12.34_f32, 56.78, 90.12, 34.56, 78.90 ];
    let  	ptr = std::ptr::NonNull::new( prices.as_mut_ptr()).unwrap();
    let  	arr = Arr::New( ptr, U32( 5));
    
    let  	mut output = String::new();
    let  	mut jsonStream = JsonOutStream::New( &mut output, true);
    
    jsonStream.OpenObject( "");
    jsonStream.KeyArray( "prices", &arr, |p| JsonValue::F64( *p as f64));
    jsonStream.CloseObject();
    
    std::fs::write( "a.json", output).unwrap();
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestInStream()
{
    let  	data = [U8( b'a'), U8( b'b'), U8( b'c')];
    let  	buff = Buff::Create( U32( 3), |i| data[i.AsUsize()]);
    let  	mut stream = InStream::New( buff);
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
    assert_eq!( stream.Remaining(), "bc");
    stream.RollTo( U32( 5));
    assert_eq!( stream.Curr(), U8::_0);
    let  	rest5 = stream.Rest();
    assert_eq!( rest5.Size(), 0);
    assert_eq!( stream.Remaining(), "");
}

//---------------------------------------------------------------------------------------------------------------------------------

