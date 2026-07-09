use crate::silo::U32;
use crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use crate::stalks::work::DynIWork;
use crate::flux::{ IXFluxSource, xflux::XField };
use std::fmt;
use crate::segue::{ IGrammar, Parser };

pub struct ParShard<'a> {
    pub _Left: Box<DynINode<'a>>,
    pub _Right: Box<DynINode<'a>>,
}

//---------------------------------------------------------------------------------------------------------------------------------
// ParShard Impls
//---------------------------------------------------------------------------------------------------------------------------------
impl<'a> IXFluxSource for ParShard<'a> {
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

impl<'a> INode<'a> for ParShard<'a> {
    fn _Size(&self) -> U32 { U32(2) }
    fn _At(&self, idx: U32) -> &DynINode<'a> {
        match idx.0 {
            0 => &*self._Left,
            1 => &*self._Right,
            _ => panic!("At called on ParShard with index > 1"),
        }
    }
    fn Value(&self) -> Option<WorkPtr<'a>> { None }
    fn AsRawLeaf(&self) -> *const () { std::ptr::null() }
    fn DocStr(&self) -> &'static str { "" }
    fn BinOp(&self) -> BinOp { BinOp::Bor }
    fn Action(&self) -> Option<*const DynIWork<'static>> { None }
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut crate::segue::Parser<'_>) };
        self.Match(p, crate::silo::U32(marker)).map(|u| u.0)
    }
}

impl<'a> IGrammar for ParShard<'a> {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        if let Some(leftMark) = self._Left.Match(parser, marker) {
            return Some(leftMark);
        }
        if let Some(rightMark) = self._Right.Match(parser, marker) {
            return Some(rightMark);
        }
        None
    }
}

impl<'a> fmt::Display for ParShard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParShard")
    }
}

impl<'a> fmt::Debug for ParShard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
