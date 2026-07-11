//-- leaves.rs -------------------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ Charset, IGrammar, Parser };
use	crate::silo::U32;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct StrShard< 'a>
{
    pub _Val: &'a str,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IXFluxSource for StrShard< 'a>
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        *field = XField::Str( self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for StrShard< 'a>
{
    fn	Match<'p>(&self, parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        self._Val.Match( parser, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Display for StrShard< 'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "StrShard( {:?})", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Debug for StrShard< 'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return fmt::Display::fmt( self, f);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct CharsetShard
{
    pub _Val: Charset,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for CharsetShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	s = self._Val.to_string();
        *field = XField::String( s);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for CharsetShard
{
    fn	Match<'p>(&self, parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        self._Val.Match( parser, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for CharsetShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "{}", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for CharsetShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "{:?}", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
