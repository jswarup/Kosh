//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    ShardTree,
    flux::{ IFluxExportSource, IFluxImportSource, fluxexport::FieldExp, fluximport::FieldImp },
    shard::{ Charset, IGrammar, Parser, IForge, WSpc },
    silo::{ U32, U64, U8},
};
use	crate::shard::numbers::Real;

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
    fn	Match( &self, parser: &mut Parser)
    {
        let  	res = JsonShard::MatchValue( parser);
        if let Some( newM) = res {
            let  	nextM = JsonShard::SkipWhitespace( parser, newM);
            parser.Deposit( Some( nextM));
        } else {
            parser.Deposit( None);
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

    fn	MatchString<'a>( parser: &mut Parser, marker: U32) -> (bool, U32)
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

    fn	MatchKeyword<'a>( parser: &mut Parser, marker: U32, keyword: &[u8]) -> (bool, U32)
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

    fn	MatchValue<'a>( parser: &mut Parser) -> Option< U32>
    {
        let mut m = parser.CurrMark();
        if let Some( newM) = parser.ParseGrammar( &WSpc(), m) {
            m = newM;
            parser.Deposit(Some(m));
        }

        let  	curr = parser.GetAt( m);

        if curr == U8( b'{') {
            return Self::MatchObject( parser);
        } else if curr == U8( b'[') {
            return Self::MatchArray( parser);
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
            if let Some( nextM) = parser.ParseGrammar( &Real, m) {
                return Some( nextM);
            }
            return None;
        }

        None
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchArray<'a>( parser: &mut Parser) -> Option< U32>
    {
        let mut m = parser.CurrMark();
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
            parser.Deposit(Some(m));
            if let Some( nxt) = Self::MatchValue( parser) {
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

    fn	MatchObject<'a>( parser: &mut Parser) -> Option< U32>
    {
        let mut m = parser.CurrMark();
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
            let ( matched, nextM) = Self::MatchString( parser, m);
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
 
            parser.Deposit(Some(m));
            if let Some( nxt) = Self::MatchValue( parser) {
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
