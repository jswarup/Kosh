//-- silo/useg.rs ---------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct USeg
{
    pub _First: u32,
    pub _Last: u32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl USeg
{
    pub fn New(first: u32, last: u32) -> Self
    {
        USeg
        {
            _First: first,
            _Last: last,
        }
    }

    pub fn First(&self) -> u32
    {
        self._First
    }

    pub fn Last(&self) -> u32
    {
        self._Last
    }

    pub fn Len(&self) -> u32
    {
        if self._Last >= self._First
        {
            self._Last - self._First + 1
        }
        else
        {
            0
        }
    }

    pub fn IsEmpty(&self) -> bool
    {
        self.Len() == 0
    }

    pub fn LSnip(&self, count: u32) -> Self
    {
        if count >= self.Len()
        {
            USeg::New(1, 0)
        }
        else
        {
            USeg::New(self._First + count, self._Last)
        }
    }

    pub fn RSnip(&self, count: u32) -> Self
    {
        if count >= self.Len()
        {
            USeg::New(1, 0)
        }
        else
        {
            USeg::New(self._First, self._Last - count)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
