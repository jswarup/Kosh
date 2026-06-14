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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraversalEvent {
    Entry(U32),
    Exit,
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

    fn TraverseDF(&self, fnMut: &mut dyn FnMut(&dyn INode, TraversalEvent))
    where
        Self: Sized,
    {
        traverse_df(self, fnMut);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> dyn INode + 'a {
    pub fn TraverseDF(&self, fnMut: &mut dyn FnMut(&dyn INode, TraversalEvent)) {
        traverse_df(self, fnMut);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub fn traverse_df(node: &dyn INode, fnMut: &mut dyn FnMut(&dyn INode, TraversalEvent)) {
    let mut stash = crate::silo::Stash::New(1024, 1, (node, U32(0)));
    while stash.Size() > U32(0) {
        let mut curr = (node, U32(0));
        let _res = stash.Pop(&mut curr);
        let (n, idx) = curr;
        let children = n.Children();
        let sz = children.Size();
        if idx < sz {
            fnMut(n, TraversalEvent::Entry(idx));
            stash.Push((n, idx + U32(1)));
            let child = *children.At(idx);
            stash.Push((child, U32(0)));
        } else {
            
            fnMut(n, TraversalEvent::Exit);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
