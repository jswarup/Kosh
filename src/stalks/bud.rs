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

#[derive( Debug, PartialEq, Eq)]
pub enum TraversalEvent
{
    Entry,
    Exit,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl dyn Bud + '_
{
    pub fn	TraverseDFS( &self, f: &mut dyn FnMut( &dyn Bud, TraversalEvent))
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

pub enum BudType< T>
{
    Val( T),
    Par( Box< dyn Bud>, Box< dyn Bud>),
    Seq( Box< dyn Bud>, Box< dyn Bud>),
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

    pub fn	NewPar( left: Box< dyn Bud>, right: Box< dyn Bud>) -> Self
    {
        Self {
            _Type: BudType::Par( left, right),
        }
    }

    pub fn	NewSeq( left: Box< dyn Bud>, right: Box< dyn Bud>) -> Self
    {
        Self {
            _Type: BudType::Seq( left, right),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Into< U32> + Clone> BudNode< T>
{
    pub fn	TraverseDFS( &self, f: &mut dyn FnMut( &dyn Bud, TraversalEvent))
    {
        ( self as &dyn Bud).TraverseDFS( f);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: Into< U32> + Clone> Bud for BudNode< T>
{
    fn	Id( &self) -> U32
    {
        match &self._Type {
            BudType::Val( id) => id.clone().into(),
            BudType::Par( left, right) => left.Id() | right.Id(),
            BudType::Seq( left, right) => {
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
            BudType::Par( left, _) => Some( &**left),
            BudType::Seq( left, _) => Some( &**left),
        }
    }

    fn	Right( &self) -> Option< &dyn Bud>
    {
        match &self._Type {
            BudType::Val( _) => None,
            BudType::Par( _, right) => Some( &**right),
            BudType::Seq( _, right) => Some( &**right),
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
        Box::new( $crate::stalks::bud::BudNode::< $crate::silo::uint::U32>::NewSeq( $crate::bud!( $($lhs)+ ), $crate::bud!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud>
    };
    ( ( $($lhs:tt)+ ) | $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::< $crate::silo::uint::U32>::NewPar( $crate::bud!( $($lhs)+ ), $crate::bud!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud>
    };
    ( $lhs:ident < $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::< $crate::silo::uint::U32>::NewSeq( $crate::stalks::bud::IntoBud::IntoBud( $lhs ), $crate::bud!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud>
    };
    ( $lhs:ident | $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::< $crate::silo::uint::U32>::NewPar( $crate::stalks::bud::IntoBud::IntoBud( $lhs ), $crate::bud!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud>
    };
    ( $leaf:ident ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $leaf )
    };
    ( $leaf:expr ) => {
        $crate::stalks::bud::IntoBud::IntoBud( $leaf )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
