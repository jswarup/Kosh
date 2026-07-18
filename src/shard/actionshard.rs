//-- actionshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::{
    flux::{ IFluxImportSource, IFluxExportSource, fluximport::FieldImp, fluxexport::FieldExp },
    shard::{ IGrammar, Parser },
    stalks::{ UniNode },
    silo::{ cast::{ IConstPtrMutRefExt, ICastExt }, Arr, U8 },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ActionOp< W>
{
    pub _Action: W,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type ActionShard< C, W> = UniNode< C, ActionOp< W>>;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INotify: Send + Sync
{
    fn	DoNotify( &mut self, matched: Arr< '_, U8>);
}

impl< F> INotify for F
where
    F: for< 'a> FnMut( Arr< 'a, U8>) + Send + Sync,
{
    fn	DoNotify( &mut self, matched: Arr< '_, U8>)
    {
        self( matched);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn	Coerce< F>( f: F) -> F
where
    F: INotify + 'static
{
    f
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> IFluxExportSource for UniNode< C, ActionOp< W>>
where
    C: IFluxExportSource,
    W: Send + Sync,
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
                *key = "Action".to_string();
                *item = FieldExp::Str( "Action");
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C, W> IFluxImportSource for UniNode< C, ActionOp< W>>
where
    C: IFluxImportSource,
    W: Send + Sync,
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
            if key == "Action" {
                *item = FieldImp::ExpectedType("Action");
                return true;
            }
            false
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------



impl< C, W> IGrammar for UniNode< C, ActionOp< W>>
where
    C: IGrammar,
    W: INotify + 'static,
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	m = parser.CurrMark();
        let  	res = parser.ParseGrammar( &self._Child, m);

        if let Some( completedMark) = res {
            let  	actionPtr = &self._Op._Action as *const W;
            let  	actionMut = actionPtr.MutRef();
            let  	arr = parser.InStream().BytesAt( m, completedMark - m);
            actionMut.DoNotify( arr);
            true
        } else {
            false
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
