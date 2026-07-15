//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::{ FixedStream, fluximport::FieldImp, fluxexport::FieldExp },
    shard::{ Charset, Parser, UInt, Int, Hex, Real, HexReal, Json },
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
            let res = parser.ParseGrammar( &g, m, FieldImp::Null);
            if let Some( nextM) = res {
                m = nextM;
            }
            res.is_some()
        };
        assert!( matched);
        
        let  	matched = {
            let  	g = &'e';
            let res = parser.ParseGrammar( &g, m, FieldImp::Null);
            if let Some( nextM) = res {
                m = nextM;
            }
            res.is_some()
        };
        assert!( matched);

        // Test &str grammar
        let  	matched = {
            let  	g = &"llo ";
            let res = parser.ParseGrammar( &g, m, FieldImp::Null);
            if let Some( nextM) = res {
                m = nextM;
            }
            res.is_some()
        };
        assert!( matched);

        let  	matched = {
            let  	g = &cs;
            let res = parser.ParseGrammar( &g, m, FieldImp::Null);
            if let Some( nextM) = res {
                m = nextM;
            }
            res.is_some()
        };
        assert!( matched);

        // Test failing match (should rollback)
        let  	matched = {
            let  	g = &"fail";
            let res = parser.ParseGrammar( &g, m, FieldImp::Null);
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
    let res = parser.ParseGrammar( &tree, U32(0), FieldImp::Null);
    let matched = res.is_some();
    let  	_m = res.unwrap_or(U32(0));
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
        jsonStream.KeyField( "identRgx", FieldExp::FluxSource( &identRgx));
    }
    println!( "{}", output);

    // Test that the Repeat and Action correctly parse strings
    let  	mut stream1 = FixedStream::from( "aBcxYZ");
    let  	mut parser1 = Parser::New( &mut stream1);
    let res1 = parser1.ParseGrammar( &identRgx, U32(0), FieldImp::Null);
    let matched1 = res1.is_some();
    let m1 = res1.unwrap_or(U32(0)); // Should match greedy
    assert!( matched1);
    assert_eq!( m1.AsUsize(), 6); // All 6 chars consumed

    // Test with non-matching string
    let  	mut stream2 = FixedStream::from( "aBcxYZ123");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = parser2.ParseGrammar( &identRgx, U32(0), FieldImp::Null);
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
    let res1 = parser1.ParseGrammar( &tree, U32(0), FieldImp::Null);
    let matched1 = res1.is_some();
    let m1 = res1.unwrap_or(U32(0));
    assert!( matched1);
    assert_eq!( m1.AsUsize(), 5);
    
    // Test with non-matching string
    let  	mut stream2 = FixedStream::from( "abc");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = parser2.ParseGrammar( &tree, U32(0), FieldImp::Null);
    let matched2 = res2.is_some();
    let  	_m2 = res2.unwrap_or(U32(0));
    assert!( !matched2);
    
    // Test with mixed string
    let  	mut stream3 = FixedStream::from( "42xyz");
    let  	mut parser3 = Parser::New( &mut stream3);
    let res3 = parser3.ParseGrammar( &tree, U32(0), FieldImp::Null);
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
    let res = parser.ParseGrammar( &tree, U32(0), FieldImp::Null);
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    
    assert!( matched);
    assert_eq!( m.AsUsize(), 6);
    
    // Negative int
    let  	mut stream2 = FixedStream::from( "-42");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = parser2.ParseGrammar( &tree, U32(0), FieldImp::Null);
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
    let res = parser.ParseGrammar( &tree, U32(0), FieldImp::Null);
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    
    assert!( matched);
    assert_eq!( m.AsUsize(), 6);
    
    // Hex with sign
    let  	mut stream2 = FixedStream::from( "-0XF");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = parser2.ParseGrammar( &tree, U32(0), FieldImp::Null);
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
    let res = parser.ParseGrammar( &tree, U32(0), FieldImp::Null);
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    
    assert!( matched);
    assert_eq!( m.AsUsize(), 7);
    
    // Real with exponent
    let  	mut stream2 = FixedStream::from( "-1.5e+10");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = parser2.ParseGrammar( &tree, U32(0), FieldImp::Null);
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
    let res = parser.ParseGrammar( &tree, U32(0), FieldImp::Null);
    let matched = res.is_some();
    let m = res.unwrap_or(U32(0));
    
    assert!( matched);
    assert_eq!( m.AsUsize(), 5);
    
    // Hex real with binary exponent
    let  	mut stream2 = FixedStream::from( "-0x1.abcP-4");
    let  	mut parser2 = Parser::New( &mut stream2);
    let res2 = parser2.ParseGrammar( &tree, U32(0), FieldImp::Null);
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
    let res1 = parser1.ParseGrammar( &tree, U32(0), FieldImp::Null);
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
    let res2 = parser2.ParseGrammar( &tree, U32(0), FieldImp::Null);
    let matched2 = res2.is_some();
    let m2 = res2.unwrap_or(U32(0));
    assert!( matched2);
    assert_eq!( m2.AsUsize(), json_text.len());
}

