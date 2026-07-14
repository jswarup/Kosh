//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::instream::IStream;
use	crate::flux::IXFluxSource;
use crate::shard::Charset;
use	crate::silo::{ U32, U8 };
use	crate::stalks::{ IWorker, WorkPtr, INode };

//---------------------------------------------------------------------------------------------------------------------------------
 
pub trait IForge: Send + Sync + 'static
{
    fn	Mark( &self) -> U32; 
    fn	Deposit( &mut self, result: Option< U32>);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BaseForge
{
    pub     prev: *const BaseForge,
    pub     _CurrMark: U32,
    pub     _IsMatched: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BaseForge
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

impl IForge for BaseForge
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

unsafe impl Send for BaseForge {}
unsafe impl Sync for BaseForge {}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar: INode
{
    fn	Match( &self, parser: &mut Parser);

    fn	Parse( &self, parser: &mut Parser, mark: U32) -> Option< U32>
    {
        let  	node = BaseForge {
            prev: parser._TopForge,
            _CurrMark: mark,
            _IsMatched: false,
        };
        let  	prevTop = parser._TopForge;
        parser._TopForge = &node as *const BaseForge;
        self.Match( parser);
        parser._TopForge = prevTop;
        let  	res = node.Result();
        if !prevTop.is_null() {
            unsafe {
                ( *( prevTop as *mut BaseForge)).Deposit( res);
            }
        }
        res
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p>
{
    pub     _InStream: &'p mut dyn IStream,
    pub     _TopForge: *const BaseForge,
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


    pub fn	Parse< G: IGrammar + ?Sized>( &mut self, grammar: &'p G) -> bool
    {
        let  	node = BaseForge {
            prev: std::ptr::null(),
            _CurrMark: U32( 0),
            _IsMatched: false,
        };
        self._TopForge = &node as *const BaseForge;
        grammar.Match( self);
        self._TopForge = std::ptr::null();
        let  	matched = node.Result().is_some();
        matched
    }

    pub fn	Forge<'a>( &'a self) -> &'a mut BaseForge
    {
        unsafe {
            &mut *( self._TopForge as *mut BaseForge)
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

impl IGrammar for Charset
{
    fn	Match( &self, parser: &mut Parser)
    {
        let  	mark = parser.Forge().Mark();
        let  	curr = parser.GetAt( mark);
        if self.Get( curr.0) {
            let  	res = Some( mark + U32( 1));
            parser.Forge().Deposit( res);
        } else {
            parser.Forge().Deposit( None);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    fn	Match( &self, parser: &mut Parser)
    {
        let  	mark = parser.Forge().Mark();
        let  	curr = parser.GetAt( mark);
        if curr == U8( *self as u8) {
            let  	res = Some( mark + U32( 1));
            parser.Forge().Deposit( res);
        } else {
            parser.Forge().Deposit( None);
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
    fn	Match( &self, parser: &mut Parser)
    {
        let  	mark = parser.Forge().Mark();
        let  	key = self.as_bytes();
        let  	mut currentMark = mark;
        
        for &b in key {
            let  	stream = parser.InStream();
            let  	curr = stream.At( currentMark);
            if curr.0 != b {
                parser.Forge().Deposit( None);
                return;
            }
            if let  	Some( next) = parser.Incr( currentMark) {
                currentMark = next;
            } else {
                parser.Forge().Deposit( None);
                return;
            }
        }
        
        parser.Forge().Deposit( Some( currentMark));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'r, T: IGrammar> IGrammar for &'r T
{
    fn	Match( &self, parser: &mut Parser)
    {
        (**self).Match( parser);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

