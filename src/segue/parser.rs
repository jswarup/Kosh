//-- parser.rs -------------------------------------------------------------------------------------------------------------------

use	std::io;
use	std::io::Read;
use	std::cell::Cell;
use	crate::{
    flux::InStream,
    segue::Charset
};
use	crate::silo::{ U8, U32, Stash, IAccess, IArr };
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

pub trait IForge<'a, 'p, 's, R: Read + 'p>
    where 's: 'p, Self: 'a
{
    fn	Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>;
    fn	Offset( &self) -> U32;
    fn	GetParser( &mut self) -> &mut Parser<'p, 's, R>;
    fn	Deposit( &self, childIdx: U32, digest: Digest);
    
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
        let  	erased_ptr = unsafe { std::mem::transmute::<_, *mut (dyn IForge<'static, 'static, 'static, R> + 'static)>(ptr) };
        self.GetParser()._Stash.Push( Some(erased_ptr));
    }
    
    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Finish( &mut self) 
        where Self: Sized 
    {
        let  	mut dummy = None;
        self.GetParser()._Stash.Pop( &mut dummy);
    }
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
    pub _Offset: U32,
    pub _Parser: &'a mut Parser<'p, 's, R>,
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for Forge<'a, 'p, 's, R>
    where 's: 'p
{
    fn	Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>
    {
        self._Parent
    }

    fn	Offset( &self) -> U32
    {
        self._Offset
    }

    fn	GetParser( &mut self) -> &mut Parser<'p, 's, R>
    {
        self._Parser
    }

    fn	Deposit( &self, _childIdx: U32, _digest: Digest)
    {
    }
}

impl<'a, 'p, 's, R: Read + 'p> Drop for Forge<'a, 'p, 's, R>
where 's: 'p
{
    fn	drop( &mut self)
    {
        self.Finish();
    }
}
//---------------------------------------------------------------------------------------------------------------------------------

