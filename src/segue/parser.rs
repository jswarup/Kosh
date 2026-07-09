//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::instream::IStream;
use	crate::segue::Charset;
use	crate::silo::{ U32, U8 };
use	crate::stalks::{ BinOp, DynINode, DynIWorker, INode, IWorker, WorkPtr };

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar
{
    fn	Match< 'p>( &'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32>;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p>
{
    pub     _InStream: &'p mut dyn IStream,
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
        grammar.Match( self, U32(0)).is_some()
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
    fn	Match< 'p>( &'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32>
    {
        let  	curr = parser.Curr( marker);
        if self.Get( curr) {
            parser.Next( marker)
        } else {
            None
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    fn	Match< 'p>( &'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32>
    {
        let  	curr = parser.Curr( marker);
        if curr == U8( *self as u8) {
            parser.Next( marker)
        } else {
            None
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for str
{
    fn	Match< 'p>( &'p self, parser: &mut Parser<'p>, mut marker: U32) -> Option<U32>
    {
        // Ensure that empty string matches without consuming
        if self.is_empty() {
            return Some( marker);
        }

        for c in self.chars() {
            let  	curr = parser.Curr( marker);
            if curr == U8( c as u8) {
                // If it's the last char, we just advance and we're good.
                if let Some( next_mark) = parser.Next( marker) {
                    marker = next_mark;
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }

        Some( marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> IGrammar for DynINode<'a>
{
    fn Match< 'p>( &'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32>
    {
        let parser_ptr = parser as *mut _ as *mut ();
        self.MatchGrammar(parser_ptr, marker.0).map(U32)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'r> IGrammar for &'r DynINode<'a>
{
    fn	Match< 'p>( &'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32>
    {
        (*self).Match( parser, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> INode<'a> for String {
    fn _Size(&self) -> U32 { U32(0) }
    fn _At(&self, _idx: U32) -> &DynINode<'a> { panic!("Leaf") }
    fn Value(&self) -> Option<WorkPtr<'a>> { None }
    fn DocStr(&self) -> &'static str { "" }
    fn BinOp(&self) -> BinOp { BinOp::None }
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut Parser<'_>) };
        self.as_str().Match(p, U32(marker)).map(|u| u.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> INode<'a> for Charset {
    fn _Size(&self) -> U32 { U32(0) }
    fn _At(&self, _idx: U32) -> &DynINode<'a> { panic!("Leaf") }
    fn Value(&self) -> Option<WorkPtr<'a>> { None }
    fn DocStr(&self) -> &'static str { "" }
    fn BinOp(&self) -> BinOp { BinOp::None }
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut Parser<'_>) };
        self.Match(p, U32(marker)).map(|u| u.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
