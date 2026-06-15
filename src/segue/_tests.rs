//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------
use	crate::{
    heist::Atelier,
    segue::
    { Charset, Shard },
    silo::U32
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
