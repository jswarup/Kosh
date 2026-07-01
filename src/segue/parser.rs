//-- parser.rs ------------------------------------------------------------------------------------------------------------------------
use	std::io;
use	crate::{
    flux::InStream,
    segue::Charset
};
use	crate::silo::{ U8, U32 };
use	std::io::Read;

use	crate::silo::Stash;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IForge<'a, 'p, 's, R: Read + 'p>
where 's: 'p, Self: 'a
{
    fn	Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>;
    fn	Offset( &self) -> U32;
    fn	GetParser( &mut self) -> &mut Parser<'p, 's, R>;
    
    fn	Init( &mut self) where Self: Sized {
        let  	ptr: *mut (dyn IForge<'a, 'p, 's, R> + 'a) = self;
        let  	erased_ptr = unsafe { std::mem::transmute::<_, *mut (dyn IForge<'static, 'static, 'static, R> + 'static)>(ptr) };
        self.GetParser()._Stash.Push( Some(erased_ptr));
    }
    
    fn	Finish( &mut self) where Self: Sized {
        let  	mut dummy = None;
        self.GetParser()._Stash.Pop( &mut dummy);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------
pub struct  Forge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p {
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Offset: U32,
    pub _Parser: &'a mut Parser<'p, 's, R>,
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for Forge<'a, 'p, 's, R>
where 's: 'p {
    fn	Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)> { self._Parent }
    fn	Offset( &self) -> U32 { self._Offset }
    fn	GetParser( &mut self) -> &mut Parser<'p, 's, R> { self._Parser }
}

impl<'a, 'p, 's, R: Read + 'p> Drop for Forge<'a, 'p, 's, R>
where 's: 'p {
    fn	drop( &mut self) {
        self.Finish();
    }
}
//---------------------------------------------------------------------------------------------------------------------------------

pub trait IParser<'p, 's, R: Read + 'p>
where 's: 'p
{
    fn	Parse< G: IGrammar>( &mut self, grammar: &G) -> bool;
    fn	Stream( &mut self) -> &mut InStream<'s, R>;
    fn	Forge<'f>( &self) -> Option<&'f (dyn IForge<'f, 'p, 's, R> + 'f)> where Self: 'f;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Stream: &'p mut InStream<'s, R>,
    // Erase the 'f lifetime by transmuting to 'static to break the cyclic dropck dependency
    pub _Stash: Stash< Option<*mut (dyn IForge<'static, 'static, 'static, R> + 'static)>>,
}

impl<'p, 's, R: Read + 'p> Parser<'p, 's, R>
where 's: 'p
{
    pub fn	New( stream: &'p mut InStream<'s, R>) -> Self
    {
        Self {
            _Stream: stream,
            _Stash: Stash::New( U32( 16), U32( 0), None),
        }
    }
}

impl<'p, 's, R: Read + 'p> IParser<'p, 's, R> for Parser<'p, 's, R>
where 's: 'p
{
    fn	Parse< G: IGrammar>( &mut self, grammar: &G) -> bool
    {
        grammar.Match( self)
    }

    fn	Stream( &mut self) -> &mut InStream<'s, R>
    {
        self._Stream
    }

    fn	Forge<'f>( &self) -> Option<&'f (dyn IForge<'f, 'p, 's, R> + 'f)> where Self: 'f
    {
        let sz = self._Stash.Size();
        if sz > crate::silo::U32(0) {
            None // Placeholder
        } else {
            None
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Charset
{
    fn	Match< 'p, 'f, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let  	curr = parser.Stream().Curr();
        if self.Get( curr) {
            parser.Stream().Next();
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    fn	Match< 'p, 'f, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let  	curr = parser.Stream().Curr();
        if curr == U8( *self as u8) {
            parser.Stream().Next();
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for &str
{
    fn	Match< 'p, 'f, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let  	startMark = parser.Stream().Marker();
        
        // Ensure that empty string matches without consuming
        if self.is_empty() {
            return true;
        }

        for c in self.chars() {
            let  	curr = parser.Stream().Curr();
            if curr == U8( c as u8) {
                // If it's the last char, we just advance and we're good.
                let _ = parser.Stream().Next();
            } else {
                parser.Stream().RollTo( startMark);
                return false;
            }
        }
        
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
