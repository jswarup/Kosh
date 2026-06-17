//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::stalks::{ DynIWorker, IWork };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug)]
pub struct Chore
{
    _Closure: fn( &DynIWorker< '_>),
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
    pub fn	New( f: fn( &DynIWorker< '_>)) -> Self
    {
        Self { _Closure: f }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IWork for Chore
{
    fn	DoWork( &mut self, worker: &DynIWorker< '_>)
    {
        ( self._Closure)( worker);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl From< fn( &DynIWorker< '_>) > for Chore
{
    fn	from( f: fn( &DynIWorker< '_>)) -> Self
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
    ( @feature_LT  $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_LT  $( $args)* ) };
    ( @feature_BOR $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_BOR $( $args)* ) };
    ( @feature_NEW $( $args:tt)* ) => { $crate::BiNodeTree!( @feature_NEW $( $args)* ) };
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    // Forward unhandled internal callbacks to BiNodeTree (e.g., disallowed features like @feature_SHL)
    ( @ $( $inner:tt )+ ) => {
        $crate::BiNodeTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::BiNodeTree!( @define [ $crate::ChoreTree ], Chore, $( $inner)+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
