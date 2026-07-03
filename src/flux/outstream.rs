//-- outstream.rs ----------------------------------------------------------------------------------------------------------------------
use	std::{ cmp, io, slice::{ from_raw_parts, from_raw_parts_mut } };
use	crate::silo::{ Arr, Buff, IAccess, IArr, U8, U32 };
use	std::io::{ Result, Write };

//---------------------------------------------------------------------------------------------------------------------------------

pub enum OutSource< 'a, W: Write>
{
    Fixed( Arr< 'a, U8>),
    Streaming( W, Buff< U8>),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct OutStream< 'a, W: Write = io::Sink>
{
    _Source: OutSource< 'a, W>,
    _Marker: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> OutStream< 'a, io::Sink>
{
    pub fn	FromArr( arr: Arr< 'a, U8>) -> Self
    {
        Self {
            _Source: OutSource::Fixed( arr),
            _Marker: U32( 0),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, W: Write> OutStream< 'a, W>
{
    pub fn	New( inner: W, cacheSize: usize) -> Self
    {
        let  	buff = Buff::New( U32( cacheSize as u32), U8::_0);
        Self {
            _Source: OutSource::Streaming( inner, buff),
            _Marker: U32( 0),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Position( &self) -> U32
    {
        self._Marker
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SetPosition( &mut self, pos: U32)
    {
        self._Marker = pos;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, W: Write> Write for OutStream< 'a, W>
{
    fn	write( &mut self, buf: &[u8]) -> Result< usize>
    {
        let  	amt = buf.len();
        if amt == 0 {
            return Ok( 0);
        }

        match &mut self._Source {
            OutSource::Fixed( arr) => {
                let  	pos = self._Marker.AsUsize();
                let  	currSize = arr.Size().AsUsize();

                if pos >= currSize {
                    return Ok( 0);
                }

                let  	available = currSize - pos;
                let  	len = cmp::min( available, amt);

                unsafe {
                    let  	ptr = arr.Ptr().cast_mut().cast::<u8>();
                    let  	slice = from_raw_parts_mut( ptr, currSize);
                    slice[pos..pos + len].copy_from_slice( &buf[..len]);
                }

                self._Marker += U32( len as u32);
                Ok( len)
            },
            OutSource::Streaming( inner, buff) => {
                let  	mut pos = self._Marker.AsUsize();
                let  	cacheSize = buff.Size().AsUsize();
                let  	mut written = 0;

                while written < amt {
                    if pos == cacheSize {
                        // Flush cache
                        unsafe {
                            let  	ptr = buff.as_ptr().cast::<u8>();
                            let  	slice = from_raw_parts( ptr, cacheSize);
                            inner.write_all( slice)?;
                        }
                        pos = 0;
                        self._Marker = U32( 0);
                    }

                    let  	available = cacheSize - pos;
                    let  	len = cmp::min( available, amt - written);

                    unsafe {
                        let  	ptr = buff.as_mut_ptr().cast::<u8>();
                        let  	slice = from_raw_parts_mut( ptr, cacheSize);
                        slice[pos..pos + len].copy_from_slice( &buf[written..written + len]);
                    }

                    pos += len;
                    written += len;
                    self._Marker = U32( pos as u32);
                }

                Ok( written)
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	flush( &mut self) -> Result< ()>
    {
        match &mut self._Source {
            OutSource::Fixed( _) => Ok( ()),
            OutSource::Streaming( inner, buff) => {
                let  	pos = self._Marker.AsUsize();
                if pos > 0 {
                    unsafe {
                        let  	ptr = buff.as_ptr().cast::<u8>();
                        let  	slice = from_raw_parts( ptr, pos);
                        inner.write_all( slice)?;
                    }
                    self._Marker = U32( 0);
                }
                inner.flush()
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, W: Write> Drop for OutStream< 'a, W>
{
    fn	drop( &mut self)
    {
        let  	_ = self.flush();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
