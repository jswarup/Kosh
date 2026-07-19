//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    ShardTree,

    shard::{ IGrammar, Parser, WSpc },
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
        let  	m = parser.CurrMark(); 
        let  	objShard = ShardTree!( '{' < *(*WSpc < Str < *WSpc < ':' < *WSpc < ( |p: &mut Parser| p.ParseGrammar( &Json, p.CurrMark()).is_some()) < ? ( ',' < *WSpc)) < *WSpc < '}');
        let  	arrShard = ShardTree!( '[' < *(*WSpc < ( |p: &mut Parser| p.ParseGrammar( &Json, p.CurrMark()).is_some()) < ? ( ',' < *WSpc)) < *WSpc < ']');
        let  	keyShard = ShardTree!( Str | "true" | "false" | "null" | Real );
        let  	valShard = ShardTree!( *WSpc < ( keyShard | arrShard | objShard) );
        
        if let Some( newM) = parser.ParseGrammar( &valShard, m) {
            let  	nextM = if let Some( m2) = parser.ParseGrammar( &WSpc, newM) { m2 } else { newM };
            parser.SetCurrMark( nextM);
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for JsonShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }
impl fmt::Debug for JsonShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }

//---------------------------------------------------------------------------------------------------------------------------------

