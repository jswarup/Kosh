//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	std::io;
use	std::io::Read;
use	crate::{
    flux::InStream,
    segue::Charset
};
use	crate::silo::{ U8, U32, Buff, Stash, IAccess, IArr, cast::IAllocRawExt };
use	crate::stalks::{ BinOp, DynINode };
use	crate::segue::shard::Shard;

//---------------------------------------------------------------------------------------------------------------------------------

pub use	crate::stalks::node::ActionFn;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug)]
pub struct  Digest
{
    pub _Start: U32,
    pub _End: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IForge<'a, 'p, 's, R: Read + 'p>
    where 's: 'p, 'p: 'a
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

}

//---------------------------------------------------------------------------------------------------------------------------------

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

pub enum ForgeKind<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    Leaf( Option<&'a Shard<'a>>),
    Composite( Buff<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>, BinOp),
    Action( *mut (dyn IForge<'p, 'p, 's, R> + 'p), *mut core::ffi::c_void),
    Repeat( *mut (dyn IForge<'p, 'p, 's, R> + 'p), crate::silo::USeg),
}

pub struct ForgeNode<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: *mut Parser<'p, 's, R>,
    pub _Kind: ForgeKind<'a, 'p, 's, R>,
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for ForgeNode<'a, 'p, 's, R>
where 's: 'p
{
    fn Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)> { self._Parent }
    fn Parser( &mut self) -> &mut Parser<'p, 's, R> { unsafe { &mut *self._Parser } }
    fn Deposit( &self, _childIdx: U32, _digest: Digest) {}

    fn MatchNode(&mut self) -> bool {
        match &mut self._Kind {
            ForgeKind::Leaf( shard_opt) => {
                let  	parser = unsafe { &mut *self._Parser };
                let  	startMark = parser.InStream().Marker();
                if let Some( shard) = shard_opt {
                    let  	shard = *shard;
                    if shard.Match( parser) {
                        let endMark = parser.InStream().Marker();
                        self.EmitDigest( startMark, endMark);
                        return true;
                    }
                }
                false
            }

            ForgeKind::Composite( children, mode) => {
                let  	startMark = unsafe { &mut *self._Parser }.InStream().Marker();
                let  	mode = *mode;
                for i in 0..children.Size().AsUsize() {
                    let  	child_ptr = children[i];
                    let  	child_ref = unsafe { &mut *child_ptr };
                    let  	matched = child_ref.MatchNode();
                    if mode == BinOp::Less {
                        if !matched {
                            unsafe { &mut *self._Parser }.InStream().RollTo( startMark);
                            return false;
                        }
                    } else if mode == BinOp::Bor {
                        if matched {
                            let endMark = unsafe { &mut *self._Parser }.InStream().Marker();
                            self.EmitDigest( startMark, endMark);
                            return true;
                        }
                        unsafe { &mut *self._Parser }.InStream().RollTo( startMark);
                    }
                }
                if mode == BinOp::Less {
                    let endMark = unsafe { &mut *self._Parser }.InStream().Marker();
                    self.EmitDigest( startMark, endMark);
                    return true;
                }
                false
            }

            ForgeKind::Action( child_ptr, action_ptr) => {
                let  	startMark = unsafe { &mut *self._Parser }.InStream().Marker();
                let  	child_ref = unsafe { &mut **child_ptr };
                if child_ref.MatchNode() {
                    let  	action_box = unsafe { &mut *(*action_ptr as *mut ActionFn) };
                    action_box( self._Parser as *mut core::ffi::c_void);
                    let endMark = unsafe { &mut *self._Parser }.InStream().Marker();
                    self.EmitDigest( startMark, endMark);
                    return true;
                }
                false
            }

            ForgeKind::Repeat( child_ptr, useg) => {
                let  	startMark = unsafe { &mut *self._Parser }.InStream().Marker();
                let  	child_ref = unsafe { &mut **child_ptr };
                let  	min = useg.First().AsU32();
                let  	max = useg.Size().AsU32();
                let  	is_inf = max == 0;
                let  	mut match_count = 0u32;
                while is_inf || match_count < max {
                    let iter_start = unsafe { &mut *self._Parser }.InStream().Marker();
                    if !child_ref.MatchNode() {
                        break;
                    }
                    if unsafe { &mut *self._Parser }.InStream().Marker() == iter_start {
                        break;
                    }
                    match_count += 1;
                }
                if match_count >= min {
                    let endMark = unsafe { &mut *self._Parser }.InStream().Marker();
                    self.EmitDigest( startMark, endMark);
                    true
                } else {
                    unsafe { &mut *self._Parser }.InStream().RollTo( startMark);
                    false
                }
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub     _InStream: &'p mut InStream<'s, R>,
    pub     _Stash: Stash< Option<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

// SAFETY: Parser is used single-threaded within a parse session.
// The raw pointers in _Stash are not shared across threads.
unsafe impl<'p, 's, R: Read + 'p> Send for Parser<'p, 's, R> {}
unsafe impl<'p, 's, R: Read + 'p> Sync for Parser<'p, 's, R> {}

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
        let  	selfPtr = self as *mut Parser<'p, 's, R>;
        let  	mut forgeStk = unsafe { &mut *selfPtr }._Stash.Stk();
        let  	opStash = Stash::<(BinOp, U32)>::New( 64, 0, (BinOp::None, 0.into()));
        let  	opStk = opStash.Stk();

        node.DiveDf( &mut |probe, enterFlg| {
            let  	curNode = probe.CurNode().unwrap();

            if enterFlg {
                if !curNode.IsLeaf() {
                    opStk.Push( ( curNode.BinOp(), forgeStk.Size()));
                    return;
                }
                let  	raw_ptr = curNode.AsRawLeaf();
                let  	leaf: &T = if raw_ptr.is_null() {
                    curNode.AsAny().unwrap().downcast_ref::<T>().unwrap()
                } else {
                    unsafe { &*(raw_ptr as *const T) }
                };
                forgeStk.Push( Some( leaf.Forge( selfPtr)));
                return;
            }

            if curNode.IsLeaf() {
                return;
            }

            let  	mut opCtx = ( BinOp::None, 0.into());
            opStk.Pop( &mut opCtx);
            let  	arr = forgeStk.Arr().Subset( opCtx.1, forgeStk.Size() - opCtx.1);
            let  	mut children = Buff::NewEmpty();
            for i in 0..arr.Size().0 {
                if let Some( ptr) = *arr.At( U32( i)) {
                    children.Push( ptr);
                }
            }
            forgeStk.SetSize( opCtx.1);

            let  	kind = if let Some( action) = curNode.Action() {
                ForgeKind::Action( children[0], action)
            } else if let Some( useg) = curNode.Repeat() {
                ForgeKind::Repeat( children[0], useg)
            } else {
                ForgeKind::Composite( children, opCtx.0)
            };

            let  	forgePtr: *mut (dyn IForge<'p, 'p, 's, R> + 'p) = ForgeNode {
                _Parent: None,
                _Parser: selfPtr,
                _Kind: kind,
            }.AllocRaw();
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
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'p, 's, R: Read + 'p> Drop for Parser<'p, 's, R>
where 's: 'p
{
    fn drop(&mut self) {
        let  	sz = self._Stash.Size().AsUsize();
        for i in 0..sz {
            if let Some(ptr) = *self._Stash.Stk().Arr().At(U32(i as u32)) {
                unsafe {
                    drop(Box::from_raw(ptr));
                }
            }
        }
    }
}

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

        if self.is_empty() {
            return true;
        }

        for c in self.chars() {
            let  	curr = parser.InStream().Curr();
            if curr == U8( c as u8) {
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

impl<'a> IGrammar for DynINode<'a>
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let  	root_forge_ptr = parser.ParseTree::<Shard<'_>>( self);
        if let Some( forge_ptr) = root_forge_ptr {
            let  	root_forge = unsafe { &mut *forge_ptr };
            return root_forge.MatchNode();
        }
        false
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
