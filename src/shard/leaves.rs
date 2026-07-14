//-- leaves.rs -------------------------------------------------------------------------------------------------------------------------


use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ IGrammar, IForge };

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
    fn	Match< F: IForge>( &self, parser: &mut crate::shard::Parser, forge: &mut F)
    {
        self._Val.Match( parser, forge);
    }
}

