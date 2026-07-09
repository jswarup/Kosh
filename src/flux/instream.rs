//-- instream.rs -----------------------------------------------------------------------------------------------------------------------
use	std::{ cmp, fs, io, path::Path };
use	crate::silo::{ Arr, Buff, IAccess, U8, U32, cast::ICastExt };
use std::io::Read;

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IStream {
    fn Size(&self) -> usize;
    fn Curr(&mut self) -> U8;
    fn Next(&mut self) -> bool;
    fn RollTo(&mut self, mark: U32);
    fn Marker(&self) -> U32;
    fn At(&mut self, offset: U32) -> U8;
    fn Bytes(&mut self, count: usize) -> &[u8];
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct FixedStream<'a> {
    _Arr: Arr<'a, U8>,
    _Marker: U32,
}

impl<'a> From<Arr<'a, U8>> for FixedStream<'a> {
    fn from(arr: Arr<'a, U8>) -> Self {
        Self {
            _Arr: arr,
            _Marker: U32(0),
        }
    }
}

impl<'a> From<&'a str> for FixedStream<'a> {
    fn from(strVal: &'a str) -> Self {
        Self::from(Arr::from(strVal))
    }
}

impl<'a> IStream for FixedStream<'a> {
    fn Size(&self) -> usize {
        self._Arr.Size().AsUsize()
    }

    fn Curr(&mut self) -> U8 {
        if self._Marker.AsUsize() < self.Size() {
            *self._Arr.At(self._Marker)
        } else {
            U8::_0
        }
    }

    fn Next(&mut self) -> bool {
        self._Marker += U32(1);
        self._Marker.AsUsize() < self.Size()
    }

    fn RollTo(&mut self, mark: U32) {
        self._Marker = mark;
    }

    fn Marker(&self) -> U32 {
        self._Marker
    }

    fn At(&mut self, offset: U32) -> U8 {
        if offset.AsUsize() < self.Size() {
            *self._Arr.At( offset)
        } else {
            U8::_0
        }
    }

    fn Bytes(&mut self, count: usize) -> &[u8] {
        let  	sz = self.Size();
        let  	slice = (&*self._Arr).Cast::<&[u8]>();
        let  	mark = self._Marker.AsUsize();
        if mark < sz {
            let  	end = cmp::min( mark + count, sz);
            &slice[mark..end]
        } else {
            &[]
        }
    }
}

impl<'a> Read for FixedStream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = buf.len();
        if amt == 0 { return Ok(0); }
        let currSize = self.Size();
        let marker = self._Marker.AsUsize();
        if marker >= currSize { return Ok(0); }
        let available = currSize - marker;
        let len = cmp::min(available, amt);
        let slice = (&*self._Arr).Cast::<&[u8]>();
        buf[..len].copy_from_slice(&slice[marker..marker+len]);
        self._Marker += U32(len as u32);
        Ok(len)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BuffStream<R: Read> {
    _Inner: R,
    _Buff: Buff<U8>,
    _Marker: U32,
}

impl<R: Read> From<R> for BuffStream<R> {
    fn from(inner: R) -> Self {
        Self {
            _Inner: inner,
            _Buff: Buff::NewEmpty(),
            _Marker: U32(0),
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
    fn EnsureCached(&mut self, amt: usize) -> io::Result<()> {
        let required = self._Marker.AsUsize() + amt;
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
    fn Size(&self) -> usize {
        self._Buff.Size().AsUsize()
    }

    fn Curr(&mut self) -> U8 {
        let _ = self.EnsureCached(1);
        if self._Marker.AsUsize() < self.Size() {
            *self._Buff.Arr().At(self._Marker)
        } else {
            U8::_0
        }
    }

    fn Next(&mut self) -> bool {
        self._Marker += U32(1);
        let _ = self.EnsureCached(1);
        self._Marker.AsUsize() < self.Size()
    }

    fn RollTo(&mut self, mark: U32) {
        self._Marker = mark;
    }

    fn Marker(&self) -> U32 {
        self._Marker
    }

    fn At(&mut self, offset: U32) -> U8 {
        let _ = self.EnsureCached( offset.AsUsize() + 1);
        if offset.AsUsize() < self.Size() {
            *self._Buff.Arr().At( offset)
        } else {
            U8::_0
        }
    }

    fn Bytes(&mut self, count: usize) -> &[u8] {
        let _ = self.EnsureCached( count);
        let  	sz = self.Size();
        let  	slice = (&*self._Buff).Cast::<&[u8]>();
        let  	mark = self._Marker.AsUsize();
        if mark < sz {
            let  	end = cmp::min( mark + count, sz);
            &slice[mark..end]
        } else {
            &[]
        }
    }
}

impl<R: Read> Read for BuffStream<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let amt = buf.len();
        if amt == 0 { return Ok(0); }
        self.EnsureCached(amt)?;
        let currSize = self.Size();
        let marker = self._Marker.AsUsize();
        if marker >= currSize { return Ok(0); }
        let available = currSize - marker;
        let len = cmp::min(available, amt);
        let slice = (&*self._Buff).Cast::<&[u8]>();
        buf[..len].copy_from_slice(&slice[marker..marker+len]);
        self._Marker += U32(len as u32);
        Ok(len)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
