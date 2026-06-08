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

    // ── Top-level entry points for backward compatibility ───────────────────────────────────────────
    ( $type:ident, $( $inner:tt )+ ) => {
        $crate::BudTree!( @cb [ $crate::BudTree ], $type, $( $inner )+ )
    };

    // ---- FEATURE OPT-INS FOR BudTree ITSELF ----------------------------------------------------------------------------
    // BudTree explicitly opts in to all features by delegating back to its own builders.
    ( @feature_STAR [ $($cb:tt)* ], $type:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $($cb)* ], $type, STAR, $l $( $r )* ) };
    ( @feature_PLUS [ $($cb:tt)* ], $type:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $($cb)* ], $type, PLUS, $l $( $r )* ) };
    ( @feature_MINUS [ $($cb:tt)* ], $type:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $($cb)* ], $type, MINUS, $l $( $r )* ) };
    ( @feature_BANG [ $($cb:tt)* ], $type:ident, $l:tt $( $r:tt )* ) => { $crate::BudTree!( @uni [ $($cb)* ], $type, BANG, $l $( $r )* ) };

    ( @feature_SHL [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, SHL, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHL [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, SHL, $l, $( $r)+ ) };

    ( @feature_SHR [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, SHR, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_SHR [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, SHR, $l, $( $r)+ ) };

    ( @feature_LT [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, LT, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, LT, $l, $( $r)+ ) };

    ( @feature_BOR [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, BOR, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, BOR, $l, $( $r)+ ) };

    ( @feature_NEW [ $($cb:tt)* ], $type:ident, | $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, || $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( || $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, move | $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( move | $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, move || $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( move || $( $body)+ ) ) };

    ( @feature_ACTION [ $($cb:tt)* ], $type:ident, $l:literal [ $( $inner:tt )* ] ) => { $crate::BudTree!( @closure_match [ $($cb)* ], $type, $l, $( $inner )* ) };
    ( @feature_ACTION [ $($cb:tt)* ], $type:ident, ( $( $expr:tt)+ ) [ $( $inner:tt )* ] ) => { $crate::BudTree!( @closure_match [ $($cb)* ], $type, $($cb)* !( @cb [ $($cb)* ], $type, $( $expr)+ ), $( $inner )* ) };

    ( @feature_BOXET [ $($cb:tt)* ], $type:ident, $s:literal ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $s )
    };


    // ── Strip outer parens ──────────────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, ( $( $inner:tt)+ ) ) => { $($cb)* !( @cb [ $($cb)* ], $type, $( $inner)+ ) };

    // ── Unary operators ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, * $l:tt $( $r:tt)* ) => { $($cb)* !( @feature_STAR [ $($cb)* ], $type, $l $( $r )* ) };
    ( @cb [ $($cb:tt)* ], $type:ident, + $l:tt $( $r:tt)* ) => { $($cb)* !( @feature_PLUS [ $($cb)* ], $type, $l $( $r )* ) };
    ( @cb [ $($cb:tt)* ], $type:ident, - $l:tt $( $r:tt)* ) => { $($cb)* !( @feature_MINUS [ $($cb)* ], $type, $l $( $r )* ) };
    ( @cb [ $($cb:tt)* ], $type:ident, ! $l:tt $( $r:tt)* ) => { $($cb)* !( @feature_BANG [ $($cb)* ], $type, $l $( $r )* ) };

    // ── Binary: (group) OP rhs ──────────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, ( $( $l:tt)+ ) << $( $r:tt)+ ) => { $($cb)* !( @feature_SHL [ $($cb)* ], @bg $type, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, ( $( $l:tt)+ ) >> $( $r:tt)+ ) => { $($cb)* !( @feature_SHR [ $($cb)* ], @bg $type, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, ( $( $l:tt)+ ) <  $( $r:tt)+ ) => { $($cb)* !( @feature_LT  [ $($cb)* ], @bg $type, ( $( $l)+ ), $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, ( $( $l:tt)+ ) |  $( $r:tt)+ ) => { $($cb)* !( @feature_BOR [ $($cb)* ], @bg $type, ( $( $l)+ ), $( $r)+ ) };

    // ── Binary: ident/literal OP rhs ────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, $l:ident << $( $r:tt)+ ) => { $($cb)* !( @feature_SHL [ $($cb)* ], @bl $type, $l, $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, $l:ident >> $( $r:tt)+ ) => { $($cb)* !( @feature_SHR [ $($cb)* ], @bl $type, $l, $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, $l:ident <  $( $r:tt)+ ) => { $($cb)* !( @feature_LT  [ $($cb)* ], @bl $type, $l, $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, $l:ident |  $( $r:tt)+ ) => { $($cb)* !( @feature_BOR [ $($cb)* ], @bl $type, $l, $( $r)+ ) };

    ( @cb [ $($cb:tt)* ], $type:ident, $l:literal << $( $r:tt)+ ) => { $($cb)* !( @feature_SHL [ $($cb)* ], @bl $type, $l, $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, $l:literal >> $( $r:tt)+ ) => { $($cb)* !( @feature_SHR [ $($cb)* ], @bl $type, $l, $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, $l:literal <  $( $r:tt)+ ) => { $($cb)* !( @feature_LT  [ $($cb)* ], @bl $type, $l, $( $r)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, $l:literal |  $( $r:tt)+ ) => { $($cb)* !( @feature_BOR [ $($cb)* ], @bl $type, $l, $( $r)+ ) };

    // ── Closure literal ─────────────────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, | $( $body:tt)+ ) => { $($cb)* !( @feature_NEW [ $($cb)* ], $type, | $( $body)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, || $( $body:tt)+ ) => { $($cb)* !( @feature_NEW [ $($cb)* ], $type, || $( $body)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, move | $( $body:tt)+ ) => { $($cb)* !( @feature_NEW [ $($cb)* ], $type, move | $( $body)+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, move || $( $body:tt)+ ) => { $($cb)* !( @feature_NEW [ $($cb)* ], $type, move || $( $body)+ ) };

    // ── Leaf [ action ] ────────────────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, $l:literal [ $( $inner:tt )* ] ) => {
        $($cb)* !( @feature_ACTION [ $($cb)* ], $type, $l [ $( $inner )* ] )
    };
    ( @cb [ $($cb:tt)* ], $type:ident, ( $( $expr:tt)+ ) [ $( $inner:tt )* ] ) => {
        $($cb)* !( @feature_ACTION [ $($cb)* ], $type, ( $( $expr )+ ) [ $( $inner )* ] )
    };

    // ── Binary: [ boxet ] OP rhs ────────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, [ $s:literal ] << $( $r:tt)+ ) => { $($cb)* !( @feature_SHL [ $($cb)* ], @bg $type, ( $($cb)* !( @feature_BOXET [ $($cb)* ], $type, $s ) ), $( $r )+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, [ $s:literal ] >> $( $r:tt)+ ) => { $($cb)* !( @feature_SHR [ $($cb)* ], @bg $type, ( $($cb)* !( @feature_BOXET [ $($cb)* ], $type, $s ) ), $( $r )+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, [ $s:literal ] <  $( $r:tt)+ ) => { $($cb)* !( @feature_LT  [ $($cb)* ], @bg $type, ( $($cb)* !( @feature_BOXET [ $($cb)* ], $type, $s ) ), $( $r )+ ) };
    ( @cb [ $($cb:tt)* ], $type:ident, [ $s:literal ] |  $( $r:tt)+ ) => { $($cb)* !( @feature_BOR [ $($cb)* ], @bg $type, ( $($cb)* !( @feature_BOXET [ $($cb)* ], $type, $s ) ), $( $r )+ ) };

    // ── Leaf Boxet ──────────────────────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, [ $s:literal ] ) => {
        $($cb)* !( @feature_BOXET [ $($cb)* ], $type, $s )
    };

    // ── Leaf fallback ───────────────────────────────────────────────────────────────────────────────
    ( @cb [ $($cb:tt)* ], $type:ident, $leaf:expr ) => { $crate::stalks::bud::IntoBud::IntoBud( $leaf ) };

    // ---- Internal helpers ----------------------------------------------------------------------------------------------------

    // @uni : unary - OP rhs
    ( @uni [ $($cb:tt)* ], $type:ident, $op:ident, $l:tt $( $r:tt)* ) => {
        $($cb)* !( @cb [ $($cb)* ], $type, ( $crate::stalks::bud::IntoBud::IntoBudUniOp( $($cb)* !( @cb [ $($cb)* ], $type, $l ), $crate::stalks::bud::BudUniOp::$op ) ) $( $r)* )
    };

    // @closure_match : resolves closure vs non-closure for action leaf
    ( @closure_match [ $($cb:tt)* ], $type:ident, $base:expr, | $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::IntoBudAction( $base, $crate::stalks::bud::IntoBud::IntoBud( $type::New( | $($closure)* ) ) )
    };
    ( @closure_match [ $($cb:tt)* ], $type:ident, $base:expr, || $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::IntoBudAction( $base, $crate::stalks::bud::IntoBud::IntoBud( $type::New( || $($closure)* ) ) )
    };
    ( @closure_match [ $($cb:tt)* ], $type:ident, $base:expr, move | $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::IntoBudAction( $base, $crate::stalks::bud::IntoBud::IntoBud( $type::New( move | $($closure)* ) ) )
    };
    ( @closure_match [ $($cb:tt)* ], $type:ident, $base:expr, move || $( $closure:tt )* ) => {
        $crate::stalks::bud::IntoBud::IntoBudAction( $base, $crate::stalks::bud::IntoBud::IntoBud( $type::New( move || $($closure)* ) ) )
    };

    // @bg : binary — (group) OP rhs
    ( @bg [ $($cb:tt)* ], $type:ident, $op:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => {
        < $type as $crate::stalks::bud::BudOp>::Create(
            $crate::stalks::bud::BudBinOp::$op,
            $($cb)* !( @cb [ $($cb)* ], $type, $( $l)+ ),
            $($cb)* !( @cb [ $($cb)* ], $type, $( $r)+ ) )
    };

    // @bl : binary — leaf OP rhs
    ( @bl [ $($cb:tt)* ], $type:ident, $op:ident, $l:expr, $( $r:tt)+ ) => {
        < $type as $crate::stalks::bud::BudOp>::Create(
            $crate::stalks::bud::BudBinOp::$op,
            $crate::stalks::bud::IntoBud::IntoBud( $l ),
            $($cb)* !( @cb [ $($cb)* ], $type, $( $r)+ ) )
    };

    // ---- DEFAULT FALLBACK ERRORS FOR DISABLED FEATURES -------------------------------------------------------------
    ( @feature_STAR $( $args:tt )* ) => { compile_error!("Unary STAR (*) is not enabled for this tree"); };
    ( @feature_PLUS $( $args:tt )* ) => { compile_error!("Unary PLUS (+) is not enabled for this tree"); };
    ( @feature_MINUS $( $args:tt )* ) => { compile_error!("Unary MINUS (-) is not enabled for this tree"); };
    ( @feature_BANG $( $args:tt )* ) => { compile_error!("Unary BANG (!) is not enabled for this tree"); };
    ( @feature_SHL $( $args:tt )* ) => { compile_error!("Binary SHL (<<) is not enabled for this tree"); };
    ( @feature_SHR $( $args:tt )* ) => { compile_error!("Binary SHR (>>) is not enabled for this tree"); };
    ( @feature_LT $( $args:tt )* ) => { compile_error!("Binary LT (<) is not enabled for this tree"); };
    ( @feature_BOR $( $args:tt )* ) => { compile_error!("Binary BOR (|) is not enabled for this tree"); };
    ( @feature_NEW $( $args:tt )* ) => { compile_error!("Closure literal is not enabled for this tree"); };
    ( @feature_ACTION $( $args:tt )* ) => { compile_error!("Bracketed action [ closure ] is not enabled for this tree"); };
    ( @feature_BOXET $( $args:tt )* ) => { compile_error!("Boxet [ ... ] is not enabled for this tree"); };
}

//---------------------------------------------------------------------------------------------------------------------------------
