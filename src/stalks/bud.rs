//-- bud.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::silo::uint::U32;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait Bud
{
    fn	Id( &self) -> U32;

    fn	Left( &self) -> Option< &dyn Bud>;

    fn	Right( &self) -> Option< &dyn Bud>;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub enum BudType< T>
{
    Val( T),
    Or( Box< dyn Bud>, Box< dyn Bud>),
    Less( Box< dyn Bud>, Box< dyn Bud>),
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

    pub fn	NewOr( left: Box< dyn Bud>, right: Box< dyn Bud>) -> Self
    {
        Self {
            _Type: BudType::Or( left, right),
        }
    }

    pub fn	NewLess( left: Box< dyn Bud>, right: Box< dyn Bud>) -> Self
    {
        Self {
            _Type: BudType::Less( left, right),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Into< U32> + Clone> Bud for BudNode< T>
{
    fn	Id( &self) -> U32
    {
        match &self._Type {
            BudType::Val( id) => id.clone().into(),
            BudType::Or( left, right) => left.Id() | right.Id(),
            BudType::Less( left, right) => {
                if left.Id() < right.Id() {
                    U32( 1)
                } else {
                    U32( 0)
                }
            }
        }
    }

    fn	Left( &self) -> Option< &dyn Bud>
    {
        match &self._Type {
            BudType::Val( _) => None,
            BudType::Or( left, _) => Some( &**left),
            BudType::Less( left, _) => Some( &**left),
        }
    }

    fn	Right( &self) -> Option< &dyn Bud>
    {
        match &self._Type {
            BudType::Val( _) => None,
            BudType::Or( _, right) => Some( &**right),
            BudType::Less( _, right) => Some( &**right),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IntoBud
{
    fn	IntoBud( self) -> Box< dyn Bud>;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IntoBud for Box< dyn Bud>
{
    fn	IntoBud( self) -> Box< dyn Bud>
    {
        self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IntoBud for U32
{
    fn	IntoBud( self) -> Box< dyn Bud>
    {
        Box::new( BudNode::NewVal( self))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IntoBud for u32
{
    fn	IntoBud( self) -> Box< dyn Bud>
    {
        Box::new( BudNode::NewVal( U32( self)))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Into< U32> + Clone + 'static> IntoBud for BudNode< T>
{
    fn	IntoBud( self) -> Box< dyn Bud>
    {
        Box::new( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! bud {
    ( ( $($inner:tt)+ ) ) => {
        $crate::bud!( $($inner)+ )
    };
    ( ( $($lhs:tt)+ ) < $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::< $crate::silo::uint::U32>::NewLess( $crate::bud!( $($lhs)+ ), $crate::bud!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud>
    };
    ( ( $($lhs:tt)+ ) | $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::< $crate::silo::uint::U32>::NewOr( $crate::bud!( $($lhs)+ ), $crate::bud!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud>
    };
    ( $lhs:ident < $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::< $crate::silo::uint::U32>::NewLess( $crate::stalks::bud::IntoBud::IntoBud( $lhs ), $crate::bud!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud>
    };
    ( $lhs:ident | $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::< $crate::silo::uint::U32>::NewOr( $crate::stalks::bud::IntoBud::IntoBud( $lhs ), $crate::bud!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud>
    };
    ( $leaf:ident ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $leaf )
    };
    ( $leaf:expr ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $leaf )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
