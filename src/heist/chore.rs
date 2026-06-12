//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::stalks::{ IWork, IWorker };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug, PartialEq)]
pub struct Chore
{
    _Closure: fn( &dyn IWorker),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Chore
{
    fn	default() -> Self
    {
        Self { _Closure: |_| {} }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Chore
{
    pub fn	New( f: fn( &dyn IWorker)) -> Self
    {
        Self { _Closure: f }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Chore
{
    fn	DoWork( &mut self, worker: &dyn IWorker)
    {
        ( self._Closure)( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< fn( &dyn IWorker) > for Chore
{
    fn	from( f: fn( &dyn IWorker)) -> Self
    {
        Self::New( f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for Chore
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        write!( f, "Chore")
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ChoreTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_LT  $( $args:tt)* ) => { $crate::BudTree!( @feature_LT  $( $args)* ) };
    ( @feature_BOR $( $args:tt)* ) => { $crate::BudTree!( @feature_BOR $( $args)* ) };
    ( @feature_NEW $( $args:tt)* ) => { $crate::BudTree!( @feature_NEW $( $args)* ) };
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    // Forward unhandled internal callbacks to BudTree (e.g., disallowed features like @feature_SHL)
    ( @ $( $inner:tt )+ ) => {
        $crate::BudTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::BudTree!( @define [ $crate::ChoreTree ], Chore, $( $inner)+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
