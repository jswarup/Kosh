//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::FixedStream,
    shard::{ Charset, Parser, IGrammar, UInt, Int, Hex, Real, HexReal },
    stalks::DynINode,
    silo::U32,
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
    let  	mut cs = Charset::New();
    cs.SetChar( b'p');
    let  	mut stream = FixedStream::from( str);
    let  	mut parser = Parser::New( &mut stream);

    {
        // Test char grammar
        assert!( 'h'.Match( &mut parser, U32(0)).is_some());
        assert!( 'e'.Match( &mut parser, U32(1)).is_some());

        // Test &str grammar
        assert!( "llo ".Match( &mut parser, U32(2)).is_some());

        assert!( cs.Match( &mut parser, U32(6)).is_some());

        // Test failing match (should rollback)
        assert!( !"fail".Match( &mut parser, U32(7)).is_some());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
#[test]
fn TestPostBoxet() 
{
    let data = "ab";
    let tree = crate::ShardTree!( "ab" [ |_worker| {
        println!("Matched");
    } ] );
    let dynNode: &DynINode<'_> = &*tree;
    let mut stream = FixedStream::from(data);
    let mut parser = Parser::New(&mut stream);
    assert!(dynNode.Match(&mut parser, U32(0)).is_some());
}

//---------------------------------------------------------------------------------------------------------------------------------
#[test]
fn TestRgx2() 
{

    let     alpha = crate::ShardTree!(  [ "a-zA-Z"]);
    let     identRgx = crate::ShardTree!(  [ "a-z"] < ["A-Z"] < +alpha[ |_worker| {
        // marker tracking removed
    } ] ); 
    let  	mut output = String::new();
    {
        let  	mut jsonStream = crate::flux::JsonOutStream::New( &mut output, true);
        jsonStream.KeyField( "identRgx", crate::flux::xflux::XField::FluxSource( &*identRgx));
    }
    println!( "{}", output);

    // Test that the Repeat and Action correctly parse strings
    let mut stream1 = FixedStream::from("aBcxYZ");
    let mut parser1 = Parser::New(&mut stream1);
    let match1 = identRgx.Match(&mut parser1, U32(0));
    assert!(match1.is_some()); // Should match greedy
    assert_eq!(match1.unwrap().AsUsize(), 6); // All 6 chars consumed

    // Test with non-matching string
    let mut stream2 = FixedStream::from("aBcxYZ123");
    let mut parser2 = Parser::New(&mut stream2);
    let match2 = identRgx.Match(&mut parser2, U32(0));
    assert!(match2.is_some()); // Should succeed but match 6 chars 
    assert_eq!(match2.unwrap().AsUsize(), 6); // Rolled back / consumed 6
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn TestUIntShard()
{
    let  	tree = crate::ShardTree!( UInt );
    
    let  	dynNode: &DynINode<'_> = &*tree;
    assert_eq!( dynNode._Size().AsUsize(), 0);
    
    // Test that the UInt shard correctly parses unsigned integer strings
    let  	mut stream1 = FixedStream::from( "12345");
    let  	mut parser1 = Parser::New( &mut stream1);
    let  	match1 = tree.Match( &mut parser1, U32( 0));
    assert!( match1.is_some());
    assert_eq!( match1.unwrap().AsUsize(), 5);
    
    // Test with non-matching string
    let  	mut stream2 = FixedStream::from( "abc");
    let  	mut parser2 = Parser::New( &mut stream2);
    let  	match2 = tree.Match( &mut parser2, U32( 0));
    assert!( match2.is_none());
    
    // Test with mixed string
    let  	mut stream3 = FixedStream::from( "42xyz");
    let  	mut parser3 = Parser::New( &mut stream3);
    let  	match3 = tree.Match( &mut parser3, U32( 0));
    assert!( match3.is_some());
    assert_eq!( match3.unwrap().AsUsize(), 2);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn TestIntShard() {
    let tree = crate::ShardTree!( Int );
    
    // Positive int
    let mut stream = FixedStream::from("+12345");
    let mut parser = Parser::New(&mut stream);
    let match1 = tree.Match(&mut parser, U32(0));
    assert!(match1.is_some());
    assert_eq!(match1.unwrap().AsUsize(), 6);
    
    // Negative int
    let mut stream2 = FixedStream::from("-42");
    let mut parser2 = Parser::New(&mut stream2);
    let match2 = tree.Match(&mut parser2, U32(0));
    assert!(match2.is_some());
    assert_eq!(match2.unwrap().AsUsize(), 3);
}

#[test]
fn TestHexShard() {
    let tree = crate::ShardTree!( Hex );
    
    // Standard hex
    let mut stream = FixedStream::from("0x1a2B");
    let mut parser = Parser::New(&mut stream);
    let match1 = tree.Match(&mut parser, U32(0));
    assert!(match1.is_some());
    assert_eq!(match1.unwrap().AsUsize(), 6);
    
    // Hex with sign
    let mut stream2 = FixedStream::from("-0XF");
    let mut parser2 = Parser::New(&mut stream2);
    let match2 = tree.Match(&mut parser2, U32(0));
    assert!(match2.is_some());
    assert_eq!(match2.unwrap().AsUsize(), 4);
}

#[test]
fn TestRealShard() {
    let tree = crate::ShardTree!( Real );
    
    // Standard real
    let mut stream = FixedStream::from("3.14159");
    let mut parser = Parser::New(&mut stream);
    let match1 = tree.Match(&mut parser, U32(0));
    assert!(match1.is_some());
    assert_eq!(match1.unwrap().AsUsize(), 7);
    
    // Real with exponent
    let mut stream2 = FixedStream::from("-1.5e+10");
    let mut parser2 = Parser::New(&mut stream2);
    let match2 = tree.Match(&mut parser2, U32(0));
    assert!(match2.is_some());
    assert_eq!(match2.unwrap().AsUsize(), 8);
}

#[test]
fn TestHexRealShard() {
    let tree = crate::ShardTree!( HexReal );
    
    // Hex real with fraction
    let mut stream = FixedStream::from("0x1.f");
    let mut parser = Parser::New(&mut stream);
    let match1 = tree.Match(&mut parser, U32(0));
    assert!(match1.is_some());
    assert_eq!(match1.unwrap().AsUsize(), 5);
    
    // Hex real with binary exponent
    let mut stream2 = FixedStream::from("-0x1.abcP-4");
    let mut parser2 = Parser::New(&mut stream2);
    let match2 = tree.Match(&mut parser2, U32(0));
    assert!(match2.is_some());
    assert_eq!(match2.unwrap().AsUsize(), 11);
}

//---------------------------------------------------------------------------------------------------------------------------------
