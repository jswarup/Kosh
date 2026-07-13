//-- leaves.rs -------------------------------------------------------------------------------------------------------------------------


use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ Charset, IGrammar, IForge };

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
    fn	Match<'p, F: IForge<'p>>(&self, forge: F) -> F
    {
        self._Val.Match( forge)
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
    fn	Match<'p, F: IForge<'p>>(&self, forge: F) -> F
    {
        self._Val.Match( forge)
    }
}
