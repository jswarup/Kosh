//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::InStream,
    segue::{ Charset, shard::Shard, Parser, IGrammar, parser::{IForge, Forge, LeafForge} },
    silo::{ U8, U32, Arr},
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
    
    let  	data = [U8( b'h'), U8( b'e'), U8( b'l'), U8( b'l'), U8( b'o'), U8( b' '), U8( b'p'), U8( b'a'), U8( b'r'), U8( b's'), U8( b'e'), U8( b'r')];
    let  	arr = Arr::from( &data[..]);
    let  	mut stream = InStream::FromArr( arr);
    let  	mut parser = Parser::New( &mut stream);
    
    {
        let  	mut forge = Forge { _Parent: None, _Parser: &mut parser };
        
        // Test char grammar
        assert!( 'h'.Match( forge.Parser()));
        assert!( 'e'.Match( forge.Parser()));

        // Test &str grammar
        assert!( "llo ".Match( forge.Parser()));

        // Test charset grammar
        let  	mut cs = Charset::New();
        cs.SetChar( b'p');
        assert!( cs.Match( forge.Parser()));
        
        // Test failing match (should rollback)
        assert!( !"fail".Match( forge.Parser()));
        
        // Test continuing after fail
        assert!( "arser".Match( forge.Parser()));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[test]
fn	TestBacktrackingParser()
{
    // Test alternative 1 success
    {
        let  	data = [U8( b'a'), U8( b'b'), U8( b'c'), U8( b'd')];
        let  	arr = Arr::from( &data[..]);
        let  	mut stream = InStream::FromArr( arr);
        let  	mut parser = Parser::New( &mut stream);
        let  	tree = crate::ShardTree!( ( "ab" < "cd" ) | ( "a" < "bc" ));
        let  	dynNode: &DynINode<'_> = &tree;
        assert!( dynNode.Match( &mut parser));
    }

    // Test alternative 2 success with backtracking
    {
        let  	data = [U8( b'a'), U8( b'b'), U8( b'c')];
        let  	arr = Arr::from( &data[..]);
        let  	mut stream = InStream::FromArr( arr);
        let  	mut parser = Parser::New( &mut stream);
        let  	tree = crate::ShardTree!( ( "ab" < "cd" ) | ( "a" < "bc" ));
        let  	dynNode: &DynINode<'_> = &tree;
        assert!( dynNode.Match( &mut parser));
    }

    // Test ancestor lookup
    {
        let  	mut stream = InStream::FromArr( Arr::from( &[U8( 0)][..]));
        let  	mut dummyStream = InStream::FromArr( Arr::from( &[U8( 0)][..]));
        let  	mut dummyParser = Parser::New( &mut dummyStream);
        let  	mut parser = Parser::New( &mut stream);
        let  	forge = LeafForge {
            _Parent: None,
            _Parser: &mut parser,
            _Shard: None,
        };
        
        let  	parentPtr = &forge as *const LeafForge<'_, '_, '_, _> as *const dyn IForge<'_, '_, '_, _>;
        let  	parent = unsafe { &*parentPtr };
        
        let  	childForge = LeafForge {
            _Parent: Some( parent),
            _Parser: &mut dummyParser,
            _Shard: None,
        };
        
        let  	mut found = false;
        let  	ancestor = childForge.FindAncestor( &mut |_a| {
            found = true;
            true
        });
        assert!( found);
        assert!( ancestor.is_some());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
