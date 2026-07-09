use crate::silo::U32;
use crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use crate::stalks::work::DynIWork;
use crate::flux::{ IXFluxSource, xflux::XField };
use std::fmt;
use crate::shard::{ IGrammar, Parser };

pub struct RepeatShard<'a> {
    pub _Child: Box<DynINode<'a>>,
    pub _USeg: crate::silo::USeg,
}

//---------------------------------------------------------------------------------------------------------------------------------
// RepeatShard Impls
//---------------------------------------------------------------------------------------------------------------------------------
impl<'a> IXFluxSource for RepeatShard<'a> {
    fn ToXField<'b>(&'b self, field: &mut XField<'b>) {
        let mut step = 0u32;
        let node = self;
        *field = XField::Obj(Box::new(move |key, item| {
            if step == 0 {
                *key = "Child".to_string();
                node._Child.ToXField(item);
                step += 1;
                true
            } else if step == 1 {
                *key = "Repeat".to_string();
                *item = XField::FluxSource(&node._USeg);
                step += 1;
                true
            } else { false }
        }));
    }
}

impl<'a> INode<'a> for RepeatShard<'a> {
    fn _Size(&self) -> U32 { U32(1) }
    fn _At(&self, idx: U32) -> &DynINode<'a> {
        if idx.0 == 0 {
            &*self._Child
        } else {
            panic!("At called on RepeatShard with index > 0")
        }
    }
    fn Value(&self) -> Option<WorkPtr<'a>> { None }
    fn AsRawLeaf(&self) -> *const () { std::ptr::null() }
    fn DocStr(&self) -> &'static str { "" }
    fn BinOp(&self) -> BinOp { BinOp::None }
    fn Action(&self) -> Option<*const DynIWork<'static>> { None }
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut crate::shard::Parser<'_>) };
        self.Match(p, crate::silo::U32(marker)).map(|u| u.0)
    }
}

impl<'a> IGrammar for RepeatShard<'a> {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        let mut count = U32(0);
        let first = self._USeg.First();
        let last = if self._USeg.IsEmpty() { U32(std::u32::MAX) } else { self._USeg.Last() };
        let mut currMark = marker;

        while count < last {
            if let Some(newMark) = self._Child.Match(parser, currMark) {
                if newMark == currMark {
                    count += U32(1);
                    break;
                }
                currMark = newMark;
                count += U32(1);
            } else {
                break;
            }
        }

        if count >= first {
            Some(currMark)
        } else {
            None
        }
    }
}

impl<'a> fmt::Display for RepeatShard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Repeat({:?})", self._USeg)
    }
}

impl<'a> fmt::Debug for RepeatShard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
