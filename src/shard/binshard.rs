//-- binshard.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ IGrammar, Parser };
use	crate::silo::U32;
use	crate::stalks::{ BinOp, DynINode, INode };
use	crate::stalks::work::DynIWork;

//---------------------------------------------------------------------------------------------------------------------------------

pub enum BinShardOp
{
    Choice,
    Sequence,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BinShard< 'a>
{
    pub _Left: &'a DynINode< 'a>,
    pub _Right: &'a DynINode< 'a>,
    pub _Op: BinShardOp,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IXFluxSource for BinShard< 'a>
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	node = self;
        
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Left".to_string();
                node._Left.ToXField( item);
                step += 1;
                
                return true;
            } else if step == 1 {
                *key = "Right".to_string();
                node._Right.ToXField( item);
                step += 1;
                
                return true;
            } else {
                return false;
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> INode< 'a> for BinShard< 'a>
{
    fn	_Size( &self) -> U32
    {
        return U32( 2);
    }

    fn	_At( &self, idx: U32) -> &DynINode< 'a>
    {
        match idx.0 {
            0 => {
                return self._Left;
            }
            1 => {
                return self._Right;
            }
            _ => {
                panic!( "At called on BinShard with index > 1");
            }
        }
    }



    fn	BinOp( &self) -> BinOp
    {
        match self._Op {
            BinShardOp::Choice => {
                return BinOp::Bor;
            }
            BinShardOp::Sequence => {
                return BinOp::Less;
            }
        }
    }

    fn	Action( &self) -> Option< *const DynIWork< 'static>>
    {
        return None;
    }

    fn	MatchGrammar( &self, parser: *mut (), marker: u32) -> Option< u32>
    {
        let  	p = unsafe { &mut *( parser as *mut crate::shard::Parser< '_>) };
        
        return self.Match( p, crate::silo::U32( marker)).map( |u| u.0);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for BinShard< 'a>
{
    fn	Match< 'p>( &'p self, parser: &mut Parser< 'p>, marker: U32) -> Option< U32>
    {
        match self._Op {
            BinShardOp::Choice => {
                if let Some( leftMark) = self._Left.Match( parser, marker) {
                    return Some( leftMark);
                }
                if let Some( rightMark) = self._Right.Match( parser, marker) {
                    return Some( rightMark);
                }
                
                return None;
            }
            BinShardOp::Sequence => {
                if let Some( leftMark) = self._Left.Match( parser, marker) {
                    if let Some( rightMark) = self._Right.Match( parser, leftMark) {
                        return Some( rightMark);
                    }
                }
                
                return None;
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Display for BinShard< 'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        match self._Op {
            BinShardOp::Choice => {
                return write!( f, "ParShard");
            }
            BinShardOp::Sequence => {
                return write!( f, "CatShard");
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Debug for BinShard< 'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return fmt::Display::fmt( self, f);
    }
}
