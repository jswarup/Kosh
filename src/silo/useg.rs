//-- silo/useg.rs ---------------------------------------------------------------------------------------------------------------------
use	crate::silo::U32;
use	crate::stalks::IWorker;

//---------------------------------------------------------------------------------------------------------------------------------

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
        let  	fst = first.into();
        let  	size = sz.into();
        USeg {
            _First: fst,
            _Last: ( fst + size) - 1,
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	First( &self) -> U32
    {
        self._First
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Last( &self) -> U32
    {
        self._Last
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Mid( &self) -> U32
    {
        // Compute mid as U32 using inner u32 arithmetic
        let  	sum = self._First.AsU32() + self._Last.AsU32();
        let  	mid = sum / 2;
        U32( mid)
    }

    //-----------------------------------------------------------------------------------------------------------------------------

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

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	LSnip< C: Into< U32>>( &self, count: C) -> Self
    {
        let  	cnt = count.into();
        if self.Size() < cnt {
            USeg::Create( U32::_X, 0)
        } else {
            USeg::Create( self._First + cnt, self.Size() - cnt)
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	RSnip< C: Into< U32>>( &self, count: C) -> Self
    {
        let  	cnt = count.into();
        if self.Size() < cnt {
            USeg::Create( U32::_X, 0)
        } else {
            USeg::Create( self._First, self.Size() - cnt)
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Span< F>( &self, mut lambda: F) -> bool
    where
        F: FnMut( U32) -> bool,
    {
        if self.IsEmpty() {
            return true;
        }
        for i in self._First.AsU32()..self._Last.AsU32() {
            if !lambda( U32( i)) {
                return false;
            }
        }
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Traverse< F>( &self, mut lambda: F)
    where
        F: FnMut( U32),
    {
        if self.IsEmpty() {
            return;
        }
        for i in self._First.AsU32()..=self._Last.AsU32() {
            lambda( U32( i))
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Partition< 'a, LessAt, SwapAt>( &self, lessAt: &'a LessAt, swapAt: &'a mut SwapAt) -> U32
    where
        LessAt: Fn( U32, U32) -> bool + 'a,
        SwapAt: FnMut( U32, U32) + 'a,
    {
        let  	mid = self.Mid();
        if lessAt( self._First, mid) {
            swapAt( self._First, mid);
        }
        let  	mut pivot = self._First;
        self.LSnip( 1).Traverse( &mut |i| {
            if lessAt( i, self._First) {
                pivot = pivot + 1;
                swapAt( pivot, i);
            }
        });
        if lessAt( pivot, self._First) {
            swapAt( self._First, pivot);
        }
        pivot
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	QSort< 'a, LessAt, SwapAt>( &self, lessAt: LessAt, mut swapAt: SwapAt)
    where
        LessAt: Fn( U32, U32) -> bool + 'a + Copy,
        SwapAt: FnMut( U32, U32) + 'a + Copy,
    {
        let  	mut currentSeg = *self;
        while currentSeg.Size() > 1 {
            let  	pivot = currentSeg.Partition( &lessAt, &mut swapAt);
            let  	useg1 = USeg::Create( currentSeg._First, pivot - currentSeg._First);
            let  	useg2 = USeg::Create( pivot + 1, currentSeg._Last - pivot);

            if useg1.Size() < useg2.Size() {
                if useg1.Size() > 1 {
                    useg1.QSort( lessAt, swapAt);
                }
                currentSeg = useg2;
            } else {
                if useg2.Size() > 1 {
                    useg2.QSort( lessAt, swapAt);
                }
                currentSeg = useg1;
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DoQSort< 'a, LessAt, SwapAt>( &self, worker: &dyn IWorker, lessAt: LessAt, swapAt: SwapAt)
    where
        LessAt: Fn( U32, U32) -> bool + Send + Sync + 'a + Copy,
        SwapAt: Fn( U32, U32) + Send + Sync + 'a + Copy,
    {
        let  	mut currentSeg = *self;
        while currentSeg.Size() > 1 {
            if currentSeg.Size() < U32( 32) {
                currentSeg.QSort( lessAt, swapAt);
                return;
            }
            let  	pivot = currentSeg.Partition( &lessAt, &mut |i, j| swapAt( i, j));
            let  	useg1 = USeg::Create( currentSeg._First, pivot - currentSeg._First);
            let  	useg2 = USeg::Create( pivot + 1, currentSeg._Last - pivot);

            if useg1.Size() > useg2.Size() {
                if useg1.Size() > 1 {
                    worker.Post( move |w: &dyn IWorker| {
                        useg1.DoQSort( w, lessAt, swapAt);
                    });
                }
                currentSeg = useg2;
            } else {
                if useg2.Size() > 1 {
                    worker.Post( move |w: &dyn IWorker| {
                        useg2.DoQSort( w, lessAt, swapAt);
                    });
                }
                currentSeg = useg1;
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}

//---------------------------------------------------------------------------------------------------------------------------------
