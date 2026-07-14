//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::FixedStream,
    shard::{ Charset, Parser, IGrammar, IForge, UInt, Int, Hex, Real, HexReal, Json },
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

    let  	mut m = U32( 0);
    {
        // Test char grammar
        let  	matched = {
            let  	g = &'h';
            let res = g.Parse( &mut parser, m);
            if let Some( nextM) = res {
                m = nextM;
            }
            res.is_some()
        };
        assert!( matched);
        
        let  	matched = {
            let  	g = &'e';
            let res = g.Parse( &mut parser, m);
            if let Some( nextM) = res {
                m = nextM;
            }
            res.is_some()
        };
        assert!( matched);

        // Test &str grammar
        let  	matched = {
            let  	g = &"llo ";
            let res = g.Parse( &mut parser, m);
            if let Some( nextM) = res {
                m = nextM;
            }
            res.is_some()
        };
        assert!( matched);

        let  	matched = {
            let  	g = &cs;
            let res = g.Parse( &mut parser, m);
            if let Some( nextM) = res {
                m = nextM;
            }
            res.is_some()
        };
        assert!( matched);

        // Test failing match (should rollback)
        let  	matched = {
            let  	g = &"fail";
            let res = g.Parse( &mut parser, m);
            res.is_some()
        };
        assert!( !matched);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
#[test]
fn	TestPostBoxet() 
{
    let     data = "ab";
    let     tree = crate::ShardTree!( "ab" [ |_worker| {
        println!("Matched");
    } ] );
    let  	mut stream = FixedStream::from( data);
    let  	mut parser = Parser::New( &mut stream);
    let res = tree.Parse( &mut parser, U32(0));
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    assert!( matched);
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
        jsonStream.KeyField( "identRgx", crate::flux::xflux::XField::FluxSource( &identRgx));
    }
    println!( "{}", output);

    // Test that the Repeat and Action correctly parse strings
    let  	mut stream1 = FixedStream::from( "aBcxYZ");
    let  	mut parser1 = Parser::New( &mut stream1);
    let res1 = identRgx.Parse( &mut parser1, U32(0));
    let matched1 = res1.is_some();
    let m1 = res1.unwrap_or(U32(0)); // Should match greedy
    assert!( matched1);
    assert_eq!( m1.AsUsize(), 6); // All 6 chars consumed

    // Test with non-matching string
    let  	mut stream2 = FixedStream::from( "aBcxYZ123");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = identRgx.Parse( &mut parser2, U32(0));
    let matched2 = res2.is_some();
    let m2 = res2.unwrap_or(U32(0)); // Should succeed but match 6 chars 
    assert!( matched2);
    assert_eq!( m2.AsUsize(), 6); // Rolled back / consumed 6
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn TestUIntShard()
{
    let  	tree = crate::ShardTree!( UInt );
    

    // Test that the UInt shard correctly parses unsigned integer strings
    let  	mut stream1 = FixedStream::from( "12345");
    let  	mut parser1 = Parser::New( &mut stream1);
    let res1 = tree.Parse( &mut parser1, U32(0));
    let matched1 = res1.is_some();
    let m1 = res1.unwrap_or(U32(0));
    assert!( matched1);
    assert_eq!( m1.AsUsize(), 5);
    
    // Test with non-matching string
    let  	mut stream2 = FixedStream::from( "abc");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = tree.Parse( &mut parser2, U32(0));
    let matched2 = res2.is_some();
    let m2 = res2.unwrap_or(U32(0));
    assert!( !matched2);
    
    // Test with mixed string
    let  	mut stream3 = FixedStream::from( "42xyz");
    let  	mut parser3 = Parser::New( &mut stream3);
    let res3 = tree.Parse( &mut parser3, U32(0));
    let matched3 = res3.is_some();
    let m3 = res3.unwrap_or(U32(0));
    assert!( matched3);
    assert_eq!( m3.AsUsize(), 2);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn TestIntShard() {
    let tree = crate::ShardTree!( Int );
    
    // Positive int
    let  	mut stream = FixedStream::from( "+12345");
    let  	mut parser = Parser::New( &mut stream);
    let res = tree.Parse( &mut parser, U32(0));
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    
    assert!( matched);
    assert_eq!( m.AsUsize(), 6);
    
    // Negative int
    let  	mut stream2 = FixedStream::from( "-42");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = tree.Parse( &mut parser2, U32(0));
    let matched2 = res2.is_some();
    let m2 = res2.unwrap_or(U32(0));
    assert!( matched2);
    assert_eq!( m2.AsUsize(), 3);
}

#[test]
fn TestHexShard() {
    let tree = crate::ShardTree!( Hex );
    
    // Standard hex
    let  	mut stream = FixedStream::from( "0x1a2B");
    let  	mut parser = Parser::New( &mut stream);
    let res = tree.Parse( &mut parser, U32(0));
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    
    assert!( matched);
    assert_eq!( m.AsUsize(), 6);
    
    // Hex with sign
    let  	mut stream2 = FixedStream::from( "-0XF");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = tree.Parse( &mut parser2, U32(0));
    let matched2 = res2.is_some();
    let m2 = res2.unwrap_or(U32(0));
    assert!( matched2);
    assert_eq!( m2.AsUsize(), 4);
}

#[test]
fn TestRealShard() {
    let tree = crate::ShardTree!( Real );
    
    // Standard real
    let  	mut stream = FixedStream::from( "3.14159");
    let  	mut parser = Parser::New( &mut stream);
    let res = tree.Parse( &mut parser, U32(0));
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    
    assert!( matched);
    assert_eq!( m.AsUsize(), 7);
    
    // Real with exponent
    let  	mut stream2 = FixedStream::from( "-1.5e+10");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = tree.Parse( &mut parser2, U32(0));
    let matched2 = res2.is_some();
    let m2 = res2.unwrap_or(U32(0));
    assert!( matched2);
    assert_eq!( m2.AsUsize(), 8);
}

#[test]
fn TestHexRealShard() {
    let tree = crate::ShardTree!( HexReal );
    
    // Hex real with fraction
    let  	mut stream = FixedStream::from( "0x1.f");
    let  	mut parser = Parser::New( &mut stream);
    let res = tree.Parse( &mut parser, U32(0));
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    
    assert!( matched);
    assert_eq!( m.AsUsize(), 5);
    
    // Hex real with binary exponent
    let  	mut stream2 = FixedStream::from( "-0x1.abcP-4");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = tree.Parse( &mut parser2, U32(0));
    let matched2 = res2.is_some();
    let m2 = res2.unwrap_or(U32(0));
    assert!( matched2);
    assert_eq!( m2.AsUsize(), 11);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn TestJsonShard() {
    let tree = crate::ShardTree!( Json );
    
    // JSON String
    let  	mut stream1 = FixedStream::from( r#"  "hello world"  "#);
    let  	mut parser1 = Parser::New( &mut stream1);
    let res1 = tree.Parse( &mut parser1, U32(0));
    let matched1 = res1.is_some();
    let m1 = res1.unwrap_or(U32(0));
    assert!( matched1);
    assert_eq!( m1.AsUsize(), 17);
    
    // JSON Object with various types
    let     json_text = r#"
    {
        "string": "value",
        "number": -1.23e4,
        "bool": true,
        "null_val": null,
        "array": [1, 2, 3, false, {"nested": "obj"}]
    }
    "#;
    let  	mut stream2 = FixedStream::from( json_text);
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = tree.Parse( &mut parser2, U32(0));
    let matched2 = res2.is_some();
    let m2 = res2.unwrap_or(U32(0));
    assert!( matched2);
    assert_eq!( m2.AsUsize(), json_text.len());
}

//---------------------------------------------------------------------------------------------------------------------------------
