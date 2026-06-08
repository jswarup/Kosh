//-- bud.rs -------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

pub trait Bud< T> {
    fn	Val( &self) -> T;
    fn	Left( &self) -> Option< &dyn Bud< T>>
    {
        None
    }
    fn	Right( &self) -> Option< &dyn Bud< T>>
    {
        None
    }
    fn	Op( &self) -> &str
    {
        ""
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

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: std::fmt::Display> dyn Bud< T> + '_
{
    pub fn	Print( &self)
    {
        let  	mut childCounts = Vec::new();
        self.TraverseDFS( &mut |node, event| match event {
            TraversalEvent::Entry => {
                if let  	Some( count) = childCounts.last_mut() {
                    if *count > 0 {
                        print!( " ");
                    }
                    *count += 1;
                }
                if node.Left().is_some() || node.Right().is_some() {
                    print!( "[{} ", node.Op());
                    childCounts.push( 0);
                } else {
                    print!( "{}", node.Val());
                }
            }
            TraversalEvent::Exit => {
                childCounts.pop();
                print!( "]");
            }
        });
        println!();
    }
}

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
    pub fn	as_str( &self) -> &'static str
    {
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
    pub fn	as_str( &self) -> &'static str
    {
        match self {
            BudUniOp::STAR => "*",
            BudUniOp::PLUS => "+",
            BudUniOp::MINUS => "-",
            BudUniOp::BANG => "!",
        }
    }
}
//---------------------------------------------------------------------------------------------------------------------------------

pub enum BudType< T> {
    Val( T),
    Bin( BudBinOp, Box< dyn Bud< T>>, Box< dyn Bud< T>>),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BudNode< T>
{
    _Type: BudType< T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: BudOp> BudNode< T>
{
    pub fn	Create( op: BudBinOp, left: Box< dyn Bud< T>>, right: Box< dyn Bud< T>>) -> Self
    {
        assert!(
            T::IsOpAllowed( op),
            "Binary operation not supported for this type"
        );
        Self {
            _Type: BudType::Bin( op, left, right),
        }
    }
}
impl< T> BudNode< T>
{
    pub fn	NewVal( id: T) -> Self
    {
        Self {
            _Type: BudType::Val( id),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Bud< T> for BudNode< T>
where
    T: Clone + Default,
{
    fn	Val( &self) -> T
    {
        match &self._Type {
            BudType::Val( val) => val.clone(),
            _ => T::default(),
        }
    }
    fn	Left( &self) -> Option< &dyn Bud< T>>
    {
        match &self._Type {
            BudType::Val( _) => None,
            BudType::Bin( _, left, _) => Some( &**left),
        }
    }
    fn	Right( &self) -> Option< &dyn Bud< T>>
    {
        match &self._Type {
            BudType::Val( _) => None,
            BudType::Bin( _, _, right) => Some( &**right),
        }
    }
    fn	Op( &self) -> &str
    {
        match &self._Type {
            BudType::Val( _) => "",
            BudType::Bin( op, _, _) => op.as_str(),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IntoBud< T>: Sized {
    fn	IntoBud( self) -> Box< dyn Bud< T>>;

    fn	IntoBudBox( self) -> Box< dyn Bud< T>>
    {
        self.IntoBud()
    }

    fn	IntoBudAction( self, _atm: Box< dyn Bud< T>>) -> Box< dyn Bud< T>>
    {
        self.IntoBud()
    }
    fn	IntoBudUniOp( self, _op: BudUniOp) ->  Box< dyn Bud< T>>
    {
        self.IntoBud()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> IntoBud< T> for Box< dyn Bud< T>>
    {
    fn	IntoBud( self) -> Box< dyn Bud< T>>
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Bud< T> + 'static> IntoBud< T> for T
{
    fn	IntoBud( self) -> Box< dyn Bud< T>>
    {
        Box::new( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait BudOp: Clone + Default + 'static {
    fn	IsOpAllowed( _op: BudBinOp) -> bool
    {
        true
    }
    fn	Create( op: BudBinOp, left: Box< dyn Bud< Self>>, right: Box< dyn Bud< Self>>) -> Box< dyn Bud< Self>>
    {
        Box::new( BudNode::Create( op, left, right))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! impl_into_bud_for_primitives {
    ( $( $t:ty),*) => {
        $(
            impl IntoBud< $t> for $t
            {
                fn	IntoBud( self) -> Box< dyn Bud< $t>>
                {
                    Box::new( BudNode::NewVal( self))
                }
            }
            impl BudOp for $t
            {
            }
        )*
    };
}
impl_into_bud_for_primitives!( f64, f32, i32, i64, u32, u64, String, &'static str);

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! BudTree {

    // ── Strip outer parens ──────────────────────────────────────────────────────────────────────────
    ( $type:ident, ( $( $inner:tt)+ ) ) => { $crate::BudTree!( $type, $( $inner)+ ) };

    // ── Unary operators ─────────────────────────────────────────────────────────────────────────────
    ( $type:ident, * $l:tt $( $r:tt)* ) => {
        $crate::BudTree!( $type, ( $crate::stalks::bud::IntoBud::IntoBudUniOp( $crate::BudTree!( $type, $l ), $crate::stalks::bud::BudUniOp::STAR ) ) $( $r)* )
    };
    ( $type:ident, + $l:tt $( $r:tt)* ) => {
        $crate::BudTree!( $type, ( $crate::stalks::bud::IntoBud::IntoBudUniOp( $crate::BudTree!( $type, $l ), $crate::stalks::bud::BudUniOp::PLUS ) ) $( $r)* )
    };
    ( $type:ident, - $l:tt $( $r:tt)* ) => {
        $crate::BudTree!( $type, ( $crate::stalks::bud::IntoBud::IntoBudUniOp( $crate::BudTree!( $type, $l ), $crate::stalks::bud::BudUniOp::MINUS ) ) $( $r)* )
    };
    ( $type:ident, ! $l:tt $( $r:tt)* ) => {
        $crate::BudTree!( $type, ( $crate::stalks::bud::IntoBud::IntoBudUniOp( $crate::BudTree!( $type, $l ), $crate::stalks::bud::BudUniOp::BANG ) ) $( $r)* )
    };

    // ── Binary: (group) OP rhs ──────────────────────────────────────────────────────────────────────
    ( $type:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $crate::BudTree!( @bg $type, SHL, ( $( $l)+ ), $( $r)+ ) };
    ( $type:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $crate::BudTree!( @bg $type, SHR, ( $( $l)+ ), $( $r)+ ) };
    ( $type:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $crate::BudTree!( @bg $type, LT,  ( $( $l)+ ), $( $r)+ ) };
    ( $type:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $crate::BudTree!( @bg $type, BOR, ( $( $l)+ ), $( $r)+ ) };

    // ── Binary: ident/literal OP rhs ────────────────────────────────────────────────────────────────
    ( $type:ident, $l:ident << $( $r:tt)+ ) => { $crate::BudTree!( @bl $type, SHL, $l, $( $r)+ ) };
    ( $type:ident, $l:ident >> $( $r:tt)+ ) => { $crate::BudTree!( @bl $type, SHR, $l, $( $r)+ ) };
    ( $type:ident, $l:ident <  $( $r:tt)+ ) => { $crate::BudTree!( @bl $type, LT,  $l, $( $r)+ ) };
    ( $type:ident, $l:ident |  $( $r:tt)+ ) => { $crate::BudTree!( @bl $type, BOR, $l, $( $r)+ ) };

    ( $type:ident, $l:literal << $( $r:tt)+ ) => { $crate::BudTree!( @bl $type, SHL, $l, $( $r)+ ) };
    ( $type:ident, $l:literal >> $( $r:tt)+ ) => { $crate::BudTree!( @bl $type, SHR, $l, $( $r)+ ) };
    ( $type:ident, $l:literal <  $( $r:tt)+ ) => { $crate::BudTree!( @bl $type, LT,  $l, $( $r)+ ) };
    ( $type:ident, $l:literal |  $( $r:tt)+ ) => { $crate::BudTree!( @bl $type, BOR, $l, $( $r)+ ) };

    // ── Closure literal ─────────────────────────────────────────────────────────────────────────────
    ( $type:ident, | $( $body:tt)+ ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $type::New( | $( $body)+ ) )
    };

    // ── Leaf [ action ] ────────────────────────────────────────────────────────────────────────────
    ( $type:ident, $l:literal [ $( $inner:tt )* ] ) => {
        $crate::BudTree!( @closure_match $type, $l, $( $inner )* )
    };
    ( $type:ident, ( $( $expr:tt)+ ) [ $( $inner:tt )* ] ) => {
        $crate::BudTree!( @closure_match $type, $crate::BudTree!( $type, $( $expr)+ ), $( $inner )* )
    };

    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( $type:ident, [ $leaf:expr ]  ) => { $crate::stalks::bud::IntoBud::IntoBudBox( $leaf ) };


    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( $type:ident, $leaf:expr ) => { $crate::stalks::bud::IntoBud::IntoBud( $leaf ) };

    // ═══ Internal helpers ═══════════════════════════════════════════════════════════════════════════

    // @closure_match : resolves closure vs non-closure for action leaf
    ( @closure_match $type:ident, $base:expr, | $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::IntoBudAction( $base, $crate::stalks::bud::IntoBud::IntoBud( $type::New( | $($closure)* ) ) )
    };
    ( @closure_match $type:ident, $base:expr, || $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::IntoBudAction( $base, $crate::stalks::bud::IntoBud::IntoBud( $type::New( || $($closure)* ) ) )
    };
    ( @closure_match $type:ident, $base:expr, move | $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::IntoBudAction( $base, $crate::stalks::bud::IntoBud::IntoBud( $type::New( move | $($closure)* ) ) )
    };
    ( @closure_match $type:ident, $base:expr, move || $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::IntoBudAction( $base, $crate::stalks::bud::IntoBud::IntoBud( $type::New( move || $($closure)* ) ) )
    };

    // @bg : binary — (group) OP rhs
    ( @bg $type:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        < $type as $crate::stalks::bud::BudOp>::Create(
            $crate::stalks::bud::BudBinOp::$op,
            $crate::BudTree!( $type, $( $l)+ ),
            $crate::BudTree!( $type, $( $r)+ ) )
    };

    // @bl : binary — leaf OP rhs
    ( @bl $type:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        < $type as $crate::stalks::bud::BudOp>::Create(
            $crate::stalks::bud::BudBinOp::$op,
            $crate::stalks::bud::IntoBud::IntoBud( $l ),
            $crate::BudTree!( $type, $( $r)+ ) )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
