//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::heist::Maestro;
use	crate::silo::{ Stash, U16 };
use	crate::stalks::{ Bud, IWork, IWorker };

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

impl Bud< Chore> for Chore
{
    fn	Val( &self) -> Chore
    {
        *self
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::BudOp for Chore
{
    fn	IsOpAllowed( op: crate::stalks::bud::BudBinOp) -> bool
    {
        matches!( 
            op,
            crate::stalks::bud::BudBinOp::LT | crate::stalks::bud::BudBinOp::BOR
        )
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

impl< T> dyn Bud< T> + '_
where
    T: IWork + Clone + Default + 'static,
{
    pub fn	Post( &self, worker: &dyn IWorker)
    {
        if worker.IsSequential() {
            fn	RunSequential< T>( node: &dyn Bud< T>, worker: &dyn IWorker)
            where
                T: IWork + Clone + Default + 'static,
            {
                if node.Left().is_none() && node.Right().is_none() {
                    let  	mut val = node.Val();
                    val.DoWork( worker);
                    return;
                }
                if node.Op() == "|" {
                    if let  	Some( left) = node.Left() {
                        RunSequential( left, worker);
                    }
                    if let  	Some( right) = node.Right() {
                        RunSequential( right, worker);
                    }
                    return;
                }
                if node.Op() == "<" {
                    if let  	Some( left) = node.Left() {
                        RunSequential( left, worker);
                    }
                    if let  	Some( right) = node.Right() {
                        RunSequential( right, worker);
                    }
                    return;
                }
            }
            RunSequential( self, worker);
            return;
        }
        let  	maestro: &Maestro< '_> = Maestro::FromWorker( worker);
        struct JobStash
        {
            _JobStash: Stash< U16>,
        }
        impl JobStash
        {
            fn	Process< T>( &mut self, node: &dyn Bud< T>, maestro: &Maestro< '_>, succId: U16) -> U16
            where
                T: IWork + Clone + Default + 'static,
            {
                if node.Left().is_none() && node.Right().is_none() {
                    let  	mut jobId = maestro.ConstructJob( succId, node.Val());
                    self._JobStash.Pushback( &mut jobId);
                    return jobId;
                }
                if node.Op() == "|" {
                    let  	_succR = self.Process( node.Right().unwrap(), maestro, succId);
                    let  	_succL = self.Process( node.Left().unwrap(), maestro, succId);
                    return succId;
                }
                if node.Op() == "<" {
                    let  	mark = self._JobStash.Size();
                    let  	rJobId = self.Process( node.Right().unwrap(), maestro, succId);
                    let  	jStk = self._JobStash.Stk();
                    let  	rSz = jStk.Size() - mark;
                    if rSz == 1 {
                        return self.Process( node.Left().unwrap(), maestro, rJobId);
                    }
                    let  	mut rXStash: Stash< U16> = Stash::New( rSz);
                    self._JobStash.Stk().Export( &rXStash.Stk(), rSz);
                    let  	branchId = maestro.ConstructEnqueueBulk( succId, rXStash.BuffOut());
                    let  	succL = self.Process( node.Left().unwrap(), maestro, branchId);
                    return succL;
                } else {
                    assert!( false);
                    return U16( 0);
                }
            }
        }
        let  	succId = maestro.CurSuccId();
        let  	mut jobStash = JobStash {
            _JobStash: Stash::New( 0),
        };
        jobStash.Process( self, maestro, succId);
        let  	jobArr = jobStash._JobStash.Stk().Arr();
        jobArr.USeg().Traverse( |i| {
            maestro.EnqueueJob( jobArr.MutAt( i));
        });
        return;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl crate::stalks::bud::IntoBud< Chore> for fn( &dyn IWorker)
{
    fn	IntoBud( self) -> Box< dyn Bud< Chore>>
    {
        Box::new( Chore::New( self))
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
    ( $( $inner:tt)+ )  => { $crate::ChoreTree!( @cb [ $crate::ChoreTree ], Chore, $( $inner)+ ) };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ChoreNodeTree {
    // ---- OPT-IN FEATURES -----------------------------------------------------------------------------------------------------
    ( @feature_LT  $( $args:tt)* ) => { $crate::BNodeTree!( @feature_LT  $( $args)* ) };
    ( @feature_BOR $( $args:tt)* ) => { $crate::BNodeTree!( @feature_BOR $( $args)* ) };
    ( @feature_NEW $( $args:tt)* ) => { $crate::BNodeTree!( @feature_NEW $( $args)* ) };
    // ---- FALLBACKS -------------------------------------------------------------------------------------------------------------
    // Forward unhandled internal callbacks to BNodeTree (e.g., disallowed features like @feature_SHL)
    ( @ $( $inner:tt )+ ) => {
        $crate::BNodeTree!( @ $( $inner )+ )
    };
    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => {
        $crate::BNodeTree!( @define [ $crate::ChoreNodeTree ], Chore, $( $inner)+ )
    };
}

//---------------------------------------------------------------------------------------------------------------------------------
