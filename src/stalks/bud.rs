//-- bud.rs -------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, U32 };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudBinOp {
    LT,
    BOR,
    SHL,
    SHR,
}
impl BudBinOp
{
    pub fn	as_str( &self) -> &'static str {
        match self {
            BudBinOp::LT => "<",
            BudBinOp::BOR => "|",
            BudBinOp::SHL => "<<",
            BudBinOp::SHR => ">>",
        }
    }
}
#[derive( Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudUniOp {
    STAR,
    PLUS,
    MINUS,
    BANG,
}
impl BudUniOp
{
    pub fn	as_str( &self) -> &'static str {
        match self {
            BudUniOp::STAR => "*",
            BudUniOp::PLUS => "+",
            BudUniOp::MINUS => "-",
            BudUniOp::BANG => "!",
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait Bud< T> {
    fn	Val( &self) -> Option< &T>;
    fn	ValMut( &mut self) -> Option< &mut T>
    {
        None
    }

    fn	Left( &self) -> Option< &dyn Bud< T>>
    {
        None
    }
    fn	LeftMut( &mut self) -> Option< &mut dyn Bud< T>>
    {
        None
    }
    fn	Right( &self) -> Option< &dyn Bud< T>>
    {
        None
    }
    fn	RightMut( &mut self) -> Option< &mut dyn Bud< T>>
    {
        None
    }
    fn	BinOp( &self) -> Option< BudBinOp>
    {
        None
    }
    fn	UniOp( &self) -> Option< BudUniOp>
    {
        None
    }

}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Debug, PartialEq, Eq)]
pub enum TraversalEvent {
    Entry,
    Exit,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> dyn Bud< T> + '_
{
    pub fn	TraverseDFS( &self, f: &mut dyn FnMut( &dyn Bud< T>, TraversalEvent))
    {
        let  	isLeaf = self.Left().is_none() && self.Right().is_none();
        f( self, TraversalEvent::Entry);
        if let  	Some( left) = self.Left() {
            left.TraverseDFS( f);
        }
        if let  	Some( right) = self.Right() {
            right.TraverseDFS( f);
        }
        if !isLeaf {
            f( self, TraversalEvent::Exit);
        }
    }
}

impl< T > dyn Bud< T > + '_
where
    T: std::fmt::Display,
{
    pub fn	Print( &self)
    {
        let  	mut childCounts = Buff::<U32>::NewEmpty();
        self.TraverseDFS( &mut |node, event| match event {
            TraversalEvent::Entry => {
                if let  	Some( count) = childCounts.last_mut() {
                    if *count > 0 {
                        print!( " ");
                    }
                    *count += 1;
                }
                let isLeaf = node.Left().is_none() && node.Right().is_none();
                if !isLeaf {
                    let op_str = if let Some(op) = node.BinOp() {
                        op.as_str()
                    } else if let Some(op) = node.UniOp() {
                        op.as_str()
                    } else {
                        ""
                    };
                    print!( "[{} ", op_str);
                    childCounts.Push( 0.into());
                } else {
                    if let Some(val) = node.Val() {
                        print!( "{}", val);
                    }
                }
            }
            TraversalEvent::Exit => {
                childCounts.Pop();
                print!( "]");
            }
        });
        println!();
    }
}

impl< T > dyn Bud< T > + '_
where
    T: crate::stalks::IWork + Clone + Default + 'static,
{
    pub fn	Post( &mut self, worker: &dyn crate::stalks::IWorker)
    {
        if worker.IsSequential() {
            fn	RunSequential< T>( node: &mut dyn Bud< T>, worker: &dyn crate::stalks::IWorker)
            where
                T: crate::stalks::IWork + Clone + Default + 'static,
            {
                if node.Left().is_none() && node.Right().is_none() {
                    if let Some( val) = node.ValMut() {
                        val.DoWork( worker);
                    }
                    return;
                }
                if let Some( op) = node.BinOp() {
                    if op == BudBinOp::BOR || op == BudBinOp::LT {
                        if let Some( left) = node.LeftMut() {
                            RunSequential( left, worker);
                        }
                        if let Some( right) = node.RightMut() {
                            RunSequential( right, worker);
                        }
                    }
                }
                if let Some( _op) = node.UniOp() {
                    if let Some( left) = node.LeftMut() {
                        RunSequential( left, worker);
                    }
                }
            }
            RunSequential( self, worker);
            return;
        }
        let  	maestro = crate::heist::Maestro::FromWorker( worker);
        struct JobStash
        {
            _JobStash: crate::silo::Stash< crate::silo::U16 >,
        }
        impl JobStash
        {
            fn	Process< T>( &mut self, node: &dyn Bud< T>, maestro: &crate::heist::Maestro< '_>, succId: crate::silo::U16) -> crate::silo::U16
            where
                T: crate::stalks::IWork + Clone + Default + 'static,
            {
                if node.Left().is_none() && node.Right().is_none() {
                    if let Some( val) = node.Val() {
                        let mut jobId = maestro.ConstructJob( succId, val.clone());
                        self._JobStash.Pushback( &mut jobId);
                        return jobId;
                    }
                    return succId;
                }
                if let Some( op) = node.BinOp() {
                    if op == BudBinOp::BOR {
                        let  _succR = self.Process( node.Right().unwrap(), maestro, succId);
                        let  _succL = self.Process( node.Left().unwrap(), maestro, succId);
                        return succId;
                    }
                    if op == BudBinOp::LT {
                        let  mark = self._JobStash.Size();
                        let  rJobId = self.Process( node.Right().unwrap(), maestro, succId);
                        let  jStk = self._JobStash.Stk();
                        let  rSz = jStk.Size() - mark;
                        if rSz == 1 {
                            return self.Process( node.Left().unwrap(), maestro, rJobId);
                        }
                        let  mut rXStash = crate::silo::Stash::< crate::silo::U16 >::New( rSz);
                        self._JobStash.Stk().Export( &rXStash.Stk(), rSz);
                        let  branchId = maestro.ConstructEnqueueBulk( succId, rXStash.BuffOut());
                        let  succL = self.Process( node.Left().unwrap(), maestro, branchId);
                        return succL;
                    }
                }
                if let Some( _op) = node.UniOp() {
                    return self.Process( node.Left().unwrap(), maestro, succId);
                }
                assert!( false);
                return crate::silo::U16( 0);
            }
        }
        let  	succId = maestro.CurSuccId();
        let  	mut jobStash = JobStash {
            _JobStash: crate::silo::Stash::New( 0),
        };
        jobStash.Process( &*self, maestro, succId);
        let  	jobArr = jobStash._JobStash.Stk().Arr();
        jobArr.USeg().Traverse( |i| {
            maestro.EnqueueJob( jobArr.MutAt( i));
        });
        return;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IntoBud< T, N: Sized > {
    fn	IntoBud( self ) -> N;
    fn	IntoBudAction( self, _act: N ) -> N
    where
        Self: Sized,
    {
        self.IntoBud()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! BudTree {
    // ---- FEATURE OPT-INS FOR BudTree ITSELF ----------------------------------------------------------------------------
    // BudTree explicitly opts in to all features by delegating back to its own builders.
    ( @feature_SHL [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $( $cb)* ], $Arg, $Node, SHL, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHL [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $( $cb)* ], $Arg, $Node, SHL, $l, $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $( $cb)* ], $Arg, $Node, SHR, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $( $cb)* ], $Arg, $Node, SHR, $l, $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $( $cb)* ], $Arg, $Node, LT, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT  [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $( $cb)* ], $Arg, $Node, LT, $l, $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bg $Arg:ident, $Node:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $( $cb)* ], $Arg, $Node, BOR, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $( $cb:tt)* ], @bl $Arg:ident, $Node:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $( $cb)* ], $Arg, $Node, BOR, $l, $( $r)+ ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $Node::New( $Arg::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $Node::New( $Arg::New( || $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $Node::New( $Arg::New( move | $( $body)+ ) ) };
    ( @feature_NEW [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $Node::New( $Arg::New( move || $( $body)+ ) ) };

    ( @feature_STAR  [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $( $cb)* ], $Arg, $Node, STAR, $l $( $r )* ) };
    ( @feature_PLUS  [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $( $cb)* ], $Arg, $Node, PLUS, $l $( $r )* ) };
    ( @feature_MINUS [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $( $cb)* ], $Arg, $Node, MINUS, $l $( $r )* ) };
    ( @feature_BANG  [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $( $cb)* ], $Arg, $Node, BANG, $l $( $r )* ) };

