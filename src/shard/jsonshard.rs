//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    ShardTree, 
    shard::{ IGrammar, Parser, WSpc },
    silo::{  Arr, U8 },
};
use	crate::shard::numbers::Real;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JsonShard
{

}

pub const Json: JsonShard = JsonShard{};

//---------------------------------------------------------------------------------------------------------------------------------

impl JsonShard
{
    fn	MatchObject( parser: &mut Parser) -> bool
    {
        
        let mut     strBuf = String::from( "");
        let     objectName = move | arr: Arr< U8>| {
            strBuf.push_str( <&str>::from( arr));
            true
        };
        let     objectValue = | _arr: Arr< U8>| {
            true
        };
        let     objShard = ShardTree!( Str[ objectName] < ?WSpc < ':' < ?WSpc < ( JsonShard::MatchValue)[ objectValue]);
        
        if let Some( newM) = parser.ParseGrammar( &objShard, parser.CurrMark()) { 
            parser.SetCurrMark( newM);
            true
        } else {
            false
        }
    }

    fn	MatchValue( parser: &mut Parser) -> bool
    { 
        let  	objShard = ShardTree!( '{' < *(?WSpc < ( JsonShard::MatchObject) < ? ( ',' < ?WSpc)) < ?WSpc < '}');
        let  	arrShard = ShardTree!( '[' < *(?WSpc < ( JsonShard::MatchValue) < ? ( ',' < ?WSpc)) < ?WSpc < ']');
        let  	keyShard = ShardTree!( Str | "true" | "false" | "null" | Real );
        let  	valShard = ShardTree!( ?WSpc < ( keyShard | arrShard | objShard) < ?WSpc);
        
        if let Some( newM) = parser.ParseGrammar( &valShard, parser.CurrMark()) { 
            parser.SetCurrMark( newM);
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for JsonShard
{
    
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	m = parser.CurrMark();   
        let  	jsonhard = ShardTree!( ?WSpc < ( JsonShard::MatchValue) < ?WSpc); 
        if let Some( newM) = parser.ParseGrammar( &jsonhard, m) { 
            parser.SetCurrMark( newM);
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

