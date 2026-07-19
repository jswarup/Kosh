//-- actionshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::{

    shard::{ IGrammar, Parser },
    stalks::{ UniNode },
    silo::{ cast::IConstPtrMutRefExt, Arr, U8 },
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
