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

pub struct BinShard< L, R>
{
    pub _Left: L,
    pub _Right: R,
    pub _Op: BinShardOp,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IXFluxSource for BinShard< L, R>
where
    L: IXFluxSource,
    R: IXFluxSource,
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

impl< 'a, L, R> INode< 'a> for BinShard< L, R>
where
    L: INode< 'a> + IGrammar + Send + Sync + 'a,
    R: INode< 'a> + IGrammar + Send + Sync + 'a,
{
    //-----------------------------------------------------------------------------------------------------------------------------
    
    fn	_Size( &self) -> U32
    {
        return U32( 2);
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    
    fn	_At( &self, idx: U32) -> &DynINode< 'a>
    {
        match idx.0 {
            0 => {
                return &self._Left;
            }
            1 => {
                return &self._Right;
            }
            _ => {
                panic!( "At called on BinShard with index > 1");
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

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

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Action( &self) -> Option< *const DynIWork< 'static>>
    {
        return None;
    }

    //-----------------------------------------------------------------------------------------------------------------------------
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IGrammar for BinShard< L, R>
where
    L: IGrammar,
    R: IGrammar,
{
    fn	Match<'p>(&self, parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        match self._Op {
            BinShardOp::Choice => {
                let (matched, m) = self._Left.Match( parser, marker);
                if matched {
                    return (true, m);
                }
                let (matched, m) = self._Right.Match( parser, marker);
                if matched {
                    return (true, m);
                }
                
                (false, marker)
            }
            BinShardOp::Sequence => {
                let (matched, m) = self._Left.Match( parser, marker);
                if matched {
                    let (matched_right, m2) = self._Right.Match( parser, m);
                    if matched_right {
                        return (true, m2);
                    }
                }
                
                (false, marker)
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> fmt::Display for BinShard< L, R>
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

impl< L, R> fmt::Debug for BinShard< L, R>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return fmt::Display::fmt( self, f);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
