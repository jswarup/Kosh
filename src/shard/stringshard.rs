//-- stringshard.rs -------------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ IGrammar, Parser };
use	crate::silo::U32;
use	crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use	crate::stalks::work::DynIWork;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct StringShard
{
    pub _Val: String,
}
//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for StringShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        *field = XField::Str( self._Val.as_str());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> INode< 'a> for StringShard
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
        self.Match( parserRef, U32( marker)).map( |u| u.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for StringShard
{
    fn	Match< 'p>( &'p self, parser: &mut Parser< 'p>, marker: U32) -> Option< U32>
    {
        self._Val.as_str().Match( parser, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for StringShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "StringShard( {:?})", self._Val)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for StringShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        fmt::Display::fmt( self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
