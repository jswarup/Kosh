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
    pub fn Create(first: U32, sz: U32) -> Self
    {
        USeg
        {
            _First: first,
            _Last: (first + sz) - U32::from(1),
        }
    }

    pub fn First(&self) -> U32 {
        self._First
    }

    pub fn Last(&self) -> U32 {
        self._Last
    }

    pub fn Mid(&self) -> U32
    {
        // Compute mid as U32 using inner u32 arithmetic
        let sum = self._First.as_u32() + self._Last.as_u32();
        let mid = sum / 2;
        U32::from(mid)
    }

    pub fn Size(&self) -> U32
    {
        if self._Last >= self._First {
            self._Last + U32::from(1) - self._First
        } else {
            U32::from(0)
        }
    }

    pub fn IsEmpty(&self) -> bool
    {
        self.Size() == U32::from(0)
    }

    pub fn LSnip(&self, count: U32) -> Self
    {
        if self.Size() < count {
            USeg::Create(U32::_X, U32::from(0))
        } else {
            USeg::Create(self._First + count, self.Size() - count)
        }
    }

    pub fn RSnip(&self, count: U32) -> Self
    {
        if self.Size() < count {
            USeg::Create(U32::_X, U32::from(0))
        } else {
            USeg::Create(self._First, self.Size() - count)
        }
    }

    pub fn Span<F>(&self, mut f: F) -> bool
    where
        F: FnMut(U32) -> bool,
    {
        if self.IsEmpty() {
            return true;
        }
        for i in self._First.as_u32()..=self._Last.as_u32() {
            if !f(U32::from(i)) {
                return false;
            }
        }
        true
    }

    fn Partition<LessAt, SwapAt>(&self, lessAt: &LessAt, swapAt: &mut SwapAt) -> U32
    where
        LessAt: Fn(U32, U32) -> bool,
        SwapAt: FnMut(U32, U32),
    {
        let mid = self.Mid();
        if lessAt(self._First, mid) {
            swapAt(self._First, mid);
        }
        let mut pivot = self._First;
        self.LSnip(U32::from(1)).Span(&mut |i| {
            if lessAt(i, self._First) {
                pivot = pivot + U32::from(1);
                swapAt(pivot, i);
            }
            true
        });
        if lessAt(pivot, self._First) {
            swapAt(self._First, pivot);
        }
        pivot
    }


    pub fn QSort<LessAt, SwapAt>(&self, lessAt: &LessAt, swapAt: &mut SwapAt)
    where
        LessAt: Fn(U32, U32) -> bool,
        SwapAt: FnMut(U32, U32),
    {
        let pivot = self.Partition(lessAt, swapAt);
        let useg1 = USeg::Create(self._First, pivot - self._First);
        if useg1.Size() > U32::from(1) {
            useg1.QSort(lessAt, swapAt);
        }
        let useg2 = USeg::Create(pivot + U32::from(1), self._Last - pivot);
        if useg2.Size() > U32::from(1) {
            useg2.QSort(lessAt, swapAt);
        }
    }

}

//---------------------------------------------------------------------------------------------------------------------------------
