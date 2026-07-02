//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	std::io;
use	std::io::Read;
use	crate::{
    flux::InStream,
    segue::Charset
};
use	crate::silo::{ U8, U32, Stash, IAccess, IArr, cast::{ ICastExt, IPtrExt, IAllocRawExt } };
use	crate::stalks::{ ChildOp, DynINode };
use	crate::segue::shard::Shard;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Copy, Debug)]
pub struct  Digest
{
    pub _Start: U32,
    pub _End: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IForge<'a, 'p, 's, R: Read + 'p>
    where 's: 'p, Self: 'a
{
    fn	Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>;
    fn	Parser( &mut self) -> &mut Parser<'p, 's, R>;
    fn	Deposit( &self, childIdx: U32, digest: Digest);
    fn	MatchNode(&mut self) -> bool;
    
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

pub trait IGrammar
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool;
}

//---------------------------------------------------------------------------------------------------------------------------------
pub struct  Forge<'a, 'p, 's, R: Read + 'p = io::Empty>
    where 's: 'p 
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: &'a mut Parser<'p, 's, R>,
}

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

pub trait IParser<'p, 's, R: Read + 'p>
where 's: 'p
{
    fn	Parse< G: IGrammar>( &mut self, grammar: &G) -> bool;
    fn	InStream( &mut self) -> &mut InStream<'s, R>;
    fn	Forge<'f>( &self) -> Option<&'f (dyn IForge<'f, 'p, 's, R> + 'f)> where Self: 'f;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _InStream: &'p mut InStream<'s, R>,
    // Erase the 'f lifetime by transmuting to 'static to break the cyclic dropck dependency
    pub _Stash: Stash< Option<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>>,
    
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

    pub fn	ParseShardTree( &mut self, node: &DynINode< '_>) -> Option<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>
    {
        
        let  selfPtr = self as *mut Parser<'p, 's, R>;
        let  mut forgeStk = unsafe { &mut *selfPtr }._Stash.Stk();
        let  opStash = Stash::<(ChildOp, U32)>::New( 1024, 0, (ChildOp::None, 0.into()));
        let  opStk = opStash.Stk();

        node.DiveDf( &mut |probe, enterFlg| {
            let  curNode = probe.CurNode().unwrap();
            let  curOp = curNode.ChildOp();
            
            if enterFlg {
                if curOp != ChildOp::None {
                    opStk.Push( ( curOp, forgeStk.Size()));
                    return;
                } 
                
                let  anyRef = curNode.AsAny().unwrap();
                let  shard = anyRef.downcast_ref::<Shard>().unwrap();
                
                let  shardPtr = Some(shard).Cast::<Option<&'static Shard>>();
                let  forgePtr: *mut (dyn IForge<'p, 'p, 's, R> + 'p) = match shard {
                    Shard::Charset(_) => {
                        CharsetForge {
                            _Parent: None,
                            _Parser: selfPtr,
                            _Shard: shardPtr,
                        }.AllocRaw()
                    },
                    Shard::String(_) => {
                        StringForge {
                            _Parent: None,
                            _Parser: selfPtr,
                            _Shard: shardPtr,
                        }.AllocRaw()
                    },
                };
                forgeStk.Push( Some( forgePtr));
                return;
            }
            if curOp == ChildOp::None { 
                return; 
            }
            let  mut opCtx = ( ChildOp::None, 0.into());
            opStk.Pop( &mut opCtx); 

            let  parentOp = if opStk.Size() != 0 { opStk.Arr().Last().0 } else { ChildOp::None };
            if parentOp == curOp { 
                return; 
            }

            let  arr = forgeStk.Arr().Subset( opCtx.1, forgeStk.Size() - opCtx.1);
            let mut children = crate::silo::Buff::NewEmpty();
            for i in 0..arr.Size().0 {
                if let Some(ptr) = *arr.At(crate::silo::U32(i)) {
                    children.Push(ptr);
                }
            }
            forgeStk.SetSize( opCtx.1);
            
            let  forgePtr: *mut (dyn IForge<'p, 'p, 's, R> + 'p) = match curOp {
                ChildOp::Less => {
                    CatArrForge {
                        _Parent: None,
                        _Parser: selfPtr,
                        _Children: children,
                    }.AllocRaw()
                },
                ChildOp::Bor => {
                    ParArrForge {
                        _Parent: None,
                        _Parser: selfPtr,
                        _Children: children,
                    }.AllocRaw()
                },
                _ => panic!( "Unsupported ChildOp in ParseShardTree: {:?}", curOp),
            };
            
            forgeStk.Push( Some( forgePtr));
        }); 
        
        if forgeStk.Size() == 0 {
            None
        } else {
            *forgeStk.Arr().Last()
        }
    }

}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'p, 's, R: Read + 'p> IParser<'p, 's, R> for Parser<'p, 's, R>
where 's: 'p
{
    fn	Parse< G: IGrammar>( &mut self, grammar: &G) -> bool
    {
        grammar.Match( self)
    }

    fn	InStream( &mut self) -> &mut InStream<'s, R>
    {
        self._InStream
    }

    fn	Forge<'f>( &self) -> Option<&'f (dyn IForge<'f, 'p, 's, R> + 'f)> where Self: 'f
    {
        let  	sz = self._Stash.Size();
        if sz > U32( 0) {
            let  	lastIdx = sz.0 - 1;
            let  	arr = self._Stash.Stk().Arr();
            let  	optPtr = arr.At( lastIdx);
            if let Some( ptr) = *optPtr {
                let  	ptrF = ptr.CastLife::<dyn IForge<'f, 'p, 's, R> + 'f>();
                return Some( unsafe { &*ptrF });
            }
            None
        } else {
            None
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

pub struct CharsetForge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: *mut Parser<'p, 's, R>,
    pub _Shard: Option<&'a Shard>,
}

impl<'a, 'p, 's, R: Read + 'p> Default for CharsetForge<'a, 'p, 's, R>
where 's: 'p
{
    fn default() -> Self {
        Self {
            _Parent: None,
            _Parser: std::ptr::null_mut(),
            _Shard: None,
        }
    }
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for CharsetForge<'a, 'p, 's, R>
where 's: 'p
{
    fn Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)> { self._Parent }
    fn Parser( &mut self) -> &mut Parser<'p, 's, R> { unsafe { &mut *self._Parser } }
    fn Deposit( &self, _childIdx: U32, _digest: Digest) {}

    fn MatchNode(&mut self) -> bool {
        let startMark = self.Parser().InStream().Marker();
        if let Some(shard) = self._Shard {
            if shard.Match(self.Parser()) {
                let endMark = self.Parser().InStream().Marker();
                let digest = Digest { _Start: startMark, _End: endMark };
                if let Some(p) = self.Parent() {
                    p.Deposit(crate::silo::U32(0), digest);
                }
                return true;
            }
        }
        false
    }
}

pub struct StringForge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: *mut Parser<'p, 's, R>,
    pub _Shard: Option<&'a Shard>,
}

impl<'a, 'p, 's, R: Read + 'p> Default for StringForge<'a, 'p, 's, R>
where 's: 'p
{
    fn default() -> Self {
        Self {
            _Parent: None,
            _Parser: std::ptr::null_mut(),
            _Shard: None,
        }
    }
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for StringForge<'a, 'p, 's, R>
where 's: 'p
{
    fn Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)> { self._Parent }
    fn Parser( &mut self) -> &mut Parser<'p, 's, R> { unsafe { &mut *self._Parser } }
    fn Deposit( &self, _childIdx: U32, _digest: Digest) {}

    fn MatchNode(&mut self) -> bool {
        let startMark = self.Parser().InStream().Marker();
        if let Some(shard) = self._Shard {
            if shard.Match(self.Parser()) {
                let endMark = self.Parser().InStream().Marker();
                let digest = Digest { _Start: startMark, _End: endMark };
                if let Some(p) = self.Parent() {
                    p.Deposit(crate::silo::U32(0), digest);
                }
                return true;
            }
        }
        false
    }
}



pub struct CatArrForge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: *mut Parser<'p, 's, R>,
    pub _Children: crate::silo::Buff<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>,
}

impl<'a, 'p, 's, R: Read + 'p> CatArrForge<'a, 'p, 's, R>
where 's: 'p
{
    pub fn new(parser: *mut Parser<'p, 's, R>, children: crate::silo::Buff<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>) -> Self {
        Self {
            _Parent: None,
            _Parser: parser,
            _Children: children,
        }
    }
}


impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for CatArrForge<'a, 'p, 's, R>
where 's: 'p
{
    fn Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)> { self._Parent }
    fn Parser( &mut self) -> &mut Parser<'p, 's, R> { unsafe { &mut *self._Parser } }
    fn Deposit( &self, _childIdx: U32, _digest: Digest) {}

    fn MatchNode(&mut self) -> bool {
        let startMark = self.Parser().InStream().Marker();
        for i in 0..self._Children.Size().AsUsize() {
            let child_ptr = self._Children[i];
            let child_ref = unsafe { &mut *child_ptr };
            if !child_ref.MatchNode() {
                self.Parser().InStream().RollTo(startMark);
                return false;
            }
        }
        let endMark = self.Parser().InStream().Marker();
        let digest = Digest { _Start: startMark, _End: endMark };
        if let Some(p) = self.Parent() {
            p.Deposit(crate::silo::U32(0), digest);
        }
        true
    }
}




