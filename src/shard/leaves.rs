//-- leaves.rs -------------------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ Charset, IGrammar, Parser };
use	crate::silo::U32;
use	crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use	crate::stalks::work::DynIWork;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct StrShard< 'a>
{
    pub _Val: &'a str,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IXFluxSource for StrShard< 'a>
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        *field = XField::Str( self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> INode< 'a> for StrShard< 'a>
{
    fn	_Size( &self) -> U32
    {
        U32( 0)
    }

    fn	_At( &self, _idx: U32) -> &DynINode< 'a>
    {
        panic!( "Leaf")
    }

    fn	Value( &self) -> Option< WorkPtr< 'a>>
    {
        None
    }

    fn	AsRawLeaf( &self) -> *const ()
    {
        std::ptr::null()
    }

    fn	DocStr( &self) -> &'static str
    {
        ""
    }

    fn	BinOp( &self) -> BinOp
    {
        BinOp::None
    }

    fn	Action( &self) -> Option< *const DynIWork< 'static>>
    {
        None
    }

    fn	MatchGrammar( &self, parser: *mut (), marker: u32) -> Option< u32>
    {
        let  	parserRef = unsafe { &mut *( parser as *mut Parser< '_>) };
        
        return self.Match( parserRef, U32( marker)).map( |u| u.0);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for StrShard< 'a>
{
    fn	Match< 'p>( &'p self, parser: &mut Parser< 'p>, marker: U32) -> Option< U32>
    {
        return self._Val.Match( parser, marker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Display for StrShard< 'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "StrShard( {:?})", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Debug for StrShard< 'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return fmt::Display::fmt( self, f);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct CharsetShard
{
    pub _Val: Charset,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for CharsetShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	s = self._Val.to_string();
        *field = XField::String( s);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> INode< 'a> for CharsetShard
{
    fn	_Size( &self) -> U32
    {
        U32( 0)
    }

    fn	_At( &self, _idx: U32) -> &DynINode< 'a>
    {
        panic!( "Leaf")
    }

    fn	Value( &self) -> Option< WorkPtr< 'a>>
    {
        None
    }

    fn	AsRawLeaf( &self) -> *const ()
    {
        std::ptr::null()
    }

    fn	DocStr( &self) -> &'static str
    {
        ""
    }

    fn	BinOp( &self) -> BinOp
    {
        BinOp::None
    }

    fn	Action( &self) -> Option< *const DynIWork< 'static>>
    {
        None
    }

    fn	MatchGrammar( &self, parser: *mut (), marker: u32) -> Option< u32>
    {
        let  	parserRef = unsafe { &mut *( parser as *mut Parser< '_>) };
        
        return self.Match( parserRef, U32( marker)).map( |u| u.0);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for CharsetShard
{
    fn	Match< 'p>( &'p self, parser: &mut Parser< 'p>, marker: U32) -> Option< U32>
    {
        return self._Val.Match( parser, marker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for CharsetShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "{}", self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for CharsetShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "{:?}", self._Val);
    }
}
