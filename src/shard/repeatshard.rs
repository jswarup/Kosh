//-- repeatshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::flux::fluximport::FieldImp;
use	crate::shard::Parser;
use	crate::{
    flux::{ IFluxExportSource, fluxexport::FieldExp },
    shard::{ IGrammar, IForge },
    silo::{ USeg, U32 },
    stalks::UniNode,
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type RepeatShard< C> = UniNode< C, USeg>;

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> IFluxExportSource for UniNode< C, USeg>
where
    C: IFluxExportSource,
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        let  	mut step = 0u32;
        let  	node = self;
        *field = FieldExp::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Child".to_string();
                node._Child.FetchFieldExp( item);
                step += 1;
                true
            } else if step == 1 {
                *key = "Repeat".to_string();
                *item = FieldExp::FluxSource( &node._Op);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> IGrammar for UniNode< C, USeg>
where
    C: IGrammar,
{

fn	Match( &self, parser: &mut Parser, mut sink: FieldImp< '_>)
    {
        let  	mut count = U32( 0);
        let  	first = self._Op.First();
        let  	last = if self._Op.IsEmpty() { U32::_X } else { self._Op.Last() };
        
        let  	mut m = parser.Forge().Mark();

        while count < last {
            sink.Resolve();
            let  	mut temp_sink = crate::flux::fluximport::FieldImp::Null;
            std::mem::swap( &mut temp_sink, &mut sink);
            
            let  	mut child_sink = crate::flux::fluximport::FieldImp::Null;
            if let crate::flux::fluximport::FieldImp::Arr( ref mut closure) = temp_sink {
                closure( &mut child_sink);
            }
            std::mem::swap( &mut temp_sink, &mut sink);

            let  	res = self._Child.Parse( parser, m, child_sink);
            if let Some( newM) = res {
                if newM == m {
                    count += U32( 1);
                    break;
                }
                m = newM;
                count += U32( 1);
            } else {
                break;
            }
        }
        
        if count >= first {
            let  	res = Some( m);
            parser.Forge().Deposit( res);
        } else {
            parser.Forge().Deposit( None);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> fmt::Display for UniNode< C, USeg>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "Repeat( {:?})", self._Op)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> fmt::Debug for UniNode< C, USeg>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        fmt::Display::fmt( self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
