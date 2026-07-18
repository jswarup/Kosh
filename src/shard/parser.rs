//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	std::ptr::NonNull;
use	crate::flux::instream::IStream;
use	crate::silo::{ U32, U8 };
use	crate::stalks::{ IWorker, WorkPtr, INode };

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IForge: Send + Sync + 'static
{
    fn	Mark( &self) -> U32;
    fn	Deposit( &mut self, result: Option< U32>);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Forge
{
    pub     _Prev: Option< NonNull< Forge>>,
    pub     _CurrMark: U32,
    pub     _IsMatched: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl Send for Forge {}
unsafe impl Sync for Forge {}

//---------------------------------------------------------------------------------------------------------------------------------

impl Forge
{
    pub fn	Result( &self) -> Option< U32>
    {
        if self._IsMatched {
            Some( self._CurrMark)
        } else {
            None
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IForge for Forge
{
    fn	Mark( &self) -> U32
    {
        self._CurrMark
    }

    fn	Deposit( &mut self, result: Option< U32>)
    {
        if let Some( mark) = result {
            self._CurrMark = mark;
            self._IsMatched = true;
        } else {
            self._IsMatched = false;
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar: INode
{
    fn	Match( &self, parser: &mut Parser);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p>
{
    pub     _InStream: &'p mut dyn IStream,
    _TopForge: Option< NonNull< Forge>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

// ParseForge removed completely

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl<'p> Send for Parser<'p> {}
unsafe impl<'p> Sync for Parser<'p> {}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'p> IWorker for Parser<'p>
{
    fn	PostJob( &self, job: WorkPtr< '_>)
    {
        if !job.IsNull() {
            ( job.func)( job.data, self);
        }
    }

    fn	AsRawWorker( &self) -> *const ()
    {
        self as *const _ as *const ()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'p> Parser<'p>
{
    pub fn	New( stream: &'p mut dyn IStream) -> Self
    {
        Self {
            _InStream: stream,
            _TopForge: None,
        }
    }
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ParseGrammar( &mut self, grammar: &( impl IGrammar + ?Sized), mark: U32) -> Option< U32>
    {
        let  	mut node = Forge {
            _Prev: self._TopForge,
            _CurrMark: mark,
            _IsMatched: false,
        };
        let  	prevTop = self._TopForge.replace( NonNull::from( &mut node));
        grammar.Match( self);
        self._TopForge = prevTop;
        let  	res = node.Result();
        if let Some( mut prevTop) = prevTop {
            unsafe { prevTop.as_mut().Deposit( res); }
        }
        res
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Compactly deposit a result to the current forge.
    pub fn	Deposit( &mut self, result: Option< U32>)
    {
        self.ForgeMut().Deposit( result);
    }

    /// Compactly get the current mark from the top forge.
    pub fn	CurrMark( &self) -> U32
    {
        self.Forge().Mark()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Returns the active forge. It is present while a grammar is being matched.
    pub fn	Forge( &self) -> &Forge
    {
        unsafe { self._TopForge.expect( "no active forge").as_ref() }
    }

    /// Returns the active forge mutably.
    fn	ForgeMut( &mut self) -> &mut Forge
    {
        unsafe { self._TopForge.expect( "no active forge").as_mut() }
    }

    pub fn InStream( &mut self) -> &mut dyn IStream
    {
        self._InStream
    }

    pub fn	GetAt( &mut self, marker: U32) -> U8
    {
        self._InStream.At( marker)
    }

    pub fn	Incr( &mut self, mut marker: U32) -> Option<U32>
    {
        marker += U32( 1);
        if marker <= self._InStream.Size() {
            Some(marker)
        } else {
            None
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
