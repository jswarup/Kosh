//-- leaves.rs -------------------------------------------------------------------------------------------------------------------------


use	crate::shard::Parser;
use	crate::flux::{ IFluxOutSource, fluxout::FieldOut };
use	crate::flux::fluxin::FieldIn;
use	crate::shard::IGrammar;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct StrShard< 'a>
{
    pub _Val: &'a str,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IFluxOutSource for StrShard< 'a>
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>)
    {
        *field = FieldOut::Str( self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for StrShard< 'a>
{

    fn	Match( &self, parser: &mut Parser, _sink: FieldIn< '_>)
    {
        self._Val.Match( parser, crate::flux::fluxin::FieldIn::Null);
    }
}
