//-- repeatshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::shard::Parser;
use	crate::{
    flux::{ IFluxImportSource, IFluxExportSource, fluximport::FieldImp, fluxexport::FieldExp },
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

impl< C> IFluxImportSource for UniNode< C, USeg>
where
    C: IFluxImportSource,
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        let ptr = self as *mut Self;
        *field = FieldImp::Obj( Box::new( move |key, item| {
            let obj = unsafe { &mut *ptr };
            if key == "Child" {
                IFluxImportSource::FetchFieldImp(&mut obj._Child, item);
                return true;
            }
            if key == "Repeat" {
                IFluxImportSource::FetchFieldImp(&mut obj._Op, item);
                return true;
            }
            false
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
            let  	mut temp_sink = FieldImp::Null;
            std::mem::swap( &mut temp_sink, &mut sink);
            
            let  	mut child_sink = FieldImp::Null;
            if let FieldImp::Arr( ref mut closure) = temp_sink {
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
