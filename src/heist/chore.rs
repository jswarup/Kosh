//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
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

impl crate::flux::IXFluxable for Chore
{
    fn	ToXFlux< 'b>( &'b self, field: &mut crate::flux::xflux::XField< 'b>)
    {
        let  	mut step = 0u32;
        *field = crate::flux::xflux::XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "DocStr".to_string();
                *item = crate::flux::xflux::XField::Str( self._DocStr);
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

impl std::fmt::Display for Chore
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
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

impl< 'a> crate::stalks::INode< 'a> for Chore
{
    fn	_Size( &self) -> crate::silo::U32 { crate::silo::U32(0) }
    fn	_At( &self, _idx: crate::silo::U32) -> &crate::stalks::DynINode< 'a> { panic!("Leaf") }
    fn	Value( &self) -> Option< crate::stalks::WorkPtr< 'a>>
    {
        Some( crate::stalks::IntoWorkPtr::IntoWorkPtr( *self))
    }
    fn	DocStr( &self) -> &'static str { self._DocStr }
    fn	Attrib( &self) -> Option< &crate::stalks::Attrib> { None }
    fn	ChildOp( &self) -> crate::stalks::ChildOp { crate::stalks::ChildOp::None }
}

//---------------------------------------------------------------------------------------------------------------------------------
