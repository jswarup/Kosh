//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::instream::IStream;
use	crate::flux::IXFluxSource;
use crate::shard::Charset;
use	crate::silo::{ U32, U8 };
use	crate::stalks::{ IWorker, WorkPtr, INode };

//---------------------------------------------------------------------------------------------------------------------------------
 
pub trait IForge< 'p>: Send + Sync
{
    fn	Parser( &mut self) -> &mut Parser< 'p>; 
    fn	Mark( &self) -> U32; 
    fn	SetMark( &mut self, mark: U32);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar: INode
{
    fn	Match<'p, F: IForge<'p>>( &self, forge: &mut F) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p>
{
    pub     _InStream: &'p mut dyn IStream,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ParseForge<'a, 'p> 
{
    pub     _Parser: &'a mut Parser<'p>,
    pub     _Marker: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'p> ParseForge<'a, 'p>
{
    pub fn	New( parser: &'a mut Parser<'p>, mark: U32) -> Self
    {
        ParseForge {
            _Parser: parser,
            _Marker: mark,
        }
    }
} 

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'p> IForge<'p> for ParseForge<'a, 'p>
{
    fn	Parser( &mut self) -> &mut Parser< 'p>
    {
        self._Parser
    }
     
    fn	Mark( &self) -> U32
    {
        self._Marker
    }

    fn	SetMark( &mut self, mark: U32)
    {
        self._Marker = mark;
    }
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
        }
    }
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn Parse< G: IGrammar + ?Sized>( &mut self, grammar: &'p G) -> bool
    {
        let  	mut forge = ParseForge::New( self, U32( 0));
        grammar.Match( &mut forge)
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
    fn	Match<'p, F: IForge<'p>>(&self, forge: &mut F) -> bool
    {
        let mark = forge.Mark();
        let  	curr = forge.Parser().Curr( mark);
        if self.Get( curr.0) {
            forge.SetMark( mark + U32( 1));
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    fn	Match<'p, F: IForge<'p>>(&self, forge: &mut F) -> bool
    {
        let mark = forge.Mark();
        let  	curr = forge.Parser().Curr( mark);
        if curr == U8( *self as u8) {
            forge.SetMark( mark + U32( 1));
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for char
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for str
{
    fn	Match<'p, F: IForge<'p>>(&self, forge: &mut F) -> bool
    {
        // Ensure that empty string matches without consuming
        if self.is_empty() {
            return true;
        }

        let  	mark = forge.Mark();
        let  	mut m = mark;
        let  	bytes = self.as_bytes();
        for &b in bytes {
            let  	curr = forge.Parser().Curr( m);
            if curr.0 != b {
                return false;
            }
            m += U32( 1);
        }

        forge.SetMark( m);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'r, T: IGrammar> IGrammar for &'r T
{
    fn	Match<'p, F: IForge<'p>>(&self, forge: &mut F) -> bool
    {
        (**self).Match( forge)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
