//-- node.rs -------------------------------------------------------------------------------------------------------------------
use crate::silo::{Arr, U32};

//---------------------------------------------------------------------------------------------------------------------------------

pub enum Attrib {
    Inv(bool),
    Repeat(U32, U32),
    Action(Box<dyn Fn()>),
}

impl std::fmt::Debug for Attrib {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Attrib::Inv(value) => f.debug_tuple("Inv").field(value).finish(),
            Attrib::Repeat(left, right) => f.debug_tuple("Repeat").field(left).field(right).finish(),
            Attrib::Action(_) => f.write_str("Action(<closure>)"),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildOp {
    Sum,
    Prod,
    Less,
    Bor,
    Shl,
    Shr,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait INode {
    fn Attrib(&self) -> Option<&Attrib> {
        None
    }

    fn ChildOp(&self) -> Option<ChildOp> {
        None
    }

    fn Children<'a>(&'a self) -> Arr<'a, &'a dyn INode>;

    fn IsLeaf(&self) -> bool {
        self.Children().Size() == U32(0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
