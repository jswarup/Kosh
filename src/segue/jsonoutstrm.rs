//-- jsonoutstrm.rs -------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ IAccess, U32 };

//---------------------------------------------------------------------------------------------------------------------------------

pub enum JsonValue< 'a>
{
    Str( &'a str),
    U32( u32),
    F64( f64),
    Bool( bool),
    Null,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait JsonListener< 'a>
{
    fn	KeyValue( &mut self, _key: &str, _value: JsonValue< 'a>) -> bool
    {
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	OpenArray( &mut self, _key: &str) -> bool
    {
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	CloseArray( &mut self) -> bool
    {
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	OpenObject( &mut self, _key: &str) -> bool
    {
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	CloseObject( &mut self) -> bool
    {
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Array< T: 'a, A, F>( &mut self, key: &str, access: &A, mut f: F) -> bool
    where
        Self: Sized,
        A: IAccess< 'a, T>,
        F: FnMut( &'a T) -> JsonValue< 'a>,
    {
        let mut res = self.OpenArray( key);
        let empty_key = "";
        for i in 0..access.Size().0 {
            res = res && self.KeyValue( empty_key, f( access.At( U32( i))));
        }
        res && self.CloseArray()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JsonOutStream< 'a, W: std::fmt::Write>
{
    _OStr: W,
    _Depth: u32,
    _EntryFlg: bool,
    _MultiLineFlg: bool,
    _marker: std::marker::PhantomData< &'a ()>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, W: std::fmt::Write> JsonOutStream< 'a, W>
{
    pub fn	New( ostr: W, multiLineFlg: bool) -> Self
    {
        Self {
            _OStr: ostr,
            _Depth: 0,
            _EntryFlg: false,
            _MultiLineFlg: multiLineFlg,
            _marker: std::marker::PhantomData,
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	LineFeed( &mut self) -> std::fmt::Result
    {
        if self._EntryFlg {
            write!( self._OStr, ",")?;
        }
        if self._MultiLineFlg {
            write!( self._OStr, "\n")?;
            for _ in 0..(self._Depth * 2) {
                write!( self._OStr, " ")?;
            }
        } else {
            write!( self._OStr, " ")?;
        }
        Ok(())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, W: std::fmt::Write> JsonListener< 'a> for JsonOutStream< 'a, W>
{
    fn	KeyValue( &mut self, key: &str, value: JsonValue< 'a>) -> bool
    {
        let _ = self.LineFeed();
        self._EntryFlg = true;
        
        if !key.is_empty() {
            let _ = write!( self._OStr, "\"{}\": ", key);
        }
        
        match value {
            JsonValue::Str( s) => { let _ = write!( self._OStr, "\"{}\"", s); },
            JsonValue::U32( n) => { let _ = write!( self._OStr, "{}", n); },
            JsonValue::F64( f) => { 
                if f.is_nan() || f.is_infinite() {
                    let _ = write!( self._OStr, "\"null\"");
                } else {
                    let _ = write!( self._OStr, "{}", f);
                }
            },
            JsonValue::Bool( b) => { let _ = write!( self._OStr, "{}", if b { "true" } else { "false" }); },
            JsonValue::Null => { let _ = write!( self._OStr, "\"null\""); },
        }
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	OpenArray( &mut self, key: &str) -> bool
    {
        let _ = self.LineFeed();
        self._EntryFlg = false;
        if !key.is_empty() {
            let _ = write!( self._OStr, "\"{}\": ", key);
        }
        let _ = write!( self._OStr, "[ ");
        self._Depth += 1;
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	CloseArray( &mut self) -> bool
    {
        if self._Depth > 0 { self._Depth -= 1; }
        self._EntryFlg = false;
        let _ = self.LineFeed();
        self._EntryFlg = true;
        let _ = write!( self._OStr, "]");
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	OpenObject( &mut self, key: &str) -> bool
    {
        let _ = self.LineFeed();
        self._EntryFlg = false;
        if !key.is_empty() {
            let _ = write!( self._OStr, "\"{}\": ", key);
        }
        let _ = write!( self._OStr, "{{");
        self._Depth += 1;
        true
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	CloseObject( &mut self) -> bool
    {
        if self._Depth > 0 { self._Depth -= 1; }
        self._EntryFlg = false;
        let _ = self.LineFeed();
        self._EntryFlg = true;
        let _ = write!( self._OStr, "}}");
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
