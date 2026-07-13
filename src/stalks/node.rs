//-- stalks/node.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::flux::{ IXFluxSource, xflux::XField };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum BinOp
{
    Sum = 0,
    Prod = 1,
    Sub = 2,
    Div = 3,
    Pow = 4,
    None = 5,

    Less = 6,
    Bor = 7,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq, Eq)]
pub struct BinNode< L, R, Op = BinOp>
{
    pub _Left: L,
    pub _Right: R,
    pub _Op: Op,
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, PartialEq, Eq)]
pub struct UniNode< C, Op>
{
    pub _Child: C,
    pub _Op: Op,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INode: IXFluxSource {}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: IXFluxSource + ?Sized> INode for T {}

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IXFluxSource for BinNode< L, R>
where
    L: IXFluxSource,
    R: IXFluxSource,
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        let  	node = self;
        let  	opVal = match node._Op {
            BinOp::Sum => 0,
            BinOp::Prod => 1,
            BinOp::Sub => 2,
            BinOp::Div => 3,
            BinOp::Pow => 4,
            BinOp::None => 5,
            BinOp::Less => 6,
            BinOp::Bor => 7,
        };
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Op".to_string();
                *item = XField::U64( opVal);
                step += 1;
                return true;
            }
            if step == 1 {
                *key = "Left".to_string();
                node._Left.ToXField( item);
                step += 1;
                return true;
            }
            if step == 2 {
                *key = "Right".to_string();
                node._Right.ToXField( item);
                step += 1;
                return true;
            }
            false
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
