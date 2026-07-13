//-- actionshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::{
    flux::{ IXFluxSource, xflux::XField },
    shard::{ IGrammar, IForge, parser::ParseForge },
    stalks::{ work::DynIWork, UniNode },
    silo::{ U32, cast::IConstPtrMutRefExt },
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

impl< C, W> IXFluxSource for UniNode< C, ActionOp< W>>
where
    C: IXFluxSource,
    W: Send + Sync,
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	node = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Child".to_string();
                node._Child.ToXField( item);
                step += 1;
                true
            } else if step == 1 {
                *key = "Action".to_string();
                *item = XField::Str( "Action");
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
    fn	Match<'p, F: IForge<'p>>(&self, forge: &mut F) -> bool
    {
        let res = self._Child.Match( forge);
        if res {
            let  	actionPtr = &self._Op._Action as &DynIWork< 'static> as *const DynIWork< 'static>;
            let  	actionMut = actionPtr.MutRef();
            actionMut.DoWork( forge.Parser());
            
        }
        res
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