    ( @wrap $leaf:expr ) => {
        Into::into( $leaf )
    };
    ( @define [ $( $cb:tt )* ], $Arg:ident, $( $inner:tt )+ ) => {
        {
            paste::paste! {
                #[derive( Debug, PartialEq, Clone)]
                enum [<$Arg Bud>] {
                    Leaf( $Arg),
                    Node {
                        _BinOp: $crate::stalks::bud::BudBinOp,
                        _Left: Box< [<$Arg Bud>]>,
                        _Right: Box< [<$Arg Bud>]>,
                    },
                    UniNode {
                        _UniOp: $crate::stalks::bud::BudUniOp,
                        _Child: Box< [<$Arg Bud>]>,
                    }
                }
                impl [<$Arg Bud>]
                {
                    fn	New( value: $Arg) -> Self
                    {
                        [<$Arg Bud>]::Leaf( value)
                    }
                    fn	NewBranch( op: $crate::stalks::bud::BudBinOp, left: Self, right: Self) -> Self
                    {
                        [<$Arg Bud>]::Node {
                            _BinOp: op,
                            _Left: Box::new( left),
                            _Right: Box::new( right),
                        }
                    }
                    fn	NewUni( op: $crate::stalks::bud::BudUniOp, child: Self) -> Self
                    {
                        [<$Arg Bud>]::UniNode {
                            _UniOp: op,
                            _Child: Box::new( child),
                        }
                    }
                    pub fn	CountLeaves( &self) -> usize
                    {
                        match self {
                            [<$Arg Bud>]::Leaf( _) => 1,
                            [<$Arg Bud>]::Node { _Left, _Right, .. } => _Left.CountLeaves() + _Right.CountLeaves(),
                            [<$Arg Bud>]::UniNode { _Child, .. } => _Child.CountLeaves(),
                        }
                    }
                }

                impl $crate::stalks::bud::Bud< $Arg> for [<$Arg Bud>]
                {
                    fn	Val( &self) -> Option< &$Arg>
                    {
                        match self {
                            [<$Arg Bud>]::Leaf( value) => Some( value),
                            _ => None,
                        }
                    }
                    fn	ValMut( &mut self) -> Option< &mut $Arg>
                    {
                        match self {
                            [<$Arg Bud>]::Leaf( value) => Some( value),
                            _ => None,
                        }
                    }
                    fn	Left( &self) -> Option< &dyn $crate::stalks::bud::Bud< $Arg>>
                    {
                        match self {
                            [<$Arg Bud>]::Node { _Left, .. } => Some( &**_Left),
                            [<$Arg Bud>]::UniNode { _Child, .. } => Some( &**_Child),
                            _ => None,
                        }
                    }
                    fn	LeftMut( &mut self) -> Option< &mut dyn $crate::stalks::bud::Bud< $Arg>>
                    {
                        match self {
                            [<$Arg Bud>]::Node { _Left, .. } => Some( &mut **_Left),
                            [<$Arg Bud>]::UniNode { _Child, .. } => Some( &mut **_Child),
                            _ => None,
                        }
                    }
                    fn	Right( &self) -> Option< &dyn $crate::stalks::bud::Bud< $Arg>>
                    {
                        match self {
                            [<$Arg Bud>]::Node { _Right, .. } => Some( &**_Right),
                            _ => None,
                        }
                    }
                    fn	RightMut( &mut self) -> Option< &mut dyn $crate::stalks::bud::Bud< $Arg>>
                    {
                        match self {
                            [<$Arg Bud>]::Node { _Right, .. } => Some( &mut **_Right),
                            _ => None,
                        }
                    }
                    fn	BinOp( &self) -> Option< $crate::stalks::bud::BudBinOp>
                    {
                        match self {
                            [<$Arg Bud>]::Node { _BinOp, .. } => Some( *_BinOp),
                            _ => None,
                        }
                    }
                    fn	UniOp( &self) -> Option< $crate::stalks::bud::BudUniOp>
                    {
                        match self {
                            [<$Arg Bud>]::UniNode { _UniOp, .. } => Some( *_UniOp),
                            _ => None,
                        }
                    }
                }
                impl std::ops::Deref for [<$Arg Bud>] {
                    type Target = dyn $crate::stalks::bud::Bud< $Arg >;
                    fn deref( &self) -> &Self::Target {
                        self
                    }
                }
                impl std::ops::DerefMut for [<$Arg Bud>] {
                    fn deref_mut( &mut self) -> &mut Self::Target {
                        self
                    }
                }
                impl< I > $crate::stalks::bud::IntoBud< $Arg, [<$Arg Bud>]> for I
                where
                    I: Into< $Arg >,
                {
                    fn	IntoBud( self) -> [<$Arg Bud>] {
                        [<$Arg Bud>]::New( self.into() )
                    }
                }
                impl $crate::stalks::bud::IntoBud< $Arg, [<$Arg Bud>]> for [<$Arg Bud>] {
                    fn	IntoBud( self) -> [<$Arg Bud>] {
                        self
                    }
                }
                $crate::BudTree!( @cb [ $( $cb)* ], $Arg, [<$Arg Bud>], $( $inner )+ )
            }
        }
    };
    ( $Arg:ident, $( $inner:tt )+ ) => {
        $crate::BudTree!( @define [ $crate::BudTree ], $Arg, $( $inner )+ )
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $inner:tt)+ ) ) => { $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $inner)+ ) };

    // ── Unary operators ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, * $l:tt $( $r:tt)* ) => { $( $cb)* !( @feature_STAR [ $( $cb)* ], $Arg, $Node, $l $( $r )* ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, + $l:tt $( $r:tt)* ) => { $( $cb)* !( @feature_PLUS [ $( $cb)* ], $Arg, $Node, $l $( $r )* ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, - $l:tt $( $r:tt)* ) => { $( $cb)* !( @feature_MINUS [ $( $cb)* ], $Arg, $Node, $l $( $r )* ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ! $l:tt $( $r:tt)* ) => { $( $cb)* !( @feature_BANG [ $( $cb)* ], $Arg, $Node, $l $( $r )* ) };
    
    // ── Binary: (group) OP rhs ──────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, ( $( $l)+ ), $( $r)+ ) };
    
    // ── Binary: ident/literal OP rhs ────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:ident |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bl $Arg, $Node, $l, $( $r)+ ) };
    
    // ── Closure literal ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, || $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move | $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, move | $( $body)+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, move || $( $body:tt)+ ) => { $( $cb)* !( @feature_NEW [ $( $cb)* ], $Arg, $Node, move || $( $body)+ ) };

    // ── Leaf [ action ] ────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal [ $( $inner:tt )* ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg, $Node, $l [ $( $inner )* ] )
    };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $expr:tt)+ ) [ $( $inner:tt )* ] ) => {
        $( $cb)* !( @feature_ACTION [ $( $cb)* ], $Arg, $Node, ( $( $expr )+ ) [ $( $inner )* ] )
    };
    
    // ── Binary: [ boxet ] OP rhs ────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] << $( $r:tt)+ ) => { $( $cb)* !( @feature_SHL [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] >> $( $r:tt)+ ) => { $( $cb)* !( @feature_SHR [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] <  $( $r:tt)+ ) => { $( $cb)* !( @feature_LT  [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] |  $( $r:tt)+ ) => { $( $cb)* !( @feature_BOR [ $( $cb)* ], @bg $Arg, $Node, ( $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s ) ), $( $r )+ ) };
    
    // ── Leaf Boxet ──────────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, [ $s:literal ] ) => {
        $( $cb)* !( @feature_BOXET [ $( $cb)* ], $Arg, $Node, $s )
    };

    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $leaf:expr ) => {
        $crate::stalks::bud::IntoBud::< $Arg, $Node >::IntoBud( $leaf )
    };

    // @uni : unary - OP rhs
    ( @uni [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, $l:tt $( $r:tt)* ) => {
        $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, ( $Node::NewUni( $crate::stalks::bud::BudUniOp::$op, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $l ) ) ) $( $r)* )
    };

    // @feature_ACTION : matches literal/group followed by action brackets, parses the closure
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $l:literal [ $( $closure:tt )* ] ) => {
        $( $cb)* !( @closure_match [ $( $cb)* ], $Arg, $Node, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $l ), $( $closure )* )
    };
    ( @feature_ACTION [ $( $cb:tt)* ], $Arg:ident, $Node:ident, ( $( $expr:tt)+ ) [ $( $closure:tt )* ] ) => {
        $( $cb)* !( @closure_match [ $( $cb)* ], $Arg, $Node, $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, ( $( $expr )+ ) ), $( $closure )* )
    };

    // @closure_match : resolves closure vs non-closure for action leaf
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $base:expr, | $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::< $Arg, $Node >::IntoBudAction( $base, $Node::New( $Arg::New( | $( $closure)* ) ) )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $base:expr, || $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::< $Arg, $Node >::IntoBudAction( $base, $Node::New( $Arg::New( || $( $closure)* ) ) )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $base:expr, move | $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::< $Arg, $Node >::IntoBudAction( $base, $Node::New( $Arg::New( move | $( $closure)* ) ) )
    };
    ( @closure_match [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $base:expr, move || $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::< $Arg, $Node >::IntoBudAction( $base, $Node::New( $Arg::New( move || $( $closure)* ) ) )
    };

    // ---- Internal helpers ----------------------------------------------------------------------------------------------------
    // @bg : binary — (group) OP rhs
    ( @bg [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        $Node::NewBranch( 
            $crate::stalks::bud::BudBinOp::$op,
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $l)+ ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) )
    };
    // @bl : binary — leaf OP rhs
    ( @bl [ $( $cb:tt)* ], $Arg:ident, $Node:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        $Node::NewBranch( 
            $crate::stalks::bud::BudBinOp::$op,
            $crate::stalks::bud::IntoBud::< $Arg, $Node >::IntoBud( $l ),
            $( $cb)* !( @cb [ $( $cb)* ], $Arg, $Node, $( $r)+ ) )
    };

    // ---- DEFAULT FALLBACK ERRORS FOR DISABLED FEATURES -------------------------------------------------------------
    ( @feature_SHL $( $args:tt )* ) => { compile_error!( "Binary SHL (<<) is not enabled for this tree") };
    ( @feature_SHR $( $args:tt )* ) => { compile_error!( "Binary SHR (>>) is not enabled for this tree") };
    ( @feature_LT  $( $args:tt )* ) => { compile_error!( "Binary LT (<) is not enabled for this tree") };
    ( @feature_BOR $( $args:tt )* ) => { compile_error!( "Binary BOR (|) is not enabled for this tree") };
    ( @feature_NEW $( $args:tt )* ) => { compile_error!( "Closure literal is not enabled for this tree") };
    ( @feature_STAR  $( $args:tt )* ) => { compile_error!( "Unary STAR (*) is not enabled for this tree") };
    ( @feature_PLUS  $( $args:tt )* ) => { compile_error!( "Unary PLUS (+) is not enabled for this tree") };
    ( @feature_MINUS $( $args:tt )* ) => { compile_error!( "Unary MINUS (-) is not enabled for this tree") };
    ( @feature_BANG  $( $args:tt )* ) => { compile_error!( "Unary BANG (!) is not enabled for this tree") };
    ( @feature_ACTION $( $args:tt )* ) => { compile_error!( "Action suffix [ closure ] is not enabled for this tree") };
    ( @feature_BOXET  $( $args:tt )* ) => { compile_error!( "Boxet [ ... ] is not enabled for this tree") };
}
pub use	crate::BudTree;

//---------------------------------------------------------------------------------------------------------------------------------
