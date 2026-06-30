//-- instream.rs -----------------------------------------------------------------------------------------------------------------------
use	std::{ cmp, fs, io, path::Path, slice::{ from_raw_parts, from_raw_parts_mut } };
use	crate::silo::{ Arr, Buff, IAccess, IArr, U8, U32 };
use std::io::Read;

//---------------------------------------------------------------------------------------------------------------------------------

pub enum InSource< 'a, R: Read>
{
    Fixed( Arr< 'a, U8>),
    Streaming( R, Buff< U8>),
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct InStream< 'a, R: Read = io::Empty>
{
    _Source: InSource< 'a, R>,
    _Marker: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> InStream< 'a, io::Empty>
{
    pub fn	FromArr( arr: Arr< 'a, U8>) -> Self
    {
        Self {
            _Source: InSource::Fixed( arr),
            _Marker: U32( 0),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, R: Read> InStream< 'a, R>
{
    pub fn	New( inner: R) -> Self
    {
        Self {
            _Source: InSource::Streaming( inner, Buff::NewEmpty()),
            _Marker: U32( 0),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> InStream< 'a, fs::File>
{
    pub fn	FromFile< P: AsRef< Path>>( path: P) -> io::Result< Self>
    {
        let  	file = fs::File::open( path)?;
        Ok( Self::New( file))
    }

    pub fn	FromFileHandle( file: fs::File) -> io::Result< Self>
    {
        Ok( Self::New( file))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> InStream< 'a, io::Stdin>
{
    pub fn	FromStdin() -> io::Result< Self>
    {
        Ok( Self::New( io::stdin()))
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, R: Read> InStream< 'a, R>
{
    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EnsureCached( &mut self, amt: usize) -> io::Result< ()>
    {
        if let InSource::Streaming( ref mut inner, ref mut buff) = self._Source {
            let  	required = self._Marker.AsUsize() + amt;
            let  	mut currSize = buff.Size().AsUsize();

            while currSize < required {
                let  	chunkSize = cmp::max( 4096, required - currSize);
                let  	mut chunk = vec![ 0u8; chunkSize];
                let  	readBytes = inner.read( &mut chunk)?;
                
                if readBytes == 0 {
                    break;
                }
                
                let  	newSize = currSize + readBytes;
                buff.Resize( U32( newSize as u32), |_| U8::_0);
                
                unsafe {
                    let  	ptr = buff.as_mut_ptr().cast::<u8>();
                    let  	slice = from_raw_parts_mut( ptr, newSize);
                    slice[currSize..newSize].copy_from_slice( &chunk[..readBytes]);
                }
                currSize = newSize;
            }
        }
        Ok( ())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Size( &self) -> usize
    {
        match &self._Source {
            InSource::Fixed( arr) => arr.Size().AsUsize(),
            InSource::Streaming( _, buff) => buff.Size().AsUsize(),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Curr( &mut self) -> U8
    {
        let  	_ = self.EnsureCached( 1);
        if self._Marker.AsUsize() < self.Size() {
            match &self._Source {
                InSource::Fixed( arr) => *arr.At( self._Marker),
                InSource::Streaming( _, buff) => *buff.At( self._Marker),
            }
        } else {
            U8::_0
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Next( &mut self) -> bool
    {
        self._Marker += U32( 1);
        let  	_ = self.EnsureCached( 1);
        self._Marker.AsUsize() < self.Size()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	RollTo( &mut self, mark: U32)
    {
        self._Marker = mark;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Marker( &self) -> U32
    {
        self._Marker
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Rest( &mut self) -> Arr< '_, U8>
    {
        if let InSource::Streaming( ref mut inner, ref mut buff) = self._Source {
            let  	mut vec = Vec::new();
            if inner.read_to_end( &mut vec).is_ok() && !vec.is_empty() {
                let  	currSize = buff.Size().AsUsize();
                let  	newSize = currSize + vec.len();
                buff.Resize( U32( newSize as u32), |_| U8::_0);
                unsafe {
                    let  	ptr = buff.as_mut_ptr().cast::<u8>();
                    let  	slice = from_raw_parts_mut( ptr, newSize);
                    slice[currSize..newSize].copy_from_slice( &vec);
                }
            }
        }
        
        let  	sz = self.Size();
        let  	arr = match &self._Source {
            InSource::Fixed( arr) => *arr,
            InSource::Streaming( _, buff) => buff.Arr(),
        };

        if self._Marker.AsUsize() < sz {
            arr.LSnip( self._Marker)
        } else {
            arr.LSnip( U32( sz as u32))
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	RemainingBytes( &mut self) -> &[u8]
    {
        let  	rest = self.Rest();
        unsafe {
            from_raw_parts( rest.Ptr().cast::<u8>(), rest.Size().AsUsize())
        }
    }


}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, R: Read> Read for InStream< 'a, R>
{
    fn	read( &mut self, buf: &mut [u8]) -> io::Result< usize>
    {
        let  	amt = buf.len();
        if amt == 0 {
            return Ok( 0);
        }

        self.EnsureCached( amt)?;

        let  	currSize = self.Size();
        let  	marker = self._Marker.AsUsize();

        if marker >= currSize {
            return Ok( 0);
        }

        let  	available = currSize - marker;
        let  	len = cmp::min( available, amt);

        match &self._Source {
            InSource::Fixed( arr) => {
                unsafe {
                    let  	ptr = arr.Ptr().cast::<u8>();
                    let  	slice = from_raw_parts( ptr, currSize);
                    buf[..len].copy_from_slice( &slice[marker..marker + len]);
                }
            },
            InSource::Streaming( _, buff) => {
                unsafe {
                    // buff implements Deref<Target=[U8]>, as_ptr returns *const U8
                    let  	ptr = buff.as_ptr().cast::<u8>();
                    let  	slice = from_raw_parts( ptr, currSize);
                    buf[..len].copy_from_slice( &slice[marker..marker + len]);
                }
            }
        }

        self._Marker += U32( len as u32);
        Ok( len)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
