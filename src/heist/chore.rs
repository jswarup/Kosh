//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::{ flux::{ IXFluxable, xflux::XField }, silo::U32, stalks::{ ChildOp, DynINode, INode, IntoWorkPtr, WorkPtr } };
use	std::fmt;
use	crate::stalks::{ DynIWorker, IWork };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug)]
pub struct Chore
{
    pub _DocStr: &'static str,
    _Closure: fn( &DynIWorker< '_>),
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Chore
{
    fn	default() -> Self
    {
        Self { _DocStr: "", _Closure: |_| {} }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for Chore
{
    fn	ToXFlux< 'b>( &'b self, field: &mut XField< 'b>)
    {
        let  	mut step = 0u32;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "DocStr".to_string();
                *item = XField::Str( self._DocStr);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Chore
{
    pub fn	New( f: fn( &DynIWorker< '_>)) -> Self
    {
        Self { _DocStr: "", _Closure: f }
    }

    pub fn	NewDoc( docStr: &'static str, f: fn( &DynIWorker< '_>)) -> Self
    {
        Self { _DocStr: docStr, _Closure: f }
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

impl fmt::Display for Chore
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        if self._DocStr.is_empty() {
            write!( f, "Chore")
        } else {
            write!( f, "Chore: {}", self._DocStr)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! Chore {
    ( $closure:expr) => {
        $crate::heist::Chore::New( $closure)
    };
    ( $doc:expr, $closure:expr) => {
        $crate::heist::Chore::NewDoc( $doc, $closure)
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ChoreTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_LT  $( $args:tt)* ) => { $crate::NoduleTree!( @feature_LT  $( $args)* ) };
    ( @feature_BOR $( $args:tt)* ) => { $crate::NoduleTree!( @feature_BOR $( $args)* ) };
    ( @feature_NEW $( $args:tt)* ) => { $crate::NoduleTree!( @feature_NEW $( $args)* ) };
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    // Forward unhandled internal callbacks to NoduleTree (e.g., disallowed features like @feature_SHL)
    ( @ $( $inner:tt )+ ) => {
        $crate::NoduleTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::NoduleTree!( @define [ $crate::ChoreTree ], Chore, $( $inner)+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> INode< 'a> for Chore
{
    fn	_Size( &self) -> U32 { U32(0) }
    fn	_At( &self, _idx: U32) -> &DynINode< 'a> { panic!("Leaf") }
    fn	Value( &self) -> Option< WorkPtr< 'a>>
    {
        Some( IntoWorkPtr::IntoWorkPtr( *self))
    }
    fn	DocStr( &self) -> &'static str { self._DocStr }
    fn	ChildOp( &self) -> ChildOp { ChildOp::None }
}

//---------------------------------------------------------------------------------------------------------------------------------
