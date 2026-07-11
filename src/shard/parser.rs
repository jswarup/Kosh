//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::instream::IStream;
use crate::shard::Charset;
use	crate::silo::{ U32, U8 };
use	crate::stalks::{ DynIWorker, IWorker, WorkPtr };

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar
{
    fn	Match<'p>(&self, parser: &mut Parser<'p>, marker: U32) -> (bool, U32);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p>
{
    pub     _InStream: &'p mut dyn IStream,
}
//---------------------------------------------------------------------------------------------------------------------------------

pub struct ParseForge<'p>
{
    pub     _Parser: &'p mut Parser<'p>,
    pub     _Marker: U32,
    pub     _MatchRslt: bool, 
}

//---------------------------------------------------------------------------------------------------------------------------------

// SAFETY: Parser is used single-threaded within a parse session.
// The raw pointers in _Stash are not shared across threads.
unsafe impl<'p> Send for Parser<'p> {}
unsafe impl<'p> Sync for Parser<'p> {}

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

/// Extension trait to easily downcast a generic IWorker into a Parser
pub trait IWorkerExt {
    fn AsParser<'p>(&self) -> Option<&mut Parser<'p>>;
}

impl IWorkerExt for DynIWorker<'_> {
    fn AsParser<'p>(&self) -> Option<&mut Parser<'p>> {
        let raw_ptr = self.AsRawWorker() as *mut Parser<'p>;
        if raw_ptr.is_null() {
            None
        } else {
            Some(unsafe { &mut *raw_ptr })
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


impl<'p> Parser<'p>
{
    pub fn	New( stream: &'p mut dyn IStream) -> Self
    {
        Self {
            _InStream: stream,
        }
    }
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn Parse< G: IGrammar + ?Sized>( &mut self, grammar: &'p G) -> bool
    {
        let marker = U32(0);
        grammar.Match( self, marker).0
    }

    pub fn InStream( &mut self) -> &mut dyn IStream
    {
        self._InStream
    }

    pub fn	Curr( &mut self, marker: U32) -> U8
    {
        self._InStream.At( marker)
    }

    pub fn	Next( &mut self, mut marker: U32) -> Option<U32>
    {
        marker += U32( 1);
        if marker.AsUsize() <= self._InStream.Size() {
            Some(marker)
        } else {
            None
        }
    }
}
 
//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Charset
{
    fn	Match<'p>(&self, parser: &mut Parser<'p>, marker: U32) -> (bool, U32)
    {
        let  	curr = parser.Curr( marker);
        if self.Get( curr.0) {
            if let Some(next) = parser.Next( marker) {
                return (true, next);
            }
        }
        (false, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    fn	Match<'p>(&self, parser: &mut Parser<'p>, marker: U32) -> (bool, U32)
    {
        let  	curr = parser.Curr( marker);
        if curr == U8( *self as u8) {
            if let Some(next) = parser.Next( marker) {
                return (true, next);
            }
        }
        (false, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for str
{
    fn	Match<'p>(&self, parser: &mut Parser<'p>, marker: U32) -> (bool, U32)
    {
        // Ensure that empty string matches without consuming
        if self.is_empty() {
            return (true, marker);
        }

        let mut m = marker;
        for c in self.chars() {
            let  	curr = parser.Curr( m);
            if curr == U8( c as u8) {
                // If it's the last char, we just advance and we're good.
                if let Some( next_mark) = parser.Next( m) {
                    m = next_mark;
                } else {
                    return (false, marker);
                }
            } else {
                return (false, marker);
            }
        }

        (true, m)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

// removed impl<'a> IGrammar for DynINode<'a>

//---------------------------------------------------------------------------------------------------------------------------------


impl<'a, 'r, T: IGrammar> IGrammar for &'r T
{
    fn	Match<'p>(&self, parser: &mut Parser<'p>, marker: U32) -> (bool, U32)
    {
        (**self).Match( parser, marker)
    }
}


//---------------------------------------------------------------------------------------------------------------------------------
