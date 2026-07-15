//-- leaves.rs -------------------------------------------------------------------------------------------------------------------------


use	crate::shard::Parser;
use	crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use	crate::flux::fluximport::FieldImp;
use	crate::shard::IGrammar;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct StrShard< 'a>
{
    pub _Val: &'a str,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IFluxExportSource for StrShard< 'a>
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        *field = FieldExp::Str( self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> crate::flux::IFluxImportSource for StrShard<'a> {
    fn FetchFieldImp<'b>(&'b mut self, field: &mut crate::flux::fluximport::FieldImp<'b>) {
        crate::flux::IFluxImportSource::FetchFieldImp(&mut self._Val, field);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for StrShard< 'a>
{

    fn	Match( &self, parser: &mut Parser, _sink: FieldImp< '_>)
    {
        self._Val.Match( parser, crate::flux::fluximport::FieldImp::Null);
    }
}
