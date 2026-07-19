//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    ShardTree,

    shard::{ IGrammar, Parser, WSpc },
    silo::{ U32, U8},
};
use	crate::shard::numbers::Real;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JsonShard;
pub const Json: JsonShard = JsonShard;

//---------------------------------------------------------------------------------------------------------------------------------



impl IGrammar for JsonShard
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	res = JsonShard::MatchValue( parser);
        if let Some( newM) = res {
            let  	nextM = if let Some( m2) = parser.ParseGrammar( &WSpc, newM) { m2 } else { newM };
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
        let  	mut m = parser.CurrMark();
        if let Some( newM) = parser.ParseGrammar( &WSpc, m) {
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
        let  	shardTree = ShardTree!( Str | "true" | "false" | "null" | Real );
        if let Some( newM) = parser.ParseGrammar( &shardTree, m) {
            return Some( newM);
        }  
        None
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchArray<'a>( parser: &mut Parser) -> Option< U32>
    {
        let  	m = parser.CurrMark();
        let  	shardTree = ShardTree!( '[' < *(*WSpc < ( |p: &mut Parser| Self::MatchValue( p).is_some()) < ? ( ',' < *WSpc)) < *WSpc < ']');
        if let Some( newM) = parser.ParseGrammar( &shardTree, m) {
            return Some( newM);
        }  
        None 
    }

    //---------------------------------------------------------------------------------------------------------------------------------

    fn	MatchObject<'a>( parser: &mut Parser) -> Option< U32>
    {
        let  	m = parser.CurrMark();
        let  	shardTree = ShardTree!( '{' < *(*WSpc < Str < *WSpc < ':' < *WSpc < ( |p: &mut Parser| Self::MatchValue( p).is_some()) < ? ( ',' < *WSpc)) < *WSpc < '}');
        if let Some( newM) = parser.ParseGrammar( &shardTree, m) {
            return Some( newM);
        }  
        None 
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for JsonShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }
impl fmt::Debug for JsonShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }

//---------------------------------------------------------------------------------------------------------------------------------

