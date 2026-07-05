//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	std::io;
use	std::io::Read;
use	crate::{
    flux::InStream,
    segue::Charset
};
use	crate::silo::{ U8, U32, Stash, IAccess, IArr, cast::{ IPtrExt, IAllocRawExt } };
use	crate::stalks::{ BinOp, DynINode, DynIWorker, IWork, IWorker, WorkPtr };
use	crate::segue::shard::Shard;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug)]
pub struct  Digest
{
    pub _Start: U32,
    pub _End: U32,
}


//---------------------------------------------------------------------------------------------------------------------------------

pub trait IForge<'a, 'p, 's, R: Read + 'p>
    where 's: 'p, Self: 'a, 'p: 'a, 's: 'a
{
    fn	Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>;
    fn	Parser( &mut self) -> &mut Parser<'p, 's, R>;
    fn	Deposit( &self, childIdx: U32, digest: Digest);
    fn	MatchNode(&mut self) -> bool;

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EmitDigest( &self, start: U32, end: U32)
    {
        if let Some( p) = self.Parent() {
            p.Deposit( U32( 0), Digest { _Start: start, _End: end });
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	FindAncestor( &self, predicate: &mut dyn FnMut( &(dyn IForge<'a, 'p, 's, R> + 'a)) -> bool) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>
    {
        let  	mut curr = self.Parent();
        while let Some( parent) = curr {
            if ( predicate)( parent) {
                return Some( parent);
            }
            curr = parent.Parent();
        }
        None
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Init( &mut self) where Self: Sized
    {
        let  	selfRef: &mut (dyn IForge<'a, 'p, 's, R> + 'a) = self;
        let  	ptr = selfRef as *mut (dyn IForge<'a, 'p, 's, R> + 'a);
        let  	erased_ptr = ptr.CastLife::<dyn IForge<'p, 'p, 's, R> + 'p>();
        self.Parser()._Stash.Push( Some(erased_ptr));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}

//---------------------------------------------------------------------------------------------------------------------------------


macro_rules! ImplForgeBase {
    ($Type:ident) => {
        fn Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)> { self._Parent }
        fn Parser( &mut self) -> &mut Parser<'p, 's, R> { unsafe { &mut *self._Parser } }
        fn Deposit( &self, _childIdx: U32, _digest: Digest) {}
    };
}


pub trait IGrammar
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool;
}

pub trait IForgeable
{
    fn Forge<'a, 'p, 's, R: Read + 'p>(
        &'a self, 
        parser: *mut Parser<'p, 's, R>
    ) -> *mut (dyn IForge<'p, 'p, 's, R> + 'p) 
    where 's: 'p;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct  Forge<'a, 'p, 's, R: Read + 'p = io::Empty>
    where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: &'a mut Parser<'p, 's, R>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for Forge<'a, 'p, 's, R>
    where 's: 'p
{
    fn	Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>
    {
        self._Parent
    }

    fn	Parser( &mut self) -> &mut Parser<'p, 's, R>
    {
        self._Parser
    }

    fn	Deposit( &self, _childIdx: U32, _digest: Digest)
    {
    }
    fn MatchNode(&mut self) -> bool {
        false
    }
}


//---------------------------------------------------------------------------------------------------------------------------------


//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub     _InStream: &'p mut InStream<'s, R>,

    // Erase the 'f lifetime by transmuting to 'static to break the cyclic dropck dependency
    pub     _Stash: Stash< Option<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

// SAFETY: Parser is used single-threaded within a parse session.
// The raw pointers in _Stash are not shared across threads.
unsafe impl<'p, 's, R: Read + 'p> Send for Parser<'p, 's, R> {}
unsafe impl<'p, 's, R: Read + 'p> Sync for Parser<'p, 's, R> {}

impl<'p, 's, R: Read + 'p> IWorker for Parser<'p, 's, R>
where 's: 'p
{
    fn	PostJob( &self, job: WorkPtr< '_>)
    {
        if !job.IsNull() {
            ( job.func)( job.data, self);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


impl<'p, 's, R: Read + 'p> Parser<'p, 's, R>
where 's: 'p
{
    pub fn	New( stream: &'p mut InStream<'s, R>) -> Self
    {
        Self {
            _InStream: stream,
            _Stash: Stash::New( U32( 16), U32( 0), None),
        }
    }
    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ParseTree<T: 'static + IForgeable>( &mut self, node: &DynINode< '_>) -> Option<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>
    {

        let  selfPtr = self as *mut Parser<'p, 's, R>;
        let  mut forgeStk = unsafe { &mut *selfPtr }._Stash.Stk();
        let  opStash = Stash::<(BinOp, U32)>::New( 1024, 0, (BinOp::None, 0.into()));
        let  opStk = opStash.Stk();

        node.DiveDf( &mut |probe, enterFlg| {
            let  curNode = probe.CurNode().unwrap();
            let  curOp = curNode.BinOp();

            if enterFlg {
                if !curNode.IsLeaf() {
                    opStk.Push( ( curOp, forgeStk.Size()));
                    return;
                }

                let  anyRef = curNode.AsAny().unwrap();
                let  leaf = anyRef.downcast_ref::<T>().unwrap();
                let  forgePtr: *mut (dyn IForge<'p, 'p, 's, R> + 'p) = leaf.Forge(selfPtr);
                forgeStk.Push( Some( forgePtr));
                return;
            }
            if curNode.IsLeaf() {
                return;
            }
            let  mut opCtx = ( BinOp::None, 0.into());
            opStk.Pop( &mut opCtx);

            // Wait, we can't use parentOp to determine if we should skip, because UniNode has BinOp::None!
            // Let's remove this optimization if it breaks UniNode.
            // Actually, we must process every non-leaf node upon exit.

            let  arr = forgeStk.Arr().Subset( opCtx.1, forgeStk.Size() - opCtx.1);
            let mut children = crate::silo::Buff::NewEmpty();
            for i in 0..arr.Size().0 {
                if let Some(ptr) = *arr.At(crate::silo::U32(i)) {
                    children.Push(ptr);
                }
            }
            forgeStk.SetSize( opCtx.1);

            let forgePtr: *mut (dyn IForge<'p, 'p, 's, R> + 'p) = if let Some(crate::stalks::node::Attrib::Action(func)) = curNode.Attrib() {
                // It's a UniNode Action
                ActionForge {
                    _Parent: None,
                    _Parser: selfPtr,
                    _Child: children[0],
                    _Action: func.as_ref() as *const (dyn IWork + 'static),
                }.AllocRaw()
            } else {
                CompositeForge {
                    _Parent: None,
                    _Parser: selfPtr,
                    _Children: children,
                    _Mode: curOp,
                }.AllocRaw()
            };

            forgeStk.Push( Some( forgePtr));
        });

        if forgeStk.Size() == 0 {
            None
        } else {
            *forgeStk.Arr().Last()
        }
    }

    pub fn Parse< G: IGrammar>( &mut self, grammar: &G) -> bool
    {
        grammar.Match( self)
    }

    pub fn InStream( &mut self) -> &mut InStream<'s, R>
    {
        self._InStream
    }

    pub fn Forge<'f>( &self) -> Option<&'f (dyn IForge<'f, 'p, 's, R> + 'f)> where Self: 'f
    {
        let  sz = self._Stash.Size();
        if sz > U32( 0) {
            let  lastIdx = sz.0 - 1;
            let  arr = self._Stash.Stk().Arr();
            let  optPtr = arr.At( lastIdx);
            if let Some( ptr) = *optPtr {
                let  ptrF = ptr.CastLife::<dyn IForge<'f, 'p, 's, R> + 'f>();
                return Some( unsafe { &*ptrF });
            }
            None
        } else {
            None
        }
    }
}



impl<'p, 's, R: Read + 'p> Drop for Parser<'p, 's, R>
where 's: 'p
{
    fn drop(&mut self) {
        let  sz = self._Stash.Size().AsUsize();
        for i in 0..sz {
            if let Some(ptr) = *self._Stash.Stk().Arr().At(crate::silo::U32(i as u32)) {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------


//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Charset
{
    fn	Match< 'p, 'f, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let  	curr = parser.InStream().Curr();
        if self.Get( curr) {
            parser.InStream().Next();
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    fn	Match< 'p, 'f, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let  	curr = parser.InStream().Curr();
        if curr == U8( *self as u8) {
            parser.InStream().Next();
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for &str
{
    fn	Match< 'p, 'f, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let  	startMark = parser.InStream().Marker();

        // Ensure that empty string matches without consuming
        if self.is_empty() {
            return true;
        }

        for c in self.chars() {
            let  	curr = parser.InStream().Curr();
            if curr == U8( c as u8) {
                // If it's the last char, we just advance and we're good.
                let _ = parser.InStream().Next();
            } else {
                parser.InStream().RollTo( startMark);
                return false;
            }
        }

        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------


pub struct LeafForge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: *mut Parser<'p, 's, R>,
    pub _Shard: Option<&'a Shard>,
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for LeafForge<'a, 'p, 's, R>
where 's: 'p
{
    ImplForgeBase!( LeafForge);

    fn MatchNode(&mut self) -> bool {
        let startMark = self.Parser().InStream().Marker();
        if let Some(shard) = self._Shard {
            if shard.Match(self.Parser()) {
                let endMark = self.Parser().InStream().Marker();
                self.EmitDigest(startMark, endMark);
                return true;
            }
        }
        false
    }
}




pub struct CompositeForge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: *mut Parser<'p, 's, R>,
    pub _Children: crate::silo::Buff<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>,
    pub _Mode: BinOp,
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for CompositeForge<'a, 'p, 's, R>
where 's: 'p
{
    ImplForgeBase!( CompositeForge);

    fn MatchNode(&mut self) -> bool {
        let startMark = self.Parser().InStream().Marker();
        for i in 0..self._Children.Size().AsUsize() {
            let child_ptr = self._Children[i];
            let child_ref = unsafe { &mut *child_ptr };
            let matched = child_ref.MatchNode();

            if self._Mode == BinOp::Less {
                if !matched {
                    self.Parser().InStream().RollTo(startMark);
                    return false;
                }
            } else if self._Mode == BinOp::Bor {
                if matched {
                    let endMark = self.Parser().InStream().Marker();
                    self.EmitDigest(startMark, endMark);
                    return true;
                }
                self.Parser().InStream().RollTo(startMark);
            }
        }

        if self._Mode == BinOp::Less {
            let endMark = self.Parser().InStream().Marker();
            self.EmitDigest(startMark, endMark);
            return true;
        }

        false
    }
}

pub struct ActionForge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: *mut Parser<'p, 's, R>,
    pub _Child: *mut (dyn IForge<'p, 'p, 's, R> + 'p),
    pub _Action: *const (dyn IWork + 'static),
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for ActionForge<'a, 'p, 's, R>
where 's: 'p
{
    ImplForgeBase!( ActionForge);

    fn MatchNode(&mut self) -> bool {
        let startMark = self.Parser().InStream().Marker();
        let child_ref = unsafe { &mut *self._Child };
        let matched = child_ref.MatchNode();
        if matched {
            let action = unsafe { &mut *(self._Action as *mut dyn IWork) };
            let parser_ref = unsafe { &mut *self._Parser };
            action.DoWork(parser_ref as &DynIWorker<'_>);
            let endMark = self.Parser().InStream().Marker();
            self.EmitDigest(startMark, endMark);
            return true;
        }
        false
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> IGrammar for DynINode<'a>
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let root_forge_ptr = parser.ParseTree::<Shard>(self);
        if let Some(forge_ptr) = root_forge_ptr {
            let root_forge = unsafe { &mut *forge_ptr };
            return root_forge.MatchNode();
        }
        false
    }
}

//---------------------------------------------------------------------------------------------------------------------------------



//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'r> IGrammar for &'r DynINode<'a>
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        (*self).Match( parser)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

