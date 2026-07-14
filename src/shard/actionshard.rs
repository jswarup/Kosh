//-- actionshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::{
    flux::{ IXFluxSource, xflux::XField },
    shard::{ IGrammar, IForge, Parser },
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

pub struct ActionForge< 'a, 'p, W>
where
    W: crate::stalks::work::IWork + 'static,
{
    pub     _Parser: &'a mut Parser< 'p>,
    pub     _Action: &'a W,
    pub     _OrigMark: U32,
    pub     _CurrMark: U32,
    pub     _Result: Option< U32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'p, W: crate::stalks::work::IWork + 'static> ActionForge< 'a, 'p, W>
{
    pub fn	New( parser: &'a mut Parser< 'p>, action: &'a W) -> Self
    {
        ActionForge {
            _Parser: parser,
            _Action: action,
            _OrigMark: U32( 0),
            _CurrMark: U32( 0),
            _Result: None,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'p, W: crate::stalks::work::IWork + 'static> IForge< 'p> for ActionForge< 'a, 'p, W>
{
    fn	Parser( &mut self) -> &mut Parser< 'p>
    {
        self._Parser
    }
     
    fn	Mark( &self) -> U32
    {
        self._CurrMark
    }

    fn	SetMark( &mut self, mark: U32)
    {
        self._CurrMark = mark;
    }

    fn	Deposit( &mut self, result: Option< U32>)
    {
        self._Result = result;
        if let Some( mark) = result {
            self._CurrMark = mark;
        }
    }

    fn	Result( &self) -> Option< U32>
    {
        self._Result
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'p, W: crate::stalks::work::IWork + 'static> Drop for ActionForge< 'a, 'p, W>
{
    fn	drop( &mut self)
    {
        if let Some( _mark) = self._Result {
            let  	actionPtr = self._Action as &DynIWork< 'static> as *const DynIWork< 'static>;
            let  	actionMut = actionPtr.MutRef();
            actionMut.DoWork( self._Parser);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl< 'a, 'p, W> Send for ActionForge< 'a, 'p, W> where W: crate::stalks::work::IWork + 'static {}
unsafe impl< 'a, 'p, W> Sync for ActionForge< 'a, 'p, W> where W: crate::stalks::work::IWork + 'static {}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> IGrammar for UniNode< C, ActionOp< W>>
where
    C: IGrammar,
    W: crate::stalks::work::IWork + 'static,
{
    fn	Forge< 'a, 'p>( &'a self, parser: &'a mut Parser< 'p>) -> impl IForge< 'p> + 'a
    where
        'p: 'a
    {
        ActionForge::New( parser, &self._Op._Action)
    }

    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F)
    {
        let  	res = {
            let  	mark = forge.Mark();
            let  	mut actionForge = self.Forge( forge.Parser());
            actionForge.SetMark( mark);
            self._Child.Match( &mut actionForge);
            actionForge.Result()
        };
        forge.Deposit( res);
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
