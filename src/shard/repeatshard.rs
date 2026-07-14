//-- repeatshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::flux::fluxin::FieldIn;
use	crate::shard::Parser;
use	crate::{
    flux::{ IFluxOutSource, fluxout::FieldOut },
    shard::{ IGrammar, IForge },
    silo::{ USeg, U32 },
    stalks::UniNode,
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type RepeatShard< C> = UniNode< C, USeg>;

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> IFluxOutSource for UniNode< C, USeg>
where
    C: IFluxOutSource,
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>)
    {
        let  	mut step = 0u32;
        let  	node = self;
        *field = FieldOut::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Child".to_string();
                node._Child.ToFieldOut( item);
                step += 1;
                true
            } else if step == 1 {
                *key = "Repeat".to_string();
                *item = FieldOut::FluxSource( &node._Op);
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

fn	Match( &self, parser: &mut Parser, mut sink: FieldIn< '_>)
    {
        let  	mut count = U32( 0);
        let  	first = self._Op.First();
        let  	last = if self._Op.IsEmpty() { U32::_X } else { self._Op.Last() };
        
        let  	mut m = parser.Forge().Mark();

        while count < last {
            sink.Resolve();
            let  	mut temp_sink = crate::flux::fluxin::FieldIn::Null;
            std::mem::swap( &mut temp_sink, &mut sink);
            
            let  	mut child_sink = crate::flux::fluxin::FieldIn::Null;
            if let crate::flux::fluxin::FieldIn::Arr( ref mut closure) = temp_sink {
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
