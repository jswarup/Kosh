//-- coro_kernel.rs ---------------------------------------------------------------------------------------------------------------

use	std::{
    cell::RefCell,
    ops::{ Index, IndexMut },
    sync::Arc,
};
use	crate::{
    rube::{ Reg, TriggerId },
    silo::{ Arr, Buff, IAccess, U32, USeg },
    stalks::Coro,
};
pub use	crate::stalks::{ CoroRes, CoroYielder, ICoro };

//---------------------------------------------------------------------------------------------------------------------------------

pub const CORO_MAX_PORTS: usize = 16;

#[derive( Copy, Clone, Debug, PartialEq, Eq)]
pub struct CoroPorts
{
    pub _Vals: [Reg; CORO_MAX_PORTS],
    pub _Len:  U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CoroPorts
{
    pub fn	New() -> Self
    {
        return Self {
            _Vals: [Reg::default(); CORO_MAX_PORTS],
            _Len:  U32::_0,
        };
    }

    pub fn	Empty() -> Self
    {
        return Self::New();
    }

    pub fn	Single( reg: Reg) -> Self
    {
        let  	mut ports = Self::New();
        ports._Vals[0] = reg;
        ports._Len = U32( 1);
        return ports;
    }

    pub fn	Pair( r1: Reg, r2: Reg) -> Self
    {
        let  	mut ports = Self::New();
        ports._Vals[0] = r1;
        ports._Vals[1] = r2;
        ports._Len = U32( 2);
        return ports;
    }

    pub fn	FromSlice( slice: &[Reg]) -> Self
    {
        let  	mut ports = Self::New();
        let  	count = slice.len().min( CORO_MAX_PORTS);
        let  	mut i = 0;
        while i < count {
            ports._Vals[i] = slice[i];
            i += 1;
        }
        ports._Len = U32( count as u32);
        return ports;
    }

    pub fn	FromArr( arr: Arr< '_, Reg>) -> Self
    {
        let  	mut ports = Self::New();
        let  	count = arr.Size().min( U32( CORO_MAX_PORTS as u32));
        USeg::New( U32::_0, count).Traverse( |i| {
            ports._Vals[i.AsUsize()] = arr[i];
        });
        ports._Len = count;
        return ports;
    }

    #[inline]
    pub fn	Len( &self) -> U32
    {
        return self._Len;
    }

    #[inline]
    pub fn	IsEmpty( &self) -> bool
    {
        return self._Len == U32::_0;
    }

    #[inline]
    pub fn	Get< I: Into< U32>>( &self, idx: I) -> Reg
    {
        let  	i: U32 = idx.into();
        assert!( i < self._Len, "Index out of bounds");
        return self._Vals[i.AsUsize()];
    }

    #[inline]
    pub fn	Set< I: Into< U32>>( &mut self, idx: I, val: Reg)
    {
        let  	i: U32 = idx.into();
        assert!( i < self._Len, "Index out of bounds");
        self._Vals[i.AsUsize()] = val;
    }

    #[inline]
    pub fn	Push( &mut self, val: Reg)
    {
        assert!( ( self._Len.0 as usize) < CORO_MAX_PORTS, "CoroPorts capacity exceeded");
        self._Vals[self._Len.AsUsize()] = val;
        self._Len = self._Len + U32( 1);
    }

    #[inline]
    pub fn	Slice( &self) -> &[Reg]
    {
        return &self._Vals[..self._Len.AsUsize()];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for CoroPorts
{
    fn	default() -> Self
    {
        return Self::New();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< I: Into< U32>> Index< I> for CoroPorts
{
    type Output = Reg;

    #[inline]
    fn	index( &self, idx: I) -> &Self::Output
    {
        let  	i: U32 = idx.into();
        assert!( i < self._Len, "Index out of bounds");
        return &self._Vals[i.AsUsize()];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< I: Into< U32>> IndexMut< I> for CoroPorts
{
    #[inline]
    fn	index_mut( &mut self, idx: I) -> &mut Self::Output
    {
        let  	i: U32 = idx.into();
        assert!( i < self._Len, "Index out of bounds");
        return &mut self._Vals[i.AsUsize()];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub type CoroInstance = Coro< CoroPorts, CoroPorts, ()>;
pub type CoroKernelFactory = Arc< dyn Fn() -> CoroInstance + Send + Sync>;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct CoroWarp
{
    pub _ModStart:    U32,
    pub _Count:       U32,
    pub _Instances:   Buff< RefCell< CoroInstance>>,
    pub _InTriggers:  Buff< Buff< TriggerId>>,
    pub _OutTriggers: Buff< Buff< TriggerId>>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CoroWarp
{
    pub fn	New(
        modStart: U32,
        count: U32,
        instances: Buff< RefCell< CoroInstance>>,
        inTriggers: Buff< Buff< TriggerId>>,
        outTriggers: Buff< Buff< TriggerId>>,
    ) -> Self
    {
        return Self {
            _ModStart:    modStart,
            _Count:       count,
            _Instances:   instances,
            _InTriggers:  inTriggers,
            _OutTriggers: outTriggers,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
