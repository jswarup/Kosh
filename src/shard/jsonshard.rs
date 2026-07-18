//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    ShardTree,
    flux::{ IFluxExportSource, IFluxImportSource, fluxexport::FieldExp, fluximport::FieldImp },
    shard::{ Charset, IGrammar, Parser, WSpc, Str },
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
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	res = JsonShard::MatchValue( parser);
        if let Some( newM) = res {
            let  	nextM = if let Some( m2) = parser.ParseGrammar( &WSpc(), newM) { m2 } else { newM };
            parser.SetCurrMark( nextM);
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl JsonShard
{


    fn	MatchValue<'a>( parser: &mut Parser) -> Option< U32>
    {
        let mut m = parser.CurrMark();
        if let Some( newM) = parser.ParseGrammar( &WSpc(), m) {
            m = newM;
            parser.SetCurrMark( m);
        }

        let  	curr = parser.GetAt( m);
        if curr == U8( b'{') {
            return Self::MatchObject( parser);
        } 
        if curr == U8( b'[') {
            return Self::MatchArray( parser);
        }
        let     shardTree = ShardTree!( Str | "true" | "false" | "null");
        if let Some( newM) = parser.ParseGrammar( &shardTree, m) {
            return Some( newM);
        } 
        if curr == U8( b'-') || ( curr >= U8( b'0') && curr <= U8( b'9')) {
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

        m = if let Some( newM) = parser.ParseGrammar( &WSpc(), m) { newM } else { m };
        if parser.GetAt( m) == U8( b']') {
            return parser.Incr( m);
        }

        loop { 
            parser.SetCurrMark( m);
            if let Some( nxt) = Self::MatchValue( parser) {
                m = nxt;
            } else {
                return None;
            }

            m = if let Some( newM) = parser.ParseGrammar( &WSpc(), m) { newM } else { m };
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

        m = if let Some( newM) = parser.ParseGrammar( &WSpc(), m) { newM } else { m };
        if parser.GetAt( m) == U8( b'}') {
            return parser.Incr( m);
        }

        loop {
            m = if let Some( newM) = parser.ParseGrammar( &WSpc(), m) { newM } else { m };
            let  	key_start = m + crate::silo::U32( 1);
            let nextM = parser.ParseGrammar( &Str, m)?;
            let  	key_end = nextM - crate::silo::U32( 1);
            m = nextM;
            m = if let Some( newM) = parser.ParseGrammar( &WSpc(), m) { newM } else { m };

            if parser.GetAt( m) != U8( b':') {
                return None;
            }
            m = if let  	Some( nxt) = parser.Incr( m) { nxt } else {
                return None;
            };
 
            parser.SetCurrMark( m);
            if let Some( nxt) = Self::MatchValue( parser) {
                m = nxt;
            } else {
                return None;
            }
            m = if let Some( newM) = parser.ParseGrammar( &WSpc(), m) { newM } else { m };

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
