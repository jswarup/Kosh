//-- parser.rs ------------------------------------------------------------------------------------------------------------------------
use	crate::{
    flux::InStream,
    segue::Charset
};
use	crate::silo::{ U8, U32 };
use	std::io::Read;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IForge<'a>
{
    fn	Parent( &self) -> Option< &'a dyn IForge<'a>>;
    fn	Offset( &self) -> U32;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar
{
    fn	Match< 'a, 'f, 's, R: Read>( &self, parser: &mut Parser<'a, 'f, 's, R>) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IParser<'a, 'f, 's, R: Read>
{
    fn	Parse< G: IGrammar>( &mut self, grammar: &G) -> bool;
    fn	Stream( &mut self) -> &mut InStream<'s, R>;
    fn	Forge( &self) -> &'f dyn IForge<'f>;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'a, 'f, 's, R: Read = std::io::Empty>
{
    _Stream: &'a mut InStream<'s, R>,
    _Forge: &'f dyn IForge<'f>,
}

impl<'a, 'f, 's, R: Read> Parser<'a, 'f, 's, R>
{
    pub fn	New( stream: &'a mut InStream<'s, R>, forge: &'f dyn IForge<'f>) -> Self
    {
        Self {
            _Stream: stream,
            _Forge: forge,
        }
    }
}

impl<'a, 'f, 's, R: Read> IParser<'a, 'f, 's, R> for Parser<'a, 'f, 's, R>
{
    fn	Parse< G: IGrammar>( &mut self, grammar: &G) -> bool
    {
        grammar.Match( self)
    }

    fn	Stream( &mut self) -> &mut InStream<'s, R>
    {
        self._Stream
    }

    fn	Forge( &self) -> &'f dyn IForge<'f>
    {
        self._Forge
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Charset
{
    fn	Match< 'a, 'f, 's, R: Read>( &self, parser: &mut Parser<'a, 'f, 's, R>) -> bool
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
    fn	Match< 'a, 'f, 's, R: Read>( &self, parser: &mut Parser<'a, 'f, 's, R>) -> bool
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
    fn	Match< 'a, 'f, 's, R: Read>( &self, parser: &mut Parser<'a, 'f, 's, R>) -> bool
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
