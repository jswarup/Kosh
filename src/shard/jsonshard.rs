//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ Charset, IGrammar, Parser, IForge };
use	crate::silo::{U32, U8};
use	crate::shard::numbers::Real;
use	crate::shard::WSpc;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JsonShard;
pub const Json: &JsonShard = &JsonShard;

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for JsonShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
{
        *field = XField::String( "Json".to_string());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for JsonShard
{
    fn	Match( &self, parser: &mut crate::shard::Parser)
    {
        let  	mark = parser.Forge().Mark();
        let  	res = JsonShard::MatchValue( parser, mark);
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

    fn	MatchString( parser: &mut crate::shard::Parser, marker: U32) -> (bool, U32)
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

    fn	MatchKeyword( parser: &mut crate::shard::Parser, marker: U32, keyword: &[u8]) -> (bool, U32)
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
        ( true, m)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchValue( parser: &mut crate::shard::Parser, mut m: U32) -> Option< U32>
    {
        if let Some( newM) = WSpc().Parse( parser, m) {
            m = newM;
        }
        
        let  	curr = parser.GetAt( m);
        
        if curr == U8( b'{') {
            return Self::MatchObject( parser, m);
        } else if curr == U8( b'[') {
            return Self::MatchArray( parser, m);
        } else if curr == U8( b'"') {
            let ( matched, nextM) = Self::MatchString( parser, m);
            if matched { return Some( nextM); }
            return None;
        } else if curr == U8( b't') {
            let ( matched, nextM) = Self::MatchKeyword( parser, m, b"true");
            if matched { return Some( nextM); }
            return None;
        } else if curr == U8( b'f') {
            let ( matched, nextM) = Self::MatchKeyword( parser, m, b"false");
            if matched { return Some( nextM); }
            return None;
        } else if curr == U8( b'n') {
            let ( matched, nextM) = Self::MatchKeyword( parser, m, b"null");
            if matched { return Some( nextM); }
            return None;
        } else if curr == U8( b'-') || ( curr >= U8( b'0') && curr <= U8( b'9')) {
            if let Some( nextM) = Real.Parse( parser, m) {
                return Some( nextM);
            }
            return None;
        }
        
        None
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchArray( parser: &mut crate::shard::Parser, mut m: U32) -> Option< U32>
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
            if let Some( nxt) = Self::MatchValue( parser, m) {
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

    fn	MatchObject( parser: &mut crate::shard::Parser, mut m: U32) -> Option< U32>
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
            let ( matched, nextM) = Self::MatchString( parser, m);
            if !matched {
                return None;
            }
            m = nextM;
            m = Self::SkipWhitespace( parser, m);
            
            if parser.GetAt( m) != U8( b':') {
                return None;
            }
            m = if let  	Some( nxt) = parser.Incr( m) { nxt } else {
                return None;
            };
            
            if let Some( nxt) = Self::MatchValue( parser, m) {
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
