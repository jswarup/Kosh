//-- actionshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::{
    flux::{ IFluxOutSource, fluxout::FieldOut },
    shard::{ IGrammar, IForge, Parser },
    stalks::{ work::DynIWork, UniNode },
    silo::{ cast::IConstPtrMutRefExt },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ActionOp< W>
{
    pub _Action: W,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type ActionShard< C, W> = UniNode< C, ActionOp< W>>;

//---------------------------------------------------------------------------------------------------------------------------------

pub fn	Coerce< F>( f: F) -> F
where
    F: crate::stalks::work::IWork + 'static
{
    f
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> IFluxOutSource for UniNode< C, ActionOp< W>>
where
    C: IFluxOutSource,
    W: Send + Sync,
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
                *key = "Action".to_string();
                *item = FieldOut::Str( "Action");
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------



impl< C, W> IGrammar for UniNode< C, ActionOp< W>>
where
    C: IGrammar,
    W: crate::stalks::work::IWork + 'static,
{
    fn	Match( &self, parser: &mut Parser, sink: crate::flux::fluxin::FieldIn< '_>)
    {
        let  	m = parser.Forge().Mark();
        let  	res = self._Child.Parse( parser, m, crate::flux::fluxin::FieldIn::Null);
        
        if res.is_some() {
            let  	actionPtr = &self._Op._Action as &DynIWork< 'static> as *const DynIWork< 'static>;
            let  	actionMut = actionPtr.MutRef();
            actionMut.DoWork( parser);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> fmt::Display for UniNode< C, ActionOp< W>>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "Action")
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> fmt::Debug for UniNode< C, ActionOp< W>>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        fmt::Display::fmt( self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
