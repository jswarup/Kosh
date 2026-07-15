//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::{ IFluxImportSource, IFluxExportSource, fluximport::FieldImp };
use	crate::flux::instream::IStream;
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
    fn	Match( &self, parser: &mut Parser, sink: FieldImp< '_>);
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

    pub fn	ParseGrammar( &mut self, grammar: &( impl IGrammar + ?Sized), mark: U32, sink: FieldImp< '_>) -> Option< U32>
    {
        let  	node = Forge {
            prev: self._TopForge,
            _CurrMark: mark,
            _IsMatched: false,
        };
        let  	prevTop = self._TopForge;
        self._TopForge = &node as *const Forge;
        grammar.Match( self, sink);
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
    pub fn	CurrentMark( &self) -> U32
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

impl IGrammar for Charset
{
    fn	Match( &self, parser: &mut Parser, _sink: FieldImp< '_>)
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
    fn	Match( &self, parser: &mut Parser, _sink: FieldImp< '_>)
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

impl IFluxExportSource for char
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for str
{
    fn	Match( &self, parser: &mut Parser, _sink: FieldImp< '_>)
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

impl< 'a, 'r, T: IGrammar + ?Sized> IGrammar for &'r T
{
    fn	Match( &self, parser: &mut Parser, sink: FieldImp< '_>)
    {
        (**self).Match( parser, sink);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


crate::ImplFluxImportSource!( char);
