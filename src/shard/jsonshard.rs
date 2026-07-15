//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::flux::{ IFluxImportSource };
use	crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use	crate::flux::fluximport::FieldImp;
use	crate::shard::{ Charset, IGrammar, Parser, IForge };
use	crate::silo::{U32, U8};
use	crate::shard::numbers::Real;
use	crate::shard::WSpc;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JsonShard;
pub const Json: &JsonShard = &JsonShard;

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for JsonShard
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
{
        *field = FieldExp::String( "Json".to_string());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for JsonShard
{
    fn	Match( &self, parser: &mut Parser, sink: FieldImp< '_>)
    {
        let  	mark = parser.Forge().Mark();
        let  	res = JsonShard::MatchValue( parser, mark, sink);
        if let Some( newM) = res {
            let  	nextM = JsonShard::SkipWhitespace( parser, newM);
            parser.Forge().Deposit( Some( nextM));
        } else {
            parser.Forge().Deposit( None);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl JsonShard
{

    fn	SkipWhitespace< 'p>( parser: &mut Parser< 'p>, marker: U32) -> U32
    {
        let  	whiteSpace = Charset::Space();
        let  	mut m = marker;
        loop {
            let  	curr = parser.GetAt( m);
            if whiteSpace.Get( curr) {
                if let  	Some( nextMark) = parser.Incr( m) {
                    m = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        m
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchString<'a>( parser: &mut Parser, marker: U32, mut sink: FieldImp< 'a>) -> (bool, U32)
    {
        let  	mut m = marker;
        let  	curr = parser.GetAt( m);
        if curr != U8( b'"') {
            return ( false, marker);
        }
        
        if let  	Some( next) = parser.Incr( m) {
            m = next;
            let  	mut escape = false;
            loop {
                let  	c = parser.GetAt( m);
                if c == U8( 0) && m >= parser.InStream().Size() {
                    return ( false, marker);
                }

                if escape {
                    escape = false;
                } else if c == U8( b'\\') {
                    escape = true;
                } else if c == U8( b'"') {
                    if let Some( nxt) = parser.Incr( m) {
                        sink.Resolve();
                        if matches!( sink, FieldImp::String( _) | FieldImp::Str( _) | FieldImp::FluxSink( _) | FieldImp::ExpectedType( _)) {
                            let  	bytes = parser.InStream().BytesAt( marker + crate::silo::U32( 1), m - marker - crate::silo::U32( 1));
                            if let  	Ok( s) = std::str::from_utf8( bytes) {
                                if let  	FieldImp::String( dst) = sink {
                                    *dst = s.to_string();
                                } else if let  	FieldImp::FluxSink( flx) = sink {
                                    let  	mut temp = s.to_string();
                                    flx.FromFieldImp( FieldImp::String( &mut temp));
                                } else if let   FieldImp::ExpectedType( exp) = sink {
                                    assert_eq!( s, exp, "Type mismatch during import. Expected '{}', got '{}'", exp, s);
                                }
                            }
                        }
                        return ( true, nxt);
                    } else {
                        return ( false, marker);
                    }
                }
                
                if let  	Some( nxt) = parser.Incr( m) {
                    m = nxt;
                } else {
                    return ( false, marker);
                }
            }
        }
        ( false, marker)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchKeyword<'a>( parser: &mut Parser, marker: U32, keyword: &[u8], mut sink: FieldImp< 'a>) -> (bool, U32)
    {
        let  	mut m = marker;
        for &b in keyword {
            if parser.GetAt( m) != U8( b) {
                return ( false, marker);
            }
            if let  	Some( nxt) = parser.Incr( m) {
                m = nxt;
            } else {
                return ( false, marker);
            }
        }
        sink.Resolve();
        if let  	FieldImp::Bool( dst) = sink {
            if keyword == b"true" { *dst = true; }
            else if keyword == b"false" { *dst = false; }
        } else if let  	FieldImp::FluxSink( flx) = sink {
            if keyword == b"true" {
                let  	mut temp = true;
                flx.FromFieldImp( FieldImp::Bool( &mut temp));
            } else if keyword == b"false" {
                let  	mut temp = false;
                flx.FromFieldImp( FieldImp::Bool( &mut temp));
            } else if keyword == b"null" {
                flx.FromFieldImp( FieldImp::Null);
            }
        }
        ( true, m)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchValue<'a>( parser: &mut Parser, mut m: U32, sink: FieldImp< 'a>) -> Option< U32>
    {
        if let Some( newM) = WSpc().Parse( parser, m, FieldImp::Null) {
            m = newM;
        }
        
        let  	curr = parser.GetAt( m);
        
        if curr == U8( b'{') {
            return Self::MatchObject( parser, m, sink);
        } else if curr == U8( b'[') {
            return Self::MatchArray( parser, m, sink);
        } else if curr == U8( b'"') {
            let ( matched, nextM) = Self::MatchString( parser, m, sink);
            if matched { return Some( nextM); }
            return None;
        } else if curr == U8( b't') {
            let ( matched, nextM) = Self::MatchKeyword( parser, m, b"true", sink);
            if matched { return Some( nextM); }
            return None;
        } else if curr == U8( b'f') {
            let ( matched, nextM) = Self::MatchKeyword( parser, m, b"false", sink);
            if matched { return Some( nextM); }
            return None;
        } else if curr == U8( b'n') {
            let ( matched, nextM) = Self::MatchKeyword( parser, m, b"null", sink);
            if matched { return Some( nextM); }
            return None;
        } else if curr == U8( b'-') || ( curr >= U8( b'0') && curr <= U8( b'9')) {
            if let Some( nextM) = Real.Parse( parser, m, sink) {
                return Some( nextM);
            }
            return None;
        }
        
        None
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchArray<'a>( parser: &mut Parser, mut m: U32, mut sink: FieldImp< 'a>) -> Option< U32>
    {
        if parser.GetAt( m) != U8( b'[') {
            return None;
        }
        m = if let  	Some( nxt) = parser.Incr( m) { nxt } else {
            return None;
        };
        
        m = Self::SkipWhitespace( parser, m);
        if parser.GetAt( m) == U8( b']') {
            return parser.Incr( m);
        }
        
        loop {
            
            sink.Resolve();
            let  	mut temp_sink = FieldImp::Null;
            std::mem::swap( &mut temp_sink, &mut sink);
            
            let  	mut child_sink = FieldImp::Null;
            if let FieldImp::Arr( ref mut closure) = temp_sink {
                closure( &mut child_sink);
            }
            std::mem::swap( &mut temp_sink, &mut sink);

            if let Some( nxt) = Self::MatchValue( parser, m, child_sink) {
                m = nxt;
            } else {
                return None;
            }
            
            m = Self::SkipWhitespace( parser, m);
            let  	curr = parser.GetAt( m);
            if curr == U8( b',') {
                m = if let  	Some( nxt) = parser.Incr( m) { nxt } else {
                    return None;
                };
            } else if curr == U8( b']') {
                return parser.Incr( m);
            } else {
                return None;
            }
        }
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchObject<'a>( parser: &mut Parser, mut m: U32, mut sink: FieldImp< 'a>) -> Option< U32>
    {
        if parser.GetAt( m) != U8( b'{') {
            return None;
        }
        m = if let  	Some( nxt) = parser.Incr( m) { nxt } else {
            return None;
        };
        
        m = Self::SkipWhitespace( parser, m);
        if parser.GetAt( m) == U8( b'}') {
            return parser.Incr( m);
        }
        
        loop {
            m = Self::SkipWhitespace( parser, m);
            let  	key_start = m + crate::silo::U32( 1);
            let ( matched, nextM) = Self::MatchString( parser, m, FieldImp::Null);
            if !matched {
                return None;
            }
            let  	key_end = nextM - crate::silo::U32( 1);
            m = nextM;
            m = Self::SkipWhitespace( parser, m);
            
            if parser.GetAt( m) != U8( b':') {
                return None;
            }
            m = if let  	Some( nxt) = parser.Incr( m) { nxt } else {
                return None;
            };
            
            
            sink.Resolve();
            let  	mut temp_sink = FieldImp::Null;
            std::mem::swap( &mut temp_sink, &mut sink);
            
            let  	mut child_sink = FieldImp::Null;
            if let FieldImp::Obj( ref mut closure) = temp_sink {
                let  	bytes = parser.InStream().BytesAt( key_start, key_end - key_start);
                if let  	Ok( s) = std::str::from_utf8( bytes) {
                    closure( s, &mut child_sink);
                }
            }
            std::mem::swap( &mut temp_sink, &mut sink);

            if let Some( nxt) = Self::MatchValue( parser, m, child_sink) {
                m = nxt;
            } else {
                return None;
            }
            m = Self::SkipWhitespace( parser, m);
            
            let  	curr = parser.GetAt( m);
            if curr == U8( b',') {
                m = if let  	Some( nxt) = parser.Incr( m) { nxt } else {
                    return None;
                };
            } else if curr == U8( b'}') {
                return parser.Incr( m);
            } else {
                return None;
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for JsonShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }
impl fmt::Debug for JsonShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxImportSource!( JsonShard);
