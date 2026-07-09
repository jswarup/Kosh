use crate::silo::U32;
use crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use crate::stalks::work::DynIWork;
use crate::flux::{ IXFluxSource, xflux::XField };
use std::fmt;
use crate::segue::{ IGrammar, Parser };

pub struct ActionShard<'a> {
    pub _Child: Box<DynINode<'a>>,
    pub _Action: Box<DynIWork<'static>>,
}

//---------------------------------------------------------------------------------------------------------------------------------
// ActionShard Impls
//---------------------------------------------------------------------------------------------------------------------------------
impl<'a> IXFluxSource for ActionShard<'a> {
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
                *key = "Action".to_string();
                *item = XField::Str("Action");
                step += 1;
                true
            } else { false }
        }));
    }
}

impl<'a> INode<'a> for ActionShard<'a> {
    fn _Size(&self) -> U32 { U32(1) }
    fn _At(&self, idx: U32) -> &DynINode<'a> {
        if idx.0 == 0 {
            &*self._Child
        } else {
            panic!("At called on ActionShard with index > 0")
        }
    }
    fn Value(&self) -> Option<WorkPtr<'a>> { None }
    fn AsRawLeaf(&self) -> *const () { std::ptr::null() }
    fn DocStr(&self) -> &'static str { "" }
    fn BinOp(&self) -> BinOp { BinOp::None }
    fn Action(&self) -> Option<*const DynIWork<'static>> {
        Some(self._Action.as_ref() as *const _)
    }
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut crate::segue::Parser<'_>) };
        self.Match(p, crate::silo::U32(marker)).map(|u| u.0)
    }
}

impl<'a> IGrammar for ActionShard<'a> {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        if let Some(childMark) = self._Child.Match(parser, marker) {
            let actionPtr = &*self._Action as *const DynIWork<'static>;
            #[allow(invalid_reference_casting)]
            let actionMut = unsafe { &mut *(actionPtr as *mut DynIWork<'static>) };
            actionMut.DoWork(parser);
            return Some(childMark);
        }
        None
    }
}

impl<'a> fmt::Display for ActionShard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action")
    }
}

impl<'a> fmt::Debug for ActionShard<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
