use crate::silo::U32;
use crate::stalks::{ BinOp, DynINode, INode, WorkPtr };
use crate::stalks::work::DynIWork;
use crate::flux::{ IXFluxSource, xflux::XField };
use std::fmt;
use crate::segue::{ Charset, IGrammar, Parser };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ParNode<'a> {
    pub _Left: Box<DynINode<'a>>,
    pub _Right: Box<DynINode<'a>>,
}

pub struct CatNode<'a> {
    pub _Left: Box<DynINode<'a>>,
    pub _Right: Box<DynINode<'a>>,
}

pub struct RepeatNode<'a> {
    pub _Child: Box<DynINode<'a>>,
    pub _USeg: crate::silo::USeg,
}

pub struct ActionNode<'a> {
    pub _Child: Box<DynINode<'a>>,
    pub _Action: Box<DynIWork<'static>>,
}

pub trait IntoDynNode<'a> {
    fn IntoDynNode(self) -> Box<DynINode<'a>>;
}

impl<'a> IntoDynNode<'a> for &'static str {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        Box::new(self.to_string())
    }
}

impl<'a> IntoDynNode<'a> for String {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        Box::new(self)
    }
}

impl<'a> IntoDynNode<'a> for char {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        Box::new(self.to_string())
    }
}

impl<'a> IntoDynNode<'a> for Charset {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        Box::new(self)
    }
}

impl<'a> IntoDynNode<'a> for Box<DynINode<'a>> {
    fn IntoDynNode(self) -> Box<DynINode<'a>> {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// ParNode Impls
//---------------------------------------------------------------------------------------------------------------------------------
impl<'a> IXFluxSource for ParNode<'a> {
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

impl<'a> INode<'a> for ParNode<'a> {
    fn _Size(&self) -> U32 { U32(2) }
    fn _At(&self, idx: U32) -> &DynINode<'a> {
        match idx.0 {
            0 => &*self._Left,
            1 => &*self._Right,
            _ => panic!("At called on ParNode with index > 1"),
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

impl<'a> IGrammar for ParNode<'a> {
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

impl<'a> fmt::Display for ParNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParNode")
    }
}

impl<'a> fmt::Debug for ParNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// CatNode Impls
//---------------------------------------------------------------------------------------------------------------------------------
impl<'a> IXFluxSource for CatNode<'a> {
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

impl<'a> INode<'a> for CatNode<'a> {
    fn _Size(&self) -> U32 { U32(2) }
    fn _At(&self, idx: U32) -> &DynINode<'a> {
        match idx.0 {
            0 => &*self._Left,
            1 => &*self._Right,
            _ => panic!("At called on CatNode with index > 1"),
        }
    }
    fn Value(&self) -> Option<WorkPtr<'a>> { None }
    fn AsRawLeaf(&self) -> *const () { std::ptr::null() }
    fn DocStr(&self) -> &'static str { "" }
    fn BinOp(&self) -> BinOp { BinOp::Less }
    fn Action(&self) -> Option<*const DynIWork<'static>> { None }
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut crate::segue::Parser<'_>) };
        self.Match(p, crate::silo::U32(marker)).map(|u| u.0)
    }
}

impl<'a> IGrammar for CatNode<'a> {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        if let Some(leftMark) = self._Left.Match(parser, marker) {
            if let Some(rightMark) = self._Right.Match(parser, leftMark) {
                return Some(rightMark);
            }
        }
        None
    }
}

impl<'a> fmt::Display for CatNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CatNode")
    }
}

impl<'a> fmt::Debug for CatNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// RepeatNode Impls
//---------------------------------------------------------------------------------------------------------------------------------
impl<'a> IXFluxSource for RepeatNode<'a> {
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

impl<'a> INode<'a> for RepeatNode<'a> {
    fn _Size(&self) -> U32 { U32(1) }
    fn _At(&self, idx: U32) -> &DynINode<'a> {
        if idx.0 == 0 {
            &*self._Child
        } else {
            panic!("At called on RepeatNode with index > 0")
        }
    }
    fn Value(&self) -> Option<WorkPtr<'a>> { None }
    fn AsRawLeaf(&self) -> *const () { std::ptr::null() }
    fn DocStr(&self) -> &'static str { "" }
    fn BinOp(&self) -> BinOp { BinOp::None }
    fn Action(&self) -> Option<*const DynIWork<'static>> { None }
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let p = unsafe { &mut *(parser as *mut crate::segue::Parser<'_>) };
        self.Match(p, crate::silo::U32(marker)).map(|u| u.0)
    }
}

