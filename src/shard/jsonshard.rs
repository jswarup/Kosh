//-- jsonshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::{
    ShardTree, flux::FieldImp, shard::{ IGrammar, Parser, WSpc }, silo::{  Arr, Stash, U8 },
};
use	crate::shard::numbers::Real;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JSon< 'a>
{
    pub _ImpStash: Stash< FieldImp< 'a>>,
}
 

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> JSon< 'a>
{
    pub fn  New( mut docImp: FieldImp< 'a>) -> Self
    {
        let  	mut json  = Self {
            _ImpStash:  Stash::NewEmpty(),

        };
        json._ImpStash.PushX( &mut docImp);
        json
    }

    fn	MatchObject( &self, parser: &mut Parser) -> bool
    { 
        let mut objArr = Arr::< U8>::NewEmpty();
        let objectName = | arr: Arr< U8>| {
            objArr = arr.LifeFix();
            true
        }; 
        let mut valArr = Arr::< U8>::NewEmpty();
        let objectValue = | arr: Arr< U8>| {
            valArr = arr.LifeFix(); 
            true
        };
        let     objShard = ShardTree!( Str[ objectName] < ?WSpc < ':' < ?WSpc < ( |p: &mut Parser| self.MatchValue(p) )[ objectValue]);
        
        let Some( newM) = parser.ParseGrammar( &objShard, parser.CurrMark()) else { 
            return false;
        };
        parser.SetCurrMark( newM);
        true
    }

    fn	MatchValue( &self, parser: &mut Parser) -> bool
    { 
        let  	objShard = ShardTree!( '{' < *(?WSpc < ( |p: &mut Parser| self.MatchObject(p) ) < ? ( ',' < ?WSpc)) < ?WSpc < '}');
        let  	arrShard = ShardTree!( '[' < *(?WSpc < ( |p: &mut Parser| self.MatchValue(p) ) < ? ( ',' < ?WSpc)) < ?WSpc < ']');
        let  	keyShard = ShardTree!( Str | "true" | "false" | "null" | Real );
        let  	valShard = ShardTree!( ?WSpc < ( keyShard | arrShard | objShard) < ?WSpc);
        
        let Some( newM) = parser.ParseGrammar( &valShard, parser.CurrMark()) else { 
            return false;
        };
        parser.SetCurrMark( newM);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for JSon< 'a>
{
    
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	m = parser.CurrMark();   
        let  	jsonhard = ShardTree!( ?WSpc < ( |p: &mut Parser| self.MatchValue(p) ) < ?WSpc); 
        let Some( newM) = parser.ParseGrammar( &jsonhard, m) else { 
            return false;
        };
        parser.SetCurrMark( newM);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Display for JSon< 'a> { fn fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }
impl< 'a> fmt::Debug for JSon< 'a> { fn   fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Json") } }

//---------------------------------------------------------------------------------------------------------------------------------
