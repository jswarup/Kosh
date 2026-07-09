//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::FixedStream,
    segue::{ Charset, shard::Shard, Parser, IGrammar },
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
    let dynNode: &DynINode<'_> = &tree;
    let mut stream = FixedStream::from(data);
    let mut parser = Parser::New(&mut stream);
    assert!(dynNode.Match(&mut parser, U32(0)).is_some());
}

//---------------------------------------------------------------------------------------------------------------------------------
#[test]
fn TestRgx2() 
{
    use crate::segue::parser::IWorkerExt;
    let     alpha = crate::ShardTree!(  [ "a-zA-Z"]);
    let     identRgx = crate::ShardTree!(  [ "a-z"] < ["A-Z"] < +alpha[ |_worker| {
        // marker tracking removed
    } ] ); 
    let  	mut output = String::new();
    {
        let  	mut jsonStream = crate::flux::JsonOutStream::New( &mut output, true);
        jsonStream.KeyField( "identRgx", crate::flux::xflux::XField::FluxSource( &identRgx));
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
