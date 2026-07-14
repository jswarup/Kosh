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
    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F)
    {
        let  	res = JsonShard::MatchValue( forge);
        if let Some( newM) = res {
            let  	nextM = JsonShard::SkipWhitespace( forge.Parser(), newM);
            forge.Deposit( Some( nextM));
        } else {
            forge.Deposit( None);
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
            let  	curr = parser.Curr( m);
            if whiteSpace.Get( curr) {
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
        let  	mut m = marker;
        let  	curr = parser.Curr( m);
        if curr != U8( b'"') {
            return ( false, marker);
        }
        
        if let  	Some( next) = parser.Next( m) {
            m = next;
            let  	mut escape = false;
            loop {
                let  	c = parser.Curr( m);
                if c == U8( 0) && m.0 as usize >= parser.InStream().Size() {
                    return ( false, marker);
                }

                if escape {
                    escape = false;
                } else if c == U8( b'\\') {
                    escape = true;
                } else if c == U8( b'"') {
                    if let Some( nxt) = parser.Next( m) {
                        return ( true, nxt);
                    } else {
                        return ( false, marker);
                    }
                }
                
                if let  	Some( nxt) = parser.Next( m) {
                    m = nxt;
                } else {
                    return ( false, marker);
                }
            }
        }
        ( false, marker)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchKeyword< 'p>( parser: &mut Parser< 'p>, marker: U32, keyword: &[u8]) -> (bool, U32)
    {
        let  	mut m = marker;
        for &b in keyword {
            if parser.Curr( m) != U8( b) {
                return ( false, marker);
            }
            if let  	Some( nxt) = parser.Next( m) {
                m = nxt;
            } else {
                return ( false, marker);
            }
        }
        ( true, m)
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchValue< 'p, F: IForge< 'p>>( forge: &mut F) -> Option< U32>
    {
        let  	mut m = forge.Mark();
        {
            let  	wspc = WSpc();
            let  	mut wspcForge = wspc.Forge( forge.Parser());
            wspcForge.SetMark( m);
            wspc.Match( &mut wspcForge);
            if wspcForge.Result().is_some() {
                m = wspcForge.Mark();
            }
        }
        
        let  	curr = forge.Parser().Curr( m);
        
        if curr == U8( b'{') {
            forge.SetMark( m);
            return Self::MatchObject( forge);
        } else if curr == U8( b'[') {
            forge.SetMark( m);
            return Self::MatchArray( forge);
        } else if curr == U8( b'"') {
            let ( matched, nextM) = Self::MatchString( forge.Parser(), m);
            if matched {
                forge.SetMark( nextM);
                return Some( nextM);
            }
            return None;
        } else if curr == U8( b't') {
            let ( matched, nextM) = Self::MatchKeyword( forge.Parser(), m, b"true");
            if matched {
                forge.SetMark( nextM);
                return Some( nextM);
            }
            return None;
        } else if curr == U8( b'f') {
            let ( matched, nextM) = Self::MatchKeyword( forge.Parser(), m, b"false");
            if matched {
                forge.SetMark( nextM);
                return Some( nextM);
            }
            return None;
        } else if curr == U8( b'n') {
            let ( matched, nextM) = Self::MatchKeyword( forge.Parser(), m, b"null");
            if matched {
                forge.SetMark( nextM);
                return Some( nextM);
            }
            return None;
        } else if curr == U8( b'-') || ( curr >= U8( b'0') && curr <= U8( b'9')) {
            forge.SetMark( m);
            let  	real = Real;
            let  	mut realForge = real.Forge( forge.Parser());
            realForge.SetMark( m);
            real.Match( &mut realForge);
            if realForge.Result().is_some() {
                let  	nextM = realForge.Mark();
                return Some( nextM);
            }
            return None;
        }
        
        None
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchArray< 'p, F: IForge< 'p>>( forge: &mut F) -> Option< U32>
    {
        let  	mut m = forge.Mark();
        if forge.Parser().Curr( m) != U8( b'[') {
            return None;
        }
        m = if let  	Some( nxt) = forge.Parser().Next( m) { nxt } else {
            return None;
        };
        
        m = Self::SkipWhitespace( forge.Parser(), m);
        if forge.Parser().Curr( m) == U8( b']') {
            if let Some( nxt) = forge.Parser().Next( m) {
                forge.SetMark( nxt);
                return Some( nxt);
            } else {
                return None;
            }
        }
        
        forge.SetMark( m);
        loop {
            let  	nextM = Self::MatchValue( forge);
            if let Some( nxt) = nextM {
                m = nxt;
            } else {
                return None;
            }
            
            m = Self::SkipWhitespace( forge.Parser(), m);
            let  	curr = forge.Parser().Curr( m);
            if curr == U8( b',') {
                m = if let  	Some( nxt) = forge.Parser().Next( m) { nxt } else {
                    return None;
                };
                forge.SetMark( m);
            } else if curr == U8( b']') {
                if let Some( nxt) = forge.Parser().Next( m) {
                    forge.SetMark( nxt);
                    return Some( nxt);
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchObject< 'p, F: IForge< 'p>>( forge: &mut F) -> Option< U32>
    {
        let  	mut m = forge.Mark();
        if forge.Parser().Curr( m) != U8( b'{') {
            return None;
        }
        m = if let  	Some( nxt) = forge.Parser().Next( m) { nxt } else {
            return None;
        };
        
        m = Self::SkipWhitespace( forge.Parser(), m);
        if forge.Parser().Curr( m) == U8( b'}') {
            if let Some( nxt) = forge.Parser().Next( m) {
                forge.SetMark( nxt);
                return Some( nxt);
            } else {
                return None;
            }
        }
        
        loop {
            m = Self::SkipWhitespace( forge.Parser(), m);
            let ( matched, nextM) = Self::MatchString( forge.Parser(), m);
            if !matched {
                return None;
            }
            m = nextM;
            m = Self::SkipWhitespace( forge.Parser(), m);
            
            if forge.Parser().Curr( m) != U8( b':') {
                return None;
            }
            m = if let  	Some( nxt) = forge.Parser().Next( m) { nxt } else {
                return None;
            };
            
            forge.SetMark( m);
            let  	nextValM = Self::MatchValue( forge);
            if let Some( nxt) = nextValM {
                m = nxt;
            } else {
                return None;
            }
            m = Self::SkipWhitespace( forge.Parser(), m);
            
            let  	curr = forge.Parser().Curr( m);
            if curr == U8( b',') {
                m = if let  	Some( nxt) = forge.Parser().Next( m) { nxt } else {
                    return None;
                };
            } else if curr == U8( b'}') {
                if let Some( nxt) = forge.Parser().Next( m) {
                    forge.SetMark( nxt);
                    return Some( nxt);
                } else {
                    return None;
                }
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
