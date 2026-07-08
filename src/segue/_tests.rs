//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	std::sync::atomic::{ AtomicBool, Ordering};
use	crate::{
    flux::InStream,
    segue::{ Charset, shard::Shard, Parser, IGrammar },
    stalks::DynINode,
};


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
fn	TestParserBasic()
{

    let     str = "hello parser";
    let  	mut stream = InStream::from( str);
    let  	mut parser = Parser::New( &mut stream);

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

#[test]
fn TestPostBoxet() 
{
    let  	fired = Arc::new( AtomicBool::new( false));
    let  	firedClone = fired.clone();
    let data = "ab";
    let mut stream = InStream::from(data);
    let mut parser = Parser::New(&mut stream);
    let tree = crate::ShardTree!( "ab" [ |_worker| {
        firedClone.store( true, Ordering::Relaxed);
    } ] );
    let dynNode: &DynINode<'_> = &tree;
    assert!( dynNode.Match(&mut parser));
    assert!( fired.load( Ordering::Relaxed), "Action closure should have fired on match");
}

//---------------------------------------------------------------------------------------------------------------------------------
#[test]
fn TestRgx() 
{
    let  	fired = Arc::new( AtomicBool::new( false));
    let  	firedClone = fired.clone();
    let     alpha = crate::ShardTree!(  [ "a-zA-Z"]);
    let     identRgx = crate::ShardTree!(  *alpha[ |_worker| {
        firedClone.store( true, Ordering::Relaxed);
    } ] ); 

    let  	mut output = String::new();
    {
        let  	mut jsonStream = crate::flux::JsonOutStream::New( &mut output, true);
        jsonStream.KeyField( "identRgx", crate::flux::xflux::XField::FluxSource( &identRgx));
    }
    println!( "{}", output);
    let data = "ab";
    let mut stream = InStream::from(data);
    let mut parser = Parser::New(&mut stream);
    
    if let Some( forge) = parser.ParseTree::<Shard>( &identRgx) {
        let forge_ref = unsafe { &mut *forge };
        assert!( forge_ref.MatchNode(), "identRgx should match 'ab'");
    } else {
        panic!( "ParseTree should return Some");
    }
    assert!( fired.load( Ordering::Relaxed), "Action closure should have fired on match");
}

//---------------------------------------------------------------------------------------------------------------------------------
