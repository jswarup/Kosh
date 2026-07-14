//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::instream::IStream;
use	crate::flux::IXFluxSource;
use crate::shard::Charset;
use	crate::silo::{ U32, U8, Stash, IAccess };
use	crate::stalks::{ IWorker, WorkPtr, INode };

//---------------------------------------------------------------------------------------------------------------------------------
 
pub trait IForge: Send + Sync + 'static
{
    fn  New() -> Self where Self: Sized;
    fn	Mark( &self) -> U32; 
    fn	SetMark( &mut self, mark: U32);
    fn	Deposit( &mut self, result: Option< U32>);
    fn	Result( &self) -> Option< U32>;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BaseForge
{
    pub     _CurrMark: U32,
    pub     _Result: Option< U32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BaseForge
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IForge for BaseForge
{
    fn  New() -> Self
    {
        BaseForge {
            _CurrMark: U32( 0),
            _Result: None,
        }
    }

    fn	Mark( &self) -> U32
    {
        self._CurrMark
    }

    fn	SetMark( &mut self, mark: U32)
    {
        self._CurrMark = mark;
    }

    fn	Deposit( &mut self, result: Option< U32>)
    {
        self._Result = result;
        if let Some( mark) = result {
            self._CurrMark = mark;
        }
    }

    fn	Result( &self) -> Option< U32>
    {
        self._Result
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl Send for BaseForge {}
unsafe impl Sync for BaseForge {}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar: INode
{
    type Forge: IForge;

    fn	Match( &self, parser: &mut Parser, forge: &mut Self::Forge);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p>
{
    pub     _InStream: &'p mut dyn IStream,
    pub     _Forges: Stash< *mut dyn IForge>,
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
            _Forges: Stash::NewEmpty(),
        }
    }
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Parse< G: IGrammar + ?Sized>( &mut self, grammar: &'p G) -> bool
    {
        let  	mut forge = <G as IGrammar>::Forge::New();
        grammar.Match( self, &mut forge);
        let  	matched = forge.Result().is_some();
        matched
    }

    pub fn	PushForge( &mut self, forge: *mut dyn IForge)
    {
        self._Forges.Push( forge);
    }

    pub fn	PopForge( &mut self)
    {
        let  	sz = self._Forges.Size();
        if sz.0 > 0 {
            let  	ptr = self._Forges.Stk().Arr().At( sz - U32( 1));
            let  	mut dummy: *mut dyn IForge = *ptr;
            self._Forges.Pop( &mut dummy);
        }
    }

    pub fn	ParentForge( &self) -> Option< &mut dyn IForge>
    {
        let  	sz = self._Forges.Size();
        if sz.0 > 0 {
            let  	ptr = self._Forges.Stk().Arr().At( sz - U32( 1));
            Some( unsafe { &mut **ptr } )
        } else {
            None
        }
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
    type Forge = BaseForge;

    fn	Match( &self, parser: &mut Parser, forge: &mut Self::Forge)
    {
        let  	mark = forge.Mark();
        let  	curr = parser.Curr( mark);
        if self.Get( curr.0) {
            let  	res = Some( mark + U32( 1));
            forge.Deposit( res);
        } else {
            forge.Deposit( None);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    type Forge = BaseForge;

    fn	Match( &self, parser: &mut Parser, forge: &mut Self::Forge)
    {
        let  	mark = forge.Mark();
        let  	curr = parser.Curr( mark);
        if curr == U8( *self as u8) {
            let  	res = Some( mark + U32( 1));
            forge.Deposit( res);
        } else {
            forge.Deposit( None);
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
    type Forge = BaseForge;

    fn	Match( &self, parser: &mut Parser, forge: &mut Self::Forge)
    {
        // Ensure that empty string matches without consuming
        if self.is_empty() {
            let  	res = Some( forge.Mark());
            forge.Deposit( res);
            return;
        }

        let  	mark = forge.Mark();
        let  	mut m = mark;
        let  	bytes = self.as_bytes();
        for &b in bytes {
            let  	curr = parser.Curr( m);
            if curr.0 != b {
                forge.Deposit( None);
                return;
            }
            m += U32( 1);
        }

        let  	res = Some( m);
        forge.Deposit( res);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'r, T: IGrammar> IGrammar for &'r T
{
    type Forge = T::Forge;

    fn	Match( &self, parser: &mut Parser, forge: &mut Self::Forge)
    {
        (**self).Match( parser, forge);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

