//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ IGrammar, Parser };
use	crate::silo::{ U32, U8, IVoidPtrExt };
use	crate::stalks::INode;
use	crate::shard::numbers::Real;

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
    fn	MatchGrammar( &self, parser: *mut (), marker: u32) -> Option< u32>
{
        let  	parserRef = parser.MutRef::< Parser< '_>>();
        return self.Match( parserRef, U32( marker)).map( |u| u.0);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for JsonShard
{
    fn	Match< 'p>( &'p self, parser: &mut Parser< 'p>, marker: U32) -> Option< U32>
{
        if let  	Some( m) = JsonShard::MatchValue( parser, marker) {
            Some( JsonShard::SkipWhitespace( parser, m))
        } else {
            None
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl JsonShard
{
    fn	SkipWhitespace< 'p>( parser: &mut Parser< 'p>, mut marker: U32) -> U32
{
        loop {
            let  	curr = parser.Curr( marker);
            if curr == U8( b' ') || curr == U8( b'\n') || curr == U8( b'\r') || curr == U8( b'\t') {
                if let  	Some( nextMark) = parser.Next( marker) {
                    marker = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        marker
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchString< 'p>( parser: &mut Parser< 'p>, marker: U32) -> Option< U32>
    {
        let  	curr = parser.Curr( marker);
        if curr != U8( b'"') {
            return None;
        }
        
        if let  	Some( mut m) = parser.Next( marker) {
            let  	mut escape = false;
            loop {
                // If we reach the end of the stream without closing quote, return None.
                // U8( 0) might technically be valid in some contexts but usually not in JSON strings.
                // We'll rely on parser.Next returning None if we go out of bounds.
                let  	c = parser.Curr( m);
                if c == U8( 0) && m.0 as usize >= parser.InStream().Size() {
                    return None;
                }

                if escape {
                    escape = false;
                } else if c == U8( b'\\') {
                    escape = true;
                } else if c == U8( b'"') {
                    return parser.Next( m);
                }
                
                if let  	Some( nxt) = parser.Next( m) {
                    m = nxt;
                } else {
                    return None; // EOF before closing quote
                }
            }
        }
        None
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchKeyword< 'p>( parser: &mut Parser< 'p>, marker: U32, keyword: &[u8]) -> Option< U32>
    {
        let  	mut m = marker;
        for &b in keyword {
            if parser.Curr( m) != U8( b) {
                return None;
            }
            if let  	Some( nxt) = parser.Next( m) {
                m = nxt;
            } else {
                // If it's the last character of the stream and the keyword matches,
                // nxt is Some( m+1) because Size includes it. So if Next fails, it really failed.
                return None;
            }
        }
        Some( m)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchValue< 'p>( parser: &mut Parser< 'p>, marker: U32) -> Option< U32>
    {
        let  	m = Self::SkipWhitespace( parser, marker);
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
            // JsonOutStream writes "null" as a string, which MatchString handles,
            // but it can also write unquoted null in some contexts, so we support it.
            return Self::MatchKeyword( parser, m, b"null");
        } else if curr == U8( b'-') || ( curr >= U8( b'0') && curr <= U8( b'9')) {
            return Real.Match( parser, m);
        }
        
        None
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchArray< 'p>( parser: &mut Parser< 'p>, mut marker: U32) -> Option< U32>
    {
        if parser.Curr( marker) != U8( b'[') {
            return None;
        }
        marker = if let  	Some( nxt) = parser.Next( marker) { nxt } else {
            return None;
        };
        
        marker = Self::SkipWhitespace( parser, marker);
        if parser.Curr( marker) == U8( b']') {
            return parser.Next( marker);
        }
        
        loop {
            marker = Self::MatchValue( parser, marker)?;
            marker = Self::SkipWhitespace( parser, marker);
            let  	curr = parser.Curr( marker);
            if curr == U8( b',') {
                marker = if let  	Some( nxt) = parser.Next( marker) { nxt } else {
                    return None;
                };
            } else if curr == U8( b']') {
                return parser.Next( marker);
            } else {
                return None;
            }
        }
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchObject< 'p>( parser: &mut Parser< 'p>, mut marker: U32) -> Option< U32>
    {
        if parser.Curr( marker) != U8( b'{') {
            return None;
        }
        marker = if let  	Some( nxt) = parser.Next( marker) { nxt } else {
            return None;
        };
        
        marker = Self::SkipWhitespace( parser, marker);
        if parser.Curr( marker) == U8( b'}') {
            return parser.Next( marker);
        }
        
        loop {
            marker = Self::SkipWhitespace( parser, marker);
            marker = Self::MatchString( parser, marker)?;
            marker = Self::SkipWhitespace( parser, marker);
            
            if parser.Curr( marker) != U8( b':') {
                return None;
            }
            marker = if let  	Some( nxt) = parser.Next( marker) { nxt } else {
                return None;
            };
            
            marker = Self::MatchValue( parser, marker)?;
            marker = Self::SkipWhitespace( parser, marker);
            
            let  	curr = parser.Curr( marker);
            if curr == U8( b',') {
                marker = if let  	Some( nxt) = parser.Next( marker) { nxt } else {
                    return None;
                };
            } else if curr == U8( b'}') {
                return parser.Next( marker);
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
