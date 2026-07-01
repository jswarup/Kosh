//-- _tests.rs ----------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::InStream,
    segue::{ Charset, shard::Shard, Parser, IGrammar, parser::{IForge, Forge, BinOpForge} },
    silo::{ U8, U32, Arr},
    stalks::DynINode,
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
 
#[test]
fn	TestParserBasic()
{
    
    let  	data = [U8( b'h'), U8( b'e'), U8( b'l'), U8( b'l'), U8( b'o'), U8( b' '), U8( b'p'), U8( b'a'), U8( b'r'), U8( b's'), U8( b'e'), U8( b'r')];
    let  	arr = Arr::from( &data[..]);
    let  	mut stream = InStream::FromArr( arr);
    let  	mut parser = Parser::New( &mut stream);
    
    {
        let  	mut forge = Forge { _Parent: None, _Offset: U32( 0), _Parser: &mut parser };
        
        // Test char grammar
        assert!( 'h'.Match( forge.GetParser()));
        assert!( 'e'.Match( forge.GetParser()));

        // Test &str grammar
        assert!( "llo ".Match( forge.GetParser()));

        // Test charset grammar
        let  	mut cs = Charset::New();
        cs.SetChar( b'p');
        assert!( cs.Match( forge.GetParser()));
        
        // Test failing match (should rollback)
        assert!( !"fail".Match( forge.GetParser()));
        
        // Test continuing after fail
        assert!( "arser".Match( forge.GetParser()));
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
        let  	mut dummyStream = InStream::FromArr( Arr::from( &[U8( 0)][..]));
        let  	mut dummyParser = Parser::New( &mut dummyStream);
        
        let  	mut stream = InStream::FromArr( Arr::from( &[U8( 0)][..]));
        let  	mut parser = Parser::New( &mut stream);
        let  	forge = BinOpForge {
            _Parent: None,
            _Offset: U32( 0),
            _Parser: &mut parser,
            _LeftDigest: std::cell::Cell::new( None),
            _RightDigest: std::cell::Cell::new( None),
        };
        
        let  	parentPtr = &forge as *const BinOpForge<'_, '_, '_, _> as *const dyn IForge<'_, '_, '_, _>;
        let  	parent = unsafe { &*parentPtr };
        
        let  	childForge = BinOpForge {
            _Parent: Some( parent),
            _Offset: U32( 1),
            _Parser: &mut dummyParser,
            _LeftDigest: std::cell::Cell::new( None),
            _RightDigest: std::cell::Cell::new( None),
        };
        
        let  	mut found = false;
        let  	ancestor = childForge.FindAncestor( &mut |a| {
            if a.Offset() == U32( 0) {
                found = true;
                return true;
            }
            false
        });
        assert!( found);
        assert!( ancestor.is_some());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
