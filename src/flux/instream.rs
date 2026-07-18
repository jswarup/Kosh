//-- instream.rs -----------------------------------------------------------------------------------------------------------------------
use	std::{ cmp, fs, io, path::Path };
use	crate::silo::{ Arr, Buff, IAccess, IArr, U8, U32, cast::ICastExt };
use std::io::Read;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IStream {
    fn Size(&self) -> U32;
    fn At(&mut self, offset: U32) -> U8;
    fn BytesAt(&mut self, offset: U32, count: U32) -> Arr<'_, U8>;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct FixedStream<'a> {
    _Arr: Arr<'a, U8>,
}

impl<'a> From<Arr<'a, U8>> for FixedStream<'a> {
    fn from(arr: Arr<'a, U8>) -> Self {
        Self {
            _Arr: arr,
        }
    }
}

impl<'a> From<&'a str> for FixedStream<'a> {
    fn from(strVal: &'a str) -> Self {
        Self::from(Arr::from(strVal))
    }
}

impl<'a> IStream for FixedStream<'a> {
    fn Size(&self) -> U32 {
        self._Arr.Size()
    }

    fn At(&mut self, offset: U32) -> U8 {
        if offset < self.Size() {
            *self._Arr.At( offset)
        } else {
            U8::_0
        }
    }

    fn BytesAt(&mut self, offset: U32, count: U32) -> Arr<'_, U8> {
        let  	sz = self.Size();
        let  	start = offset.AsUsize();
        if offset < sz {
            let  	end = cmp::min( start + count.AsUsize(), sz.AsUsize());
            self._Arr.Subset( offset, U32((end - start) as u32))
        } else {
            let  	empty: &[U8] = &[];
            Arr::from( empty)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BuffStream<R: Read> {
    _Inner: R,
    _Buff: Buff<U8>,
}

impl<R: Read> From<R> for BuffStream<R> {
    fn from(inner: R) -> Self {
        Self {
            _Inner: inner,
            _Buff: Buff::NewEmpty(),
        }
    }
}

impl TryFrom<&Path> for BuffStream<fs::File> {
    type Error = io::Error;
    fn try_from(path: &Path) -> io::Result<Self> {
        let file = fs::File::open(path)?;
        Ok(Self::from(file))
    }
}

impl TryFrom<&str> for BuffStream<fs::File> {
    type Error = io::Error;
    fn try_from(path: &str) -> io::Result<Self> {
        let file = fs::File::open(path)?;
        Ok(Self::from(file))
    }
}

impl BuffStream<fs::File> {
    pub fn FromFile<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::try_from(path.as_ref())
    }

    pub fn FromPath(path: &Path) -> io::Result<Self> {
        Self::try_from(path)
    }

    pub fn FromFileHandle(file: fs::File) -> io::Result<Self> {
        Ok(Self::from(file))
    }
}

impl BuffStream<io::Stdin> {
    pub fn FromStdin() -> io::Result<Self> {
        Ok(Self::from(io::stdin()))
    }
}

impl<R: Read> BuffStream<R> {
    fn EnsureCached(&mut self, required: usize) -> io::Result<()> {
        let mut currSize = self._Buff.Size().AsUsize();

        while currSize < required {
            let chunkSize = cmp::max(4096, required - currSize);
            let mut chunk = vec![0u8; chunkSize];
            let readBytes = self._Inner.read(&mut chunk)?;

            if readBytes == 0 {
                break;
            }

            let newSize = currSize + readBytes;
            self._Buff.Resize(U32(newSize as u32), |_| U8::_0);

            let slice = (&mut *self._Buff).Cast::<&mut [u8]>();
            slice[currSize..newSize].copy_from_slice(&chunk[..readBytes]);
            currSize = newSize;
        }

        Ok(())
    }
}

impl<R: Read> IStream for BuffStream<R> {
    fn Size(&self) -> U32 {
        self._Buff.Size()
    }

    fn At(&mut self, offset: U32) -> U8 {
        let _ = self.EnsureCached( offset.AsUsize() + 1);
        if offset < self.Size() {
            *self._Buff.Arr().At( offset)
        } else {
            U8::_0
        }
    }

    fn BytesAt(&mut self, offset: U32, count: U32) -> Arr<'_, U8> {
        let _ = self.EnsureCached( offset.AsUsize() + count.AsUsize());
        let  	sz = self.Size();
        let  	start = offset.AsUsize();
        if offset < sz {
            let  	end = cmp::min( start + count.AsUsize(), sz.AsUsize());
            self._Buff.Arr().Subset( offset, U32((end - start) as u32))
        } else {
            let  	empty: &[U8] = &[];
            Arr::from( empty)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
