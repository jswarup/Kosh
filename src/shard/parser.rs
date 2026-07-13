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
    fn	Deposit( &mut self, result: Option< U32>);
    fn	Result( &self) -> Option< U32>;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BaseForge< 'a, 'p, P: IForge< 'p>>
{
    pub     _Parent: &'a mut P,
    pub     _OrigMark: U32,
    pub     _CurrMark: U32,
    pub     _Result: Option< U32>,
    pub     _Phantom: std::marker::PhantomData<&'p ()>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'p, P: IForge< 'p>> BaseForge< 'a, 'p, P>
{
    pub fn	New( parent: &'a mut P) -> Self
    {
        let  	mark = parent.Mark();
        BaseForge {
            _Parent: parent,
            _OrigMark: mark,
            _CurrMark: mark,
            _Result: None,
            _Phantom: std::marker::PhantomData,
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'p, P: IForge< 'p>> IForge< 'p> for BaseForge< 'a, 'p, P>
{
    fn	Parser( &mut self) -> &mut Parser< 'p>
    {
        self._Parent.Parser()
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

impl< 'a, 'p, P: IForge< 'p>> Drop for BaseForge< 'a, 'p, P>
{
    fn	drop( &mut self)
    {
        if let Some( mark) = self._Result {
            self._Parent.Deposit( Some( mark));
        } else {
            self._Parent.SetMark( self._OrigMark);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

unsafe impl< 'a, 'p, P: IForge< 'p>> Send for BaseForge< 'a, 'p, P> {}
unsafe impl< 'a, 'p, P: IForge< 'p>> Sync for BaseForge< 'a, 'p, P> {}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IGrammar: INode
{
    fn	Forge< 'a, 'p, P: IForge< 'p> + 'a>( &'a self, parent: &'a mut P) -> impl IForge< 'p> + 'a
    where
        'p: 'a
    {
        BaseForge::New( parent)
    }

    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F);
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
    pub     _Result: Option< U32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'p> ParseForge<'a, 'p>
{
    pub fn	New( parser: &'a mut Parser<'p>, mark: U32) -> Self
    {
        ParseForge {
            _Parser: parser,
            _Marker: mark,
            _Result: None,
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

    fn	Deposit( &mut self, result: Option< U32>)
    {
        self._Result = result;
        if let Some( mark) = result {
            self._Marker = mark;
        }
    }

    fn	Result( &self) -> Option< U32>
    {
        self._Result
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
        grammar.Match( &mut forge);
        let  	matched = forge.Result().is_some();
        matched
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
    fn	Match< 'p, F: IForge< 'p>>(&self, forge: &mut F)
    {
        let  	mark = forge.Mark();
        let  	curr = forge.Parser().Curr( mark);
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
    fn	Match< 'p, F: IForge< 'p>>(&self, forge: &mut F)
    {
        let  	mark = forge.Mark();
        let  	curr = forge.Parser().Curr( mark);
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
    fn	Match< 'p, F: IForge< 'p>>(&self, forge: &mut F)
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
            let  	curr = forge.Parser().Curr( m);
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
    fn	Match< 'p, F: IForge< 'p>>(&self, forge: &mut F)
    {
        (**self).Match( forge);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

