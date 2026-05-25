//-- silo/useg.rs ---------------------------------------------------------------------------------------------------------------------
 
use     crate::silo::uint32::UInt32;

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct USeg
{
    pub _First: u32,
    pub _Last: u32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl USeg
{
    pub fn Create( first: u32, sz: u32) -> Self
    {
        USeg
        {
            _First: first,
            _Last: (first +sz).wrapping_sub( 1),
        }
    }

    pub fn First( &self) -> u32
    {
        self._First
    }

    pub fn Last( &self) -> u32
    {
        self._Last
    }

    pub fn Mid( &self) -> u32
    {
        ( self._First + self._Last) /2
    }

    pub fn Size( &self) -> u32
    {
        if self._Last >= self._First
        {
            self._Last.wrapping_add( 1).wrapping_sub( self._First)
        }
        else
        {
            0
        }
    }

    pub fn IsEmpty( &self) -> bool
    {
        self.Size() == 0
    }

    pub fn LSnip( &self, count: u32) -> Self
    {
        if  self.Size() < count {
            USeg::Create( u32::MAX, 0)
        } else {
            USeg::Create( self._First + count, self.Size() -count)
        }
    }

    pub fn RSnip( &self, count: u32) -> Self
    {
        if  self.Size() < count {
            USeg::Create( u32::MAX, 0)
        } else {
            USeg::Create( self._First, self.Size() - count)
        }
    }

    pub fn Span<F>( &self, mut f: F) -> bool
        where
            F: FnMut( u32) -> bool,
    {
        if self.IsEmpty()
        {
            return true;
        }
        for i in self._First..=self._Last
        {
            if !f( i)
            {
                return false;
            }
        }
        true
    }

    fn Partition<LessAt, SwapAt>( &self, lessAt: &LessAt, swapAt: &mut SwapAt) -> u32
        where
            LessAt: Fn( u32, u32) -> bool, SwapAt: FnMut( u32, u32),
    {
        let     mid = self.Mid();
        if lessAt( self._First, mid)
        {
            swapAt( self._First, mid);
        }
        let mut pivot = self._First;
        self.LSnip( 1).Span( &mut |i|
        {
            if lessAt( i, self._First)
            {
                pivot += 1;
                swapAt( pivot, i);
            }
            true
        });
        if lessAt( pivot, self._First)
        {
            swapAt( self._First, pivot);
        }
        return pivot;
    }


    pub fn QSort<LessAt, SwapAt>( &self, lessAt: &LessAt, swapAt: &mut SwapAt)
        where
            LessAt: Fn( u32, u32) -> bool, SwapAt: FnMut( u32, u32),
    {

        let     pivot = self.Partition( lessAt, swapAt);

        // Recursively sort the two sub-arrays
        let     useg1 = USeg::Create( self._First, pivot - self._First);
        if  useg1.Size() > 1
        {
            useg1.QSort( lessAt, swapAt);
        }
        let     useg2 = USeg::Create( pivot + 1, self._Last - pivot);
        if useg2.Size() > 1
        {
            useg2.QSort( lessAt, swapAt);
        }
    }

}

//---------------------------------------------------------------------------------------------------------------------------------