pub struct ParArrForge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Parser: *mut Parser<'p, 's, R>,
    pub _Children: crate::silo::Buff<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>,
}

impl<'a, 'p, 's, R: Read + 'p> ParArrForge<'a, 'p, 's, R>
where 's: 'p
{
    pub fn new(parser: *mut Parser<'p, 's, R>, children: crate::silo::Buff<*mut (dyn IForge<'p, 'p, 's, R> + 'p)>) -> Self {
        Self {
            _Parent: None,
            _Parser: parser,
            _Children: children,
        }
    }
}



impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for ParArrForge<'a, 'p, 's, R>
where 's: 'p
{
    fn Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)> { self._Parent }
    fn Parser( &mut self) -> &mut Parser<'p, 's, R> { unsafe { &mut *self._Parser } }
    fn Deposit( &self, _childIdx: U32, _digest: Digest) {}

    fn MatchNode(&mut self) -> bool {
        let startMark = self.Parser().InStream().Marker();
        for i in 0..self._Children.Size().AsUsize() {
            let child_ptr = self._Children[i];
            let child_ref = unsafe { &mut *child_ptr };
            if child_ref.MatchNode() {
                let endMark = self.Parser().InStream().Marker();
                let digest = Digest { _Start: startMark, _End: endMark };
                if let Some(p) = self.Parent() {
                    p.Deposit(crate::silo::U32(0), digest);
                }
                return true;
            }
            self.Parser().InStream().RollTo(startMark);
        }
        false
    }
}



//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> IGrammar for DynINode<'a>
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let root_forge_ptr = parser.ParseShardTree(self);
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
        let  	res = ( *self).Match( parser);
        return res;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
