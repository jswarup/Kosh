//-- silo/useg.rs ---------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------
use crate::silo::uint::U32;
#[derive( Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct USeg
{
    pub _First: U32,
    pub _Last: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl USeg
{
    pub fn	Create< F: Into< U32>, S: Into< U32>>( first: F, sz: S) -> Self
    {
        let fst = first.into();
        let size = sz.into();
        USeg {
            _First: fst,
            _Last: ( fst + size) - 1,
        }
    }

    pub fn	First( &self) -> U32
    {
        self._First
    }

    pub fn	Last( &self) -> U32
    {
        self._Last
    }

    pub fn	Mid( &self) -> U32
    {
        // Compute mid as U32 using inner u32 arithmetic
        let sum = self._First.as_u32() + self._Last.as_u32();
        let mid = sum / 2;
        U32( mid)
    }

    pub fn	Size( &self) -> U32
    {
        if self._Last >= self._First {
            self._Last + 1 - self._First
        } else {
            U32::_0
        }
    }

    pub fn	IsEmpty( &self) -> bool
    {
        self.Size() == 0
    }

    pub fn	LSnip< C: Into< U32>>( &self, count: C) -> Self
    {
        let cnt = count.into();
        if self.Size() < cnt {
            USeg::Create( U32::_X, 0)
        } else {
            USeg::Create( self._First + cnt, self.Size() - cnt)
        }
    }

    pub fn	RSnip< C: Into< U32>>( &self, count: C) -> Self
    {
        let cnt = count.into();
        if self.Size() < cnt {
            USeg::Create( U32::_X, 0)
        } else {
            USeg::Create( self._First, self.Size() - cnt)
        }
    }

    pub fn	Span< F>( &self, mut f: F) -> bool
    where
        F: FnMut( U32) -> bool,
    {
        if self.IsEmpty() {
            return true;
        }
        for i in self._First.as_u32()..=self._Last.as_u32() {
            if !f( U32( i)) {
                return false;
            }
        }
        true
    }

    fn	Partition< LessAt, SwapAt>( &self, lessAt: &LessAt, swapAt: &mut SwapAt) -> U32
    where
        LessAt: Fn( U32, U32) -> bool,
        SwapAt: FnMut( U32, U32),
    {
        let mid = self.Mid();
        if lessAt( self._First, mid) {
            swapAt( self._First, mid);
        }
        let mut pivot = self._First;
        self.LSnip( 1).Span( &mut |i| {
            if lessAt( i, self._First) {
                pivot = pivot + 1;
                swapAt( pivot, i);
            }
            true
        });
        if lessAt( pivot, self._First) {
            swapAt( self._First, pivot);
        }
        pivot
    }

    pub fn	QSort< LessAt, SwapAt>( &self, lessAt: &LessAt, swapAt: &mut SwapAt)
    where
        LessAt: Fn( U32, U32) -> bool,
        SwapAt: FnMut( U32, U32),
    {
        if self.Size() <= 1 {
            return;
        }
        let pivot = self.Partition( lessAt, swapAt);
        let useg1 = USeg::Create( self._First, pivot - self._First);
        if useg1.Size() > 1 {
            useg1.QSort( lessAt, swapAt);
        }
        let useg2 = USeg::Create( pivot + 1, self._Last - pivot);
        if useg2.Size() > 1 {
            useg2.QSort( lessAt, swapAt);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
