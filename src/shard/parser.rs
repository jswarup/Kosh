//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::fluximport::FieldImp;
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
    pub     prev: *const Forge,
    pub     _CurrMark: U32,
    pub     _IsMatched: bool,
}

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

unsafe impl Send for Forge {}
unsafe impl Sync for Forge {}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar: INode
{
    fn	Match( &self, parser: &mut Parser);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p>
{
    pub     _InStream: &'p mut dyn IStream,
    pub     _TopForge: *const Forge,
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
            _TopForge: std::ptr::null(),
        }
    }
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ParseGrammar( &mut self, grammar: &( impl IGrammar + ?Sized), mark: U32) -> Option< U32>
    {
        let  	node = Forge {
            prev: self._TopForge,
            _CurrMark: mark,
            _IsMatched: false,
        };
        let  	prevTop = self._TopForge;
        self._TopForge = &node as *const Forge;
        grammar.Match( self);
        self._TopForge = prevTop;
        let  	res = node.Result();
        if !prevTop.is_null() {
            unsafe {
                ( *( prevTop as *mut Forge)).Deposit( res);
            }
        }
        res
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Compactly deposit a result to the current forge.
    pub fn	Deposit( &mut self, result: Option< U32>)
    {
        self.Forge().Deposit( result);
    }

    /// Compactly get the current mark from the top forge.
    pub fn	CurrMark( &self) -> U32
    {
        self.Forge().Mark()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Forge<'a>( &'a self) -> &'a mut Forge
    {
        unsafe {
            &mut *( self._TopForge as *mut Forge)
        }
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
