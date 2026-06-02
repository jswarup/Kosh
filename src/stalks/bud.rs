//-- bud.rs -------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

pub trait Bud<T>
{
    fn	Val( &self) -> T;

    fn	Left( &self) -> Option< &dyn Bud<T>>;

    fn	Right( &self) -> Option< &dyn Bud<T>>;

    fn	Op( &self) -> &str;
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

pub enum BudType< T>
{
    Val( T),
    Par( Box< dyn Bud<T>>, Box< dyn Bud<T>>),
    Seq( Box< dyn Bud<T>>, Box< dyn Bud<T>>),
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

    pub fn	NewPar( left: Box< dyn Bud<T>>, right: Box< dyn Bud<T>>) -> Self
    {
        Self {
            _Type: BudType::Par( left, right),
        }
    }

    pub fn	NewSeq( left: Box< dyn Bud<T>>, right: Box< dyn Bud<T>>) -> Self
    {
        Self {
            _Type: BudType::Seq( left, right),
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
            BudType::Par( _left, _right) => T::default(),
            BudType::Seq( _left, _right) => T::default()
        }
    }

    fn	Left( &self) -> Option< &dyn Bud<T>>
    {
        match &self._Type {
            BudType::Val( _) => None,
            BudType::Par( left, _) => Some( &**left),
            BudType::Seq( left, _) => Some( &**left),
        }
    }

    fn	Right( &self) -> Option< &dyn Bud<T>>
    {
        match &self._Type {
            BudType::Val( _) => None,
            BudType::Par( _, right) => Some( &**right),
            BudType::Seq( _, right) => Some( &**right),
        }
    }

    fn	Op( &self) -> &str
    {
        match &self._Type {
            BudType::Val( _) => "",
            BudType::Par( _, _) => "|",
            BudType::Seq( _, _) => "<",
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IntoBud<T>
{
    fn	IntoBud( self) -> Box< dyn Bud<T>>;
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

impl< T> IntoBud<T> for BudNode< T>
where
    T: Clone + Default + 'static,
{
    fn	IntoBud( self) -> Box< dyn Bud<T>>
    {
        Box::new( self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! BudTree {
    ( ( $($inner:tt)+ ) ) => {
        $crate::BudTree!( $($inner)+ )
    };
    ( ( $($lhs:tt)+ ) < $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::NewSeq( $crate::BudTree!( $($lhs)+ ), $crate::BudTree!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud< _ >>
    };
    ( ( $($lhs:tt)+ ) | $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::NewPar( $crate::BudTree!( $($lhs)+ ), $crate::BudTree!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud< _ >>
    };
    ( $lhs:ident < $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::NewSeq( Box::new( $crate::stalks::bud::BudNode::NewVal( $lhs )), $crate::BudTree!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud< _ >>
    };
    ( $lhs:ident | $($rhs:tt)+ ) => {
        Box::new( $crate::stalks::bud::BudNode::NewPar( Box::new( $crate::stalks::bud::BudNode::NewVal( $lhs)), $crate::BudTree!( $($rhs)+ ) ) ) as Box< dyn $crate::stalks::bud::Bud< _ >>
    };
    ( $leaf:ident ) => {
        Box::new( $crate::stalks::bud::BudNode::NewVal( $leaf ))
    };
    ( $leaf:expr ) => {
        Box::new( $crate::stalks::bud::BudNode::NewVal( $leaf ))
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