pub trait IParser<'p, 's, R: Read + 'p>
where 's: 'p
{
    fn	Parse< G: IGrammar>( &mut self, grammar: &G) -> bool;
    fn	Stream( &mut self) -> &mut InStream<'s, R>;
    fn	Forge<'f>( &self) -> Option<&'f (dyn IForge<'f, 'p, 's, R> + 'f)> where Self: 'f;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Parser<'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Stream: &'p mut InStream<'s, R>,
    // Erase the 'f lifetime by transmuting to 'static to break the cyclic dropck dependency
    pub _Stash: Stash< Option<*mut (dyn IForge<'static, 'static, 'static, R> + 'static)>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'p, 's, R: Read + 'p> Parser<'p, 's, R>
where 's: 'p
{
    pub fn	New( stream: &'p mut InStream<'s, R>) -> Self
    {
        Self {
            _Stream: stream,
            _Stash: Stash::New( U32( 16), U32( 0), None),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DepositArr( &mut self, _arr: crate::silo::Arr<'_, Option<*const DynINode<'static>>>, _isConcat: bool) -> Option<*const DynINode<'static>>
    {
        None
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ParseShardTree( &mut self, node: &DynINode< '_>) -> Option<*const DynINode<'static>>
    {
        let  	nodeStash = Stash::<Option<*const DynINode<'static>>>::New( 1024, 0, None);
        let  	mut nodeStk = nodeStash.Stk();
        let  	opStash = Stash::<(ChildOp, U32)>::New( 1024, 0, (ChildOp::None, 0.into()));
        let  	opStk = opStash.Stk();

        node.DiveDf( &mut |probe, enterFlg| {
            let  	curNode = probe.CurNode().unwrap();
            let  	curOp = curNode.ChildOp();
            if enterFlg {
                if curOp != ChildOp::None {
                    opStk.Push( ( curOp, nodeStk.Size()));
                    return;
                } 
                let  	nodePtr = unsafe { std::mem::transmute::<*const DynINode<'_>, *const DynINode<'static>>(curNode as *const DynINode<'_>) };
                nodeStk.Push( Some( nodePtr));
                return;
            }
            if curOp == ChildOp::None { 
                return; 
            }
            let  	mut opCtx = ( ChildOp::None, 0.into());
            opStk.Pop( &mut opCtx); 

            let  	parentOp = if opStk.Size() != 0 { opStk.Arr().Last().0 } else { ChildOp::None };
            if parentOp == curOp { 
                return; 
            }

            let  	arr = nodeStk.Arr().Subset( opCtx.1, nodeStk.Size() - opCtx.1);
            nodeStk.SetSize( opCtx.1);
            let  	isConcat = curOp == ChildOp::Less;
            let  	newNode = self.DepositArr( arr, isConcat);
            nodeStk.Push( newNode);
        }); 
        
        if nodeStk.Size() == 0 {
            None
        } else {
            *nodeStk.Arr().Last()
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

    fn	Stream( &mut self) -> &mut InStream<'s, R>
    {
        self._Stream
    }

    fn	Forge<'f>( &self) -> Option<&'f (dyn IForge<'f, 'p, 's, R> + 'f)> where Self: 'f
    {
        let  	sz = self._Stash.Size();
        if sz > U32( 0) {
            let  	lastIdx = sz.0 - 1;
            let  	arr = self._Stash.Stk().Arr();
            let  	optPtr = arr.At( lastIdx);
            if let Some( ptr) = *optPtr {
                let  	ptrF = unsafe {
                    std::mem::transmute::<
                        *mut (dyn IForge<'static, 'static, 'static, R> + 'static),
                        *mut (dyn IForge<'f, 'p, 's, R> + 'f)
                    >( ptr)
                };
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
        let  	curr = parser.Stream().Curr();
        if self.Get( curr) {
            parser.Stream().Next();
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
        let  	curr = parser.Stream().Curr();
        if curr == U8( *self as u8) {
            parser.Stream().Next();
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
        let  	startMark = parser.Stream().Marker();
        
        // Ensure that empty string matches without consuming
        if self.is_empty() {
            return true;
        }

        for c in self.chars() {
            let  	curr = parser.Stream().Curr();
            if curr == U8( c as u8) {
                // If it's the last char, we just advance and we're good.
                let _ = parser.Stream().Next();
            } else {
                parser.Stream().RollTo( startMark);
                return false;
            }
        }
        
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct  BinOpForge<'a, 'p, 's, R: Read + 'p = io::Empty>
where 's: 'p
{
    pub _Parent: Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>,
    pub _Offset: U32,
    pub _Parser: &'a mut Parser<'p, 's, R>,
    pub _Node: Option<*const DynINode<'static>>,
    pub _LeftDigest: Cell< Option< Digest>>,
    pub _RightDigest: Cell< Option< Digest>>,
}

impl<'a, 'p, 's, R: Read + 'p> IForge<'a, 'p, 's, R> for BinOpForge<'a, 'p, 's, R>
where 's: 'p
{
    fn	Parent( &self) -> Option< &'a (dyn IForge<'a, 'p, 's, R> + 'a)>
    {
        self._Parent
    }

    fn	Offset( &self) -> U32
    {
        self._Offset
    }

    fn	GetParser( &mut self) -> &mut Parser<'p, 's, R>
    {
        self._Parser
    }

    fn	Deposit( &self, childIdx: U32, digest: Digest)
    {
        if childIdx == U32( 0) {
            self._LeftDigest.set( Some( digest));
        } else {
            self._RightDigest.set( Some( digest));
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'p, 's, R: Read + 'p> Drop for BinOpForge<'a, 'p, 's, R>
where 's: 'p
{
    fn	drop( &mut self)
    {
        self.Finish();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> IGrammar for DynINode<'a>
{
    fn	Match< 'p, 's, R: Read>( &self, parser: &mut Parser<'p, 's, R>) -> bool
    {
        let  	startMark = parser.Stream().Marker();
        let  	parent = parser.Forge();
        
        match self.ChildOp() {
            ChildOp::None => {
                if let Some( anyRef) = self.AsAny() {
                    if let Some( shard) = anyRef.downcast_ref::< Shard>() {
                        let  	res = shard.Match( parser);
                        return res;
                    }
                }
                false
            }
            ChildOp::Less | ChildOp::Bor => {
                let  	node_ptr = unsafe { std::mem::transmute::<*const DynINode<'a>, *const DynINode<'static>>(self as *const DynINode<'a>) };
                let  	mut forge = BinOpForge {
                    _Parent: parent,
                    _Offset: startMark,
                    _Parser: parser,
                    _Node: Some(node_ptr),
                    _LeftDigest: Cell::new( None),
                    _RightDigest: Cell::new( None),
                };
                forge.Init();
                return forge.MatchNode();
            }
            _ => {
                false
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a, 'p, 's, R: Read + 'p> BinOpForge<'a, 'p, 's, R>
where 's: 'p
{
    pub fn MatchNode( &mut self) -> bool
    {
        let  	node = unsafe { &*(self._Node.unwrap() as *const DynINode<'_>) };
        
        match node.ChildOp() {
            ChildOp::Less => {
                let  	leftChild = node._At( U32( 0));
                let  	rightChild = node._At( U32( 1));
                
                if leftChild.Match( self.GetParser()) {
                    if rightChild.Match( self.GetParser()) {
                        let  	endMark = self.GetParser().Stream().Marker();
                        let  	digest = Digest { _Start: self._Offset, _End: endMark };
                        if let Some( p) = self.Parent() {
                            p.Deposit( U32( 0), digest);
                        }
                        return true;
                    }
                }
                
                let  	startMark = self._Offset;
                self.GetParser().Stream().RollTo( startMark);
                false
            }
            ChildOp::Bor => {
                let  	leftChild = node._At( U32( 0));
                let  	rightChild = node._At( U32( 1));
                
                if leftChild.Match( self.GetParser()) {
                    let  	endMark = self.GetParser().Stream().Marker();
                    let  	digest = Digest { _Start: self._Offset, _End: endMark };
                    if let Some( p) = self.Parent() {
                        p.Deposit( U32( 0), digest);
                    }
                    return true;
                }
                
                let  	startMark = self._Offset;
                self.GetParser().Stream().RollTo( startMark);
                
                if rightChild.Match( self.GetParser()) {
                    let  	endMark = self.GetParser().Stream().Marker();
                    let  	digest = Digest { _Start: self._Offset, _End: endMark };
                    if let Some( p) = self.Parent() {
                        p.Deposit( U32( 0), digest);
                    }
                    return true;
                }
                
                let  	startMark = self._Offset;
                self.GetParser().Stream().RollTo( startMark);
                false
            }
            _ => false
        }
    }
}

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
