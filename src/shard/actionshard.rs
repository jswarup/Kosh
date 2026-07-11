//-- actionshard.rs -----------------------------------------------------------------------------------------------------------------
use	crate::silo::U32;
use	crate::stalks::work::DynIWork;
use	crate::flux::{ IXFluxSource, xflux::XField };
use	std::fmt;
use	crate::shard::{ IGrammar, Parser };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ActionShard< C, W>
{
    pub _Child: C,
    pub _Action: W,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn	Coerce< F>( f: F) -> F
where
    F: crate::stalks::work::IWork + 'static
{
    f
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> IXFluxSource for ActionShard< C, W>
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
            } else { false }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> IGrammar for ActionShard< C, W>
where
    C: IGrammar,
    W: crate::stalks::work::IWork + 'static,
{
    fn	Match<'p>(&self, parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
{
        let (matched, new_mark) = self._Child.Match( parser, marker);
        if matched {
            let  	actionPtr = &self._Action as &DynIWork< 'static> as *const DynIWork< 'static>;
            #[allow( invalid_reference_casting)]
            let  	actionMut = unsafe { &mut *( actionPtr as *mut DynIWork< 'static>) };
            actionMut.DoWork( parser);
            return (true, new_mark);
        }
        (false, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> fmt::Display for ActionShard< C, W>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
{
        write!( f, "Action")
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> fmt::Debug for ActionShard< C, W>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
{
        fmt::Display::fmt( self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
