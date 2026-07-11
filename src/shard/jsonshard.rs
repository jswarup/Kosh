//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ Charset, IGrammar, Parser };
use	crate::silo::{U32, U8, IVoidPtrExt};
use	crate::stalks::INode;
use	crate::shard::numbers::Real;
use	crate::WSpc;

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

impl< 'a> INode< 'a> for JsonShard
{
    fn	MatchGrammar( &self, parser: *mut (), marker: U32) -> (bool, U32)
{
        let  	parserRef = parser.MutRef::< Parser< '_>>();
        self.Match( parserRef, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for JsonShard
{
    fn	Match< 'p>( &'p self, parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        let (matched, new_mark) = JsonShard::MatchValue( parser, marker);
        if matched {
            (true, JsonShard::SkipWhitespace( parser, new_mark))
        } else {
            (false, marker)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl JsonShard
{

    fn	SkipWhitespace< 'p>( parser: &mut Parser< 'p>, marker: U32) -> U32
    {
        let     whiteSpace = Charset::Space();
        let mut m = marker;
        loop {
            let  	curr = parser.Curr( m);
            if whiteSpace.Get( curr)  {
                if let  	Some( nextMark) = parser.Next( m) {
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

    fn	MatchString< 'p>( parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        let mut m = marker;
        let  	curr = parser.Curr( m);
        if curr != U8( b'"') {
            return (false, marker);
        }
        
        if let  	Some( next) = parser.Next( m) {
            m = next;
            let  	mut escape = false;
            loop {
                // If we reach the end of the stream without closing quote, return false.
                let  	c = parser.Curr( m);
                if c == U8( 0) && m.0 as usize >= parser.InStream().Size() {
                    return (false, marker);
                }

                if escape {
                    escape = false;
                } else if c == U8( b'\\') {
                    escape = true;
                } else if c == U8( b'"') {
                    if let Some(nxt) = parser.Next( m) {
                        return (true, nxt);
                    } else {
                        return (false, marker);
                    }
                }
                
                if let  	Some( nxt) = parser.Next( m) {
                    m = nxt;
                } else {
                    return (false, marker); // EOF before closing quote
                }
            }
        }
        (false, marker)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchKeyword< 'p>( parser: &mut Parser< 'p>, marker: U32, keyword: &[u8]) -> (bool, U32)
    {
        let  	mut m = marker;
        for &b in keyword {
            if parser.Curr( m) != U8( b) {
                return (false, marker);
            }
            if let  	Some( nxt) = parser.Next( m) {
                m = nxt;
            } else {
                return (false, marker);
            }
        }
        (true, m)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchValue< 'p>( parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        let     ws = WSpc!();
        let     parser_ptr = parser as *mut _ as *mut ();
        let     (_, m) = ws.MatchGrammar( parser_ptr, marker);
        let  	curr = parser.Curr( m);
        
        if curr == U8( b'{') {
            return Self::MatchObject( parser, m);
        } else if curr == U8( b'[') {
            return Self::MatchArray( parser, m);
        } else if curr == U8( b'"') {
            return Self::MatchString( parser, m);
        } else if curr == U8( b't') {
            return Self::MatchKeyword( parser, m, b"true");
        } else if curr == U8( b'f') {
            return Self::MatchKeyword( parser, m, b"false");
        } else if curr == U8( b'n') {
            return Self::MatchKeyword( parser, m, b"null");
        } else if curr == U8( b'-') || ( curr >= U8( b'0') && curr <= U8( b'9')) {
            return Real.Match( parser, m);
        }
        
        (false, marker)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchArray< 'p>( parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        let mut m = marker;
        if parser.Curr( m) != U8( b'[') {
            return (false, marker);
        }
        m = if let  	Some( nxt) = parser.Next( m) { nxt } else {
            return (false, marker);
        };
        
        m = Self::SkipWhitespace( parser, m);
        if parser.Curr( m) == U8( b']') {
            if let Some(nxt) = parser.Next( m) {
                return (true, nxt);
            } else {
                return (false, marker);
            }
        }
        
        loop {
            let (matched, next_m) = Self::MatchValue( parser, m);
            if !matched {
                return (false, marker);
            }
            m = next_m;
            m = Self::SkipWhitespace( parser, m);
            let  	curr = parser.Curr( m);
            if curr == U8( b',') {
                m = if let  	Some( nxt) = parser.Next( m) { nxt } else {
                    return (false, marker);
                };
            } else if curr == U8( b']') {
                if let Some(nxt) = parser.Next( m) {
                    return (true, nxt);
                } else {
                    return (false, marker);
                }
            } else {
                return (false, marker);
            }
        }
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchObject< 'p>( parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        let mut m = marker;
        if parser.Curr( m) != U8( b'{') {
            return (false, marker);
        }
        m = if let  	Some( nxt) = parser.Next( m) { nxt } else {
            return (false, marker);
        };
        
        m = Self::SkipWhitespace( parser, m);
        if parser.Curr( m) == U8( b'}') {
            if let Some(nxt) = parser.Next( m) {
                return (true, nxt);
            } else {
                return (false, marker);
            }
        }
        
        loop {
            m = Self::SkipWhitespace( parser, m);
            let (matched, next_m) = Self::MatchString( parser, m);
            if !matched {
                return (false, marker);
            }
            m = next_m;
            m = Self::SkipWhitespace( parser, m);
            
            if parser.Curr( m) != U8( b':') {
                return (false, marker);
            }
            m = if let  	Some( nxt) = parser.Next( m) { nxt } else {
                return (false, marker);
            };
            
            let (matched, next_m) = Self::MatchValue( parser, m);
            if !matched {
                return (false, marker);
            }
            m = next_m;
            m = Self::SkipWhitespace( parser, m);
            
            let  	curr = parser.Curr( m);
            if curr == U8( b',') {
                m = if let  	Some( nxt) = parser.Next( m) { nxt } else {
                    return (false, marker);
                };
            } else if curr == U8( b'}') {
                if let Some(nxt) = parser.Next( m) {
                    return (true, nxt);
                } else {
                    return (false, marker);
                }
            } else {
                return (false, marker);
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for JsonShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }
impl fmt::Debug for JsonShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }

//---------------------------------------------------------------------------------------------------------------------------------
