//-- instream.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Arr, Buff, IAccess, IArr, U8, U32, U64 };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct InStream< 'a>
{
    _Arr: Arr< 'a, U8>,
    _Marker: U32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> InStream< 'a>
{
    pub fn	New( arr: Arr< 'a, U8>) -> Self
    {
        Self {
            _Arr: arr,
            _Marker: U32( 0),
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Curr( &self) -> U8
    {
        if self._Marker.AsUsize() < self._Arr.Size().AsUsize() {
            *self._Arr.At( self._Marker)
        } else {
            U8::_0
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Next( &mut self) -> bool
    {
        self._Marker += U32( 1);
        self._Marker.AsUsize() < self._Arr.Size().AsUsize()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	RollTo( &mut self, val: U32)
    {
        self._Marker = val;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Rest( &self) -> Arr< 'a, U8>
    {
        let  	sz = self._Arr.Size();
        if self._Marker < sz {
            self._Arr.LSnip( self._Marker)
        } else {
            self._Arr.LSnip( sz)
        }
    }


    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	RemainingBytes( &self) -> &'a [u8]
    {
        let  	rest = self.Rest();
        unsafe {
            std::slice::from_raw_parts( rest.Ptr().cast::<u8>(), rest.Size().AsUsize())
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DeserializeNext< T>( &mut self) -> Result< T, Box< bincode::ErrorKind>>
    where
        T: serde::de::DeserializeOwned
    {
        let  	bytes = self.RemainingBytes();
        let  	mut cursor = std::io::Cursor::new( bytes);
        let  	result: T = bincode::deserialize_from( &mut cursor)?; 
        self.RollTo( self._Marker + U64( cursor.position()).ExplodeToU32()[ 0]);
        Ok( result)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct InBuffStream
{
    _Buff: Buff< U8>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl InBuffStream
{
    pub fn	FromFile< P: AsRef< std::path::Path>>( path: P) -> std::io::Result< Self>
    {
        let  	bytes = std::fs::read( path)?;
        let  	mut buff = Buff::NewEmpty();
        for b in bytes {
            buff.Push( U8( b));
        }
        Ok( Self { _Buff: buff })
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	FromStdin() -> std::io::Result< Self>
    {
        use std::io::Read;
        let  	mut bytes = Vec::new();
        std::io::stdin().read_to_end( &mut bytes)?;
        let  	mut buff = Buff::NewEmpty();
        for b in bytes {
            buff.Push( U8( b));
        }
        Ok( Self { _Buff: buff })
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Stream( &self) -> InStream< '_>
    {
        InStream::New( self._Buff.Arr())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
