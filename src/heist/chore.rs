//-- chore.rs -------------------------------------------------------------------------------------------------------------------------
use	crate::heist::maestro::Maestro;
use	crate::silo::stash::Stash;
use	crate::silo::uint::U16;
use	crate::stalks::bud::Bud;
use	crate::stalks::work::{ IWork, IWorker };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Copy, Clone, Debug)]
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
                    let  	choreJob: Box< dyn IWork> = Box::new( node.Val());
                    let  	mut jobId = maestro.ConstructJob( succId, choreJob);
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
    // ═══ OPT-IN FEATURES ════════════════════════════════════════════════════════════════════════════

    // Enable LT (<)
    ( @feature_LT [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, LT, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_LT [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, LT, $l, $( $r)+ ) };

    // Enable BOR (|)
    ( @feature_BOR [ $($cb:tt)* ], @bg $type:ident, ( $( $l:tt)+ ), $( $r:tt)+ ) => { $crate::BudTree!( @bg [ $($cb)* ], $type, BOR, ( $( $l)+ ), $( $r)+ ) };
    ( @feature_BOR [ $($cb:tt)* ], @bl $type:ident, $l:expr, $( $r:tt)+ ) => { $crate::BudTree!( @bl [ $($cb)* ], $type, BOR, $l, $( $r)+ ) };

    // Enable Closure literal (NEW)
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, | $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( | $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, || $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( || $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, move | $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( move | $( $body)+ ) ) };
    ( @feature_NEW [ $($cb:tt)* ], $type:ident, move || $( $body:tt)+ ) => { $crate::stalks::bud::IntoBud::IntoBud( $type::New( move || $( $body)+ ) ) };

    // ═══ FALLBACKS ══════════════════════════════════════════════════════════════════════════════════

    // Forward unhandled internal callbacks to BudTree (e.g., disallowed features like @feature_SHL)
    ( @ $( $inner:tt )+ ) => {
        $crate::BudTree!( @ $( $inner )+ )
    };

    // Top-level entry (user code)
    ( $( $inner:tt)+ )  => { $crate::ChoreTree!( @cb [ $crate::ChoreTree ], Chore, $( $inner)+ ) };
}

//---------------------------------------------------------------------------------------------------------------------------------
