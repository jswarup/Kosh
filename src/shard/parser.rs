//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::instream::IStream;
use	crate::silo::{ IAccess, IArr, Stash, U32, U8 };
use	crate::stalks::{ IWorker, WorkPtr, INode };

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar: INode
{
    fn	Match( &self, parser: &mut Parser) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p>
{
    pub     _InStream: &'p mut dyn IStream,
    pub     _Markers: Stash< U32>,
}

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
            _Markers: Stash::NewEmpty(),
        }
    }
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ParseGrammar( &mut self, grammar: &( impl IGrammar + ?Sized), mark: U32) -> Option< U32>
    {
        self._Markers.Push( mark);
        let  	matched = grammar.Match( self);
        let  	completedMark = self.CurrMark();
        let  	mut marker = U32( 0);
        assert!( self._Markers.Pop( &mut marker), "missing parse marker");
        if matched {
            if !self._Markers.Stk().Arr().IsEmpty() {
                self.SetCurrMark( completedMark);
            }
            Some( completedMark)
        } else {
            None
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Updates the active parse marker.
    pub fn	SetCurrMark( &mut self, mark: U32)
    {
        let   	markers = self._Markers.Stk().Arr();
        let   	last = markers.Size() - U32( 1);
        markers.SetAt( last, &mark);
    }

    /// Returns the active parse marker.
    pub fn	CurrMark( &self) -> U32
    {
        *self._Markers.Stk().Arr().Last()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

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
