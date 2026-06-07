//-- shard.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::stalks::bud::Bud;
use	crate::stalks::work::{ IWork, IWorker };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct Shard 
{
    _Closure: Option< fn( &dyn IWorker)>,
    _Char: Option< char>,
    _String: Option< String>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Shard 
{
    fn	default() -> Self 
    {
        Self {
            _Closure: Some( |_| {}),
            _Char: None,
            _String: None,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Shard 
{
    pub fn	New( f: fn( &dyn IWorker)) -> Self 
    {
        Self {
            _Closure: Some( f),
            _Char: None,
            _String: None,
        }
    }
    pub fn	NewChar( c: char) -> Self 
    {
        Self {
            _Closure: None,
            _Char: Some( c),
            _String: None,
        }
    }
    pub fn	NewString( s: String) -> Self 
    {
        Self {
            _Closure: None,
            _Char: None,
            _String: Some( s),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Shard 
{
    fn	DoWork( &mut self, worker: &dyn IWorker) 
    {
        if let  	Some( f) = self._Closure {
            ( f)( worker);
        } else if let  	Some( c) = self._Char {
            print!( "{} ", c);
        } else if let  	Some( s) = &self._String {
            print!( "{} ", s);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Bud< Shard> for Shard 
{
    fn	Val( &self) -> Shard 
    {
        self.clone()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::BudOp for Shard 
{
    fn	IsOpAllowed( op: crate::stalks::bud::BudBinOp) -> bool 
    {
        matches!( 
            op,
            crate::stalks::bud::BudBinOp::LT | crate::stalks::bud::BudBinOp::BOR
        )
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for Shard 
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result 
    {
        if let  	Some( c) = self._Char {
            write!( f, "Shard( {})", c)
        } else if let  	Some( s) = &self._String {
            write!( f, "Shard( {})", s)
        } else {
            write!( f, "Shard")
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Shard> for fn( &dyn IWorker) 
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>> 
    {
        Box::new( Shard::New( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Shard> for char 
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>> 
    {
        Box::new( Shard::from( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Shard> for String 
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>> 
    {
        Box::new( Shard::from( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Shard> for &'static str 
{
    fn	IntoBud( self) -> Box< dyn Bud< Shard>> 
    {
        Box::new( Shard::from( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< char> for Shard 
{
    fn	from( c: char) -> Self 
    {
        Self::NewChar( c)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< String> for Shard 
{
    fn	from( s: String) -> Self 
    {
        Self::NewString( s)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< &str> for Shard 
{
    fn	from( s: &str) -> Self 
    {
        Self::NewString( s.to_string())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