impl<'a> IGrammar for RepeatNode<'a> {
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

impl<'a> fmt::Display for RepeatNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Repeat({:?})", self._USeg)
    }
}

impl<'a> fmt::Debug for RepeatNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
// ActionNode Impls
//---------------------------------------------------------------------------------------------------------------------------------
impl<'a> IXFluxSource for ActionNode<'a> {
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

impl<'a> INode<'a> for ActionNode<'a> {
    fn _Size(&self) -> U32 { U32(1) }
    fn _At(&self, idx: U32) -> &DynINode<'a> {
        if idx.0 == 0 {
            &*self._Child
        } else {
            panic!("At called on ActionNode with index > 0")
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

impl<'a> IGrammar for ActionNode<'a> {
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

impl<'a> fmt::Display for ActionNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Action")
    }
}

impl<'a> fmt::Debug for ActionNode<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ShardTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_STAR   $( $args:tt)* ) => { $crate::NodeTree!( @feature_STAR   $( $args)* ) };
    ( @feature_PLUS   $( $args:tt)* ) => { $crate::NodeTree!( @feature_PLUS   $( $args)* ) };

    ( @feature_LT     $( $args:tt)* ) => { $crate::NodeTree!( @feature_LT     $( $args)* ) };
    ( @feature_SHL    $( $args:tt)* ) => { $crate::NodeTree!( @feature_SHL    $( $args)* ) };

    ( @feature_NEW    $( $args:tt)* ) => { $crate::NodeTree!( @feature_NEW    $( $args)* ) };
    ( @feature_PostBoxet $( $args:tt)* ) => { $crate::NodeTree!( @feature_PostBoxet $( $args)* ) };

    // ── Shard AST Hooks (overrides NodeTree default) ──────────────────────────────────────────────
    ( @feature_RESOLVE_LEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        $crate::segue::shard::IntoDynNode::IntoDynNode($val)
    };
    ( @feature_NEWLEAF [ $( $cb:tt)* ], $Arg:ident, $val:expr ) => {
        $crate::segue::shard::IntoDynNode::IntoDynNode($val)
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Bor, $l:expr, $r:expr ) => {
        Box::new($crate::segue::shard::ParNode { _Left: $l, _Right: $r }) as Box<$crate::stalks::DynINode<'static>>
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, Less, $l:expr, $r:expr ) => {
        Box::new($crate::segue::shard::CatNode { _Left: $l, _Right: $r }) as Box<$crate::stalks::DynINode<'static>>
    };
    ( @feature_NEWBINNODE [ $( $cb:tt)* ], $Arg:ident, $op:ident, $l:expr, $r:expr ) => {
        compile_error!("ShardTree only supports ParNode (Bor) and CatNode (Less).")
    };
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $action:expr, $child:expr ) => {
        Box::new($crate::segue::shard::ActionNode { _Child: $child, _Action: $action }) as Box<$crate::stalks::DynINode<'static>>
    };
    ( @feature_REPEAT_STAR [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        Box::new($crate::segue::shard::RepeatNode { _Child: $child, _USeg: $crate::silo::USeg::NewInf( 0) }) as Box<$crate::stalks::DynINode<'static>>
    };
    ( @feature_REPEAT_PLUS [ $( $cb:tt)* ], $Arg:ident, $child:expr ) => {
        Box::new($crate::segue::shard::RepeatNode { _Child: $child, _USeg: $crate::silo::USeg::NewInf( 1) }) as Box<$crate::stalks::DynINode<'static>>
    };

    // ── Custom: Boxet stringification (overrides NodeTree default) ─────────────────────────────────
    ( @feature_BOXET [ $( $cb:tt)* ], $Arg:ident, $s:literal ) => {
        Box::new(<$crate::segue::Charset>::from( $s.as_bytes() )) as Box<$crate::stalks::DynINode<'static>>
    };

    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    ( @ $( $inner:tt )+ ) => {
        $crate::NodeTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::NodeTree!( @define [ $crate::ShardTree ], DynINode, $( $inner)+ )
    };
}
