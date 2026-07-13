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
    fn	Ok( &self) -> bool; 
    fn  Success( self, mark: U32) -> Self;
    fn  Failure( self) -> Self;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar: INode
{
    fn	Match<'p, F: IForge<'p>>( &self, forge: F) -> F;
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
    pub     _Rslt: bool, 
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'p> ParseForge<'a, 'p>
{
    pub fn	New( parser: &'a mut Parser<'p>, mark: U32, succ: bool) -> Self
    {
        ParseForge {
            _Parser: parser,
            _Marker: mark,
            _Rslt: succ,
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

    fn	Ok( &self) -> bool
    {
        self._Rslt
    }

    fn  Success( mut self, mark: U32) -> Self
    {
        self._Marker = mark;
        self._Rslt = true;
        self
    }

    fn  Failure( mut self) -> Self
    {
        self._Rslt = false;
        self
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
        let  	forge = ParseForge::New( self, U32( 0), false);
        grammar.Match( forge).Ok()
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
    fn	Match<'p, F: IForge<'p>>(&self, mut forge: F) -> F
    {
        let mark = forge.Mark();
        let  	curr = forge.Parser().Curr( mark);
        if self.Get( curr.0) {
            if let Some(next) = forge.Parser().Next( mark) {
                return forge.Success( next);
            }
        }
        forge.Failure()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    fn	Match<'p, F: IForge<'p>>(&self, mut forge: F) -> F
    {
        let mark = forge.Mark();
        let  	curr = forge.Parser().Curr( mark);
        if curr == U8( *self as u8) {
            if let Some(next) = forge.Parser().Next( mark) {
                return forge.Success( next);
            }
        }
        forge.Failure()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for char
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for str
{
    fn	Match<'p, F: IForge<'p>>(&self, mut forge: F) -> F
    {
        // Ensure that empty string matches without consuming
        if self.is_empty() {
            let m = forge.Mark();
            return forge.Success( m);
        }

        let  	orig_mark = forge.Mark();
        let  	mut m = orig_mark;
        for c in self.chars() {
            let  	curr = forge.Parser().Curr( m);
            if curr == U8( c as u8) {
                // If it's the last char, we just advance and we're good.
                if let Some( next_mark) = forge.Parser().Next( m) {
                    m = next_mark;
                } else {
                    return forge.Failure();
                }
            } else {
                return forge.Failure();
            }
        }

        forge.Success( m)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'r, T: IGrammar> IGrammar for &'r T
{
    fn	Match<'p, F: IForge<'p>>(&self, forge: F) -> F
    {
        (**self).Match( forge)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
