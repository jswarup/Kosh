//-- catshard.rs --------------------------------------------------------------------------------------------------------------------
use crate::silo::U32;
use crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use crate::stalks::work::DynIWork;
use crate::flux::{ IXFluxSource, xflux::XField };
use std::fmt;
use crate::shard::{ IGrammar, Parser };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct CatShard<'a> {
    pub _Left: &'a DynINode<'a>,
    pub _Right: &'a DynINode<'a>,
}

//---------------------------------------------------------------------------------------------------------------------------------
 
 impl<'a> IXFluxSource for CatShard<'a> {
    fn ToXField<'b>(&'b self, field: &mut XField<'b>) {
        let mut step = 0u32;
        let node = self;
        *field = XField::Obj(Box::new(move |key, item| {
            if step == 0 {
                *key = "Left".to_string();
                node._Left.ToXField(item);
                step += 1;
                true
            } else if step == 1 {
                *key = "Right".to_string();
                node._Right.ToXField(item);
                step += 1;
                true
            } else { false }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> INode<'a> for CatShard<'a> {
    fn _Size(&self) -> U32 { U32(2) }
    fn _At(&self, idx: U32) -> &DynINode<'a> {
        match idx.0 {
            0 => self._Left,
            1 => self._Right,
            _ => panic!("At called on CatShard with index > 1"),
        }
    }
    fn Value(&self) -> Option<WorkPtr<'a>> { None }
    fn AsRawLeaf(&self) -> *const () { std::ptr::null() }
    fn DocStr(&self) -> &'static str { "" }
    fn BinOp(&self) -> BinOp { BinOp::Less }
    fn Action(&self) -> Option<*const DynIWork<'static>> { None }
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut crate::shard::Parser<'_>) };
        self.Match(p, crate::silo::U32(marker)).map(|u| u.0)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> IGrammar for CatShard<'a> {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        if let Some(leftMark) = self._Left.Match(parser, marker) {
            if let Some(rightMark) = self._Right.Match(parser, leftMark) {
                return Some(rightMark);
            }
        }
        None
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> fmt::Display for CatShard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CatShard")
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> fmt::Debug for CatShard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