//---------------------------------------------------------------------------------------------------------------------------------


//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestJsonParsingStruct()
{
    use crate::flux::fluximport::{ FieldImp, IFluxImportSource };
    use crate::silo::U64;

    #[derive( Default, Debug, PartialEq)]
    struct Person {
        name: String,
        age: U64,
        is_active: bool,
    }

    impl IFluxImportSource for Person {
        fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>) {
            let person_ptr = self as *mut Person;
            *field = FieldImp::Obj( Box::new( move |key, child| {
                let person = unsafe { &mut *person_ptr };
                if key == "name" {
                    *child = FieldImp::String( &mut person.name);
                    true
                } else if key == "age" {
                    *child = FieldImp::U64( &mut person.age);
                    true
                } else if key == "is_active" {
                    *child = FieldImp::Bool( &mut person.is_active);
                    true
                } else {
                    false
                }
            }));
        }
    }

    let  	str = r#"{ "name": "Alice", "age": 30, "is_active": true }"#;
    let  	mut stream = crate::flux::FixedStream::from( str);
    let  	mut parser = Parser::New( &mut stream);
    
    let  	mut person = Person::default();
    let  	m = U32(0);
    
    let  	sink = FieldImp::FluxSource( &mut person);       
    
    // We expect Json.Parse to update `person` via the sink closure!
    let  	matched = parser.ParseGrammar( &Json, m, sink);
    
    assert!( matched.is_some());
    assert_eq!( person.name, "Alice");
    assert_eq!( person.age, U64( 30));
    assert_eq!( person.is_active, true);
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestIGrammarString()
{
	// ---- 1. Match a plain quoted string into a String sink ----------------------------

	let  	src = r#""hello""#;
	let  	mut stream = FixedStream::from( src);
	let  	mut parser = Parser::New( &mut stream);
	let  	mut captured = String::new();
	let  	grammar: String = String::new();

	let  	result = parser.ParseGrammar( &grammar, U32( 0), FieldImp::String( &mut captured));
	assert!( result.is_some(), "plain string match failed");
	assert_eq!( captured, "hello");
	// Mark should be exactly past the closing quote (7 bytes: "hello")
	assert_eq!( result.unwrap(), U32( 7));

	// ---- 2. Match with escaped quote inside ------------------------------------------

	let  	src2 = "\"say \\\"hi\\\"\"";
	let  	mut stream2 = FixedStream::from( src2);
	let  	mut parser2 = Parser::New( &mut stream2);
	let  	mut captured2 = String::new();
	let  	grammar2: String = String::new();

	let  	result2 = parser2.ParseGrammar( &grammar2, U32( 0), FieldImp::String( &mut captured2));
	assert!( result2.is_some(), "escaped-quote string match failed");
			assert_eq!( captured2, "say \\\"hi\\\"");

	// ---- 3. Null sink: match succeeds, no capture -----------------------------------

	let  	src3 = r#""world""#;
	let  	mut stream3 = FixedStream::from( src3);
	let  	mut parser3 = Parser::New( &mut stream3);
	let  	grammar3: String = String::new();

	let  	result3 = parser3.ParseGrammar( &grammar3, U32( 0), FieldImp::Null);
	assert!( result3.is_some(), "null-sink match failed");
	assert_eq!( result3.unwrap(), U32( 7));

	// ---- 4. No opening quote: match fails -------------------------------------------

	let  	src4 = "not_quoted";
	let  	mut stream4 = FixedStream::from( src4);
	let  	mut parser4 = Parser::New( &mut stream4);
	let  	grammar4: String = String::new();

	let  	result4 = parser4.ParseGrammar( &grammar4, U32( 0), FieldImp::Null);
	assert!( result4.is_none(), "non-quoted input should fail");

	// ---- 5. Empty quoted string -----------------------------------------------------

	let  	src5 = r#""""#;
	let  	mut stream5 = FixedStream::from( src5);
	let  	mut parser5 = Parser::New( &mut stream5);
	let  	mut captured5 = String::new();
	let  	grammar5: String = String::new();

	let  	result5 = parser5.ParseGrammar( &grammar5, U32( 0), FieldImp::String( &mut captured5));
	assert!( result5.is_some(), "empty string match failed");
	assert_eq!( captured5, "");
}

