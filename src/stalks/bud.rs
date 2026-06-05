//-- bud.rs -------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

pub trait Bud<T>
{
    fn	Val( &self) -> T;

    fn	Left( &self) -> Option< &dyn Bud<T>>
    {
        None
    }

    fn	Right( &self) -> Option< &dyn Bud<T>>
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
pub enum TraversalEvent
{
    Entry,
    Exit,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> dyn Bud<T>+ '_
{
    pub fn	TraverseDFS( &self, f: &mut dyn FnMut( &dyn Bud<T>, TraversalEvent))
    {
        let  	isLeaf = self.Left().is_none() && self.Right().is_none();
        f( self, TraversalEvent::Entry);

        if let Some( left) = self.Left() {
            left.TraverseDFS( f);
        }
        if let Some( right) = self.Right() {
            right.TraverseDFS( f);
        }

        if !isLeaf {
            f( self, TraversalEvent::Exit);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: std::fmt::Display> dyn Bud<T>+ '_
{
    pub fn	Print( &self)
    {
        let  	mut childCounts = Vec::new();
        self.TraverseDFS( &mut |node, event| {
            match event {
                TraversalEvent::Entry => {
                    if let Some( count) = childCounts.last_mut() {
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
            }
        });
        println!();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudBinOp {
    LT,
    BOR,
}

impl BudBinOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            BudBinOp::LT => "<",
            BudBinOp::BOR => "|",
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub enum BudType< T>
{
    Val( T),
    Bin( BudBinOp, Box< dyn Bud<T>>, Box< dyn Bud<T>>),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BudNode< T>
{
    _Type: BudType< T>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> BudNode< T>
{
    pub fn	NewVal( id: T) -> Self
    {
        Self {
            _Type: BudType::Val( id),
        }
    }

    pub fn	Create( op: BudBinOp, left: Box< dyn Bud<T>>, right: Box< dyn Bud<T>>) -> Self
    {
        Self {
            _Type: BudType::Bin( op, left, right),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> Bud<T> for BudNode< T>
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

    fn	Left( &self) -> Option< &dyn Bud<T>>
    {
        match &self._Type {
            BudType::Val( _) => None,
            BudType::Bin( _, left, _) => Some( &**left),
        }
    }

    fn	Right( &self) -> Option< &dyn Bud<T>>
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

pub trait IntoBud< T>: Sized
{
    fn	IntoBud( self) -> Box< dyn Bud< T>>;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T> IntoBud<T> for Box< dyn Bud<T>>
{
    fn	IntoBud( self) -> Box< dyn Bud<T>>
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Bud< T> + 'static> IntoBud< T> for T
{
    fn	IntoBud( self) -> Box< dyn Bud<T>>
    {
        Box::new( self )
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait BudOp: Clone + Default + 'static {
    fn seq(left: Box<dyn Bud<Self>>, right: Box<dyn Bud<Self>>) -> Box<dyn Bud<Self>> {
        Box::new(BudNode::Create(BudBinOp::LT, left, right))
    }
    fn par(left: Box<dyn Bud<Self>>, right: Box<dyn Bud<Self>>) -> Box<dyn Bud<Self>> {
        Box::new(BudNode::Create(BudBinOp::BOR, left, right))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! impl_into_bud_for_primitives {
    ($($t:ty),*) => {
        $(
            impl IntoBud<$t> for $t {
                fn IntoBud(self) -> Box<dyn Bud<$t>> {
                    Box::new(BudNode::NewVal(self))
                }
            }
            impl BudOp for $t {}
        )*
    };
}

impl_into_bud_for_primitives!(f64, f32, i32, i64, u32, u64, String, &'static str);

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! BudTree {
    ( $type:ident, ( $($inner:tt)+ ) ) => {
        $crate::BudTree!( $type, $($inner)+ )
    };
    ( $type:ident,  ( $($lhs:tt)+ ) < $($rhs:tt)+ ) => {
        <$type as $crate::stalks::bud::BudOp>::seq( $crate::BudTree!( $type, $($lhs)+ ), $crate::BudTree!( $type,$($rhs)+ ) )
    };
    ( $type:ident, ( $($lhs:tt)+ ) | $($rhs:tt)+ ) => {
        <$type as $crate::stalks::bud::BudOp>::par( $crate::BudTree!( $type,$($lhs)+ ), $crate::BudTree!( $type,$($rhs)+ ) )
    };
    ( $type:ident, $lhs:ident < $($rhs:tt)+ ) => {
        <$type as $crate::stalks::bud::BudOp>::seq( $crate::stalks::bud::IntoBud::IntoBud( $lhs ), $crate::BudTree!( $type, $($rhs)+ ) )
    };
    ( $type:ident, $lhs:ident | $($rhs:tt)+ ) => {
        <$type as $crate::stalks::bud::BudOp>::par( $crate::stalks::bud::IntoBud::IntoBud( $lhs ), $crate::BudTree!( $type, $($rhs)+ ) )
    };
    ( $type:ident, $leaf:expr ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $leaf )
    };

    ( ( $($inner:tt)+ ) ) => {
        $crate::BudTree!( $($inner)+ )
    };
    (  ( $($lhs:tt)+ ) < $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::Create( $crate::stalks::bud::BudBinOp::LT, $crate::BudTree!( $($lhs)+ ), $crate::BudTree!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud< _ >>
    };
    ( ( $($lhs:tt)+ ) | $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::Create( $crate::stalks::bud::BudBinOp::BOR, $crate::BudTree!( $($lhs)+ ), $crate::BudTree!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud< _ >>
    };
    ( $lhs:ident < $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::Create( $crate::stalks::bud::BudBinOp::LT, $crate::stalks::bud::IntoBud::IntoBud( $lhs ), $crate::BudTree!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud< _ >>
    };
    ( $lhs:ident | $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::Create( $crate::stalks::bud::BudBinOp::BOR, $crate::stalks::bud::IntoBud::IntoBud( $lhs ), $crate::BudTree!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud< _ >>
    };
    ( $leaf:expr ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $leaf )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
