//-- jsonoutstrm.rs -------------------------------------------------------------------------------------------------------------------
use	crate::segue::{ JsonListener, JsonValue };
use	crate::segue::xflux::{ IXFlux, XField };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JsonOutStream< W: std::fmt::Write>
{
    _OStr: W,
    _Depth: u32,
    _EntryFlg: bool,
    _MultiLineFlg: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< W: std::fmt::Write> JsonOutStream< W>
{
    pub fn	New( ostr: W, multiLineFlg: bool) -> Self
    {
        Self {
            _OStr: ostr,
            _Depth: 0,
            _EntryFlg: false,
            _MultiLineFlg: multiLineFlg,
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

impl< W: std::fmt::Write> JsonListener for JsonOutStream< W>
{
    fn	KeyValue( &mut self, key: &str, value: JsonValue< '_>) -> bool
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

impl< W: std::fmt::Write> IXFlux for JsonOutStream< W>
{
    fn	OStream( &mut self) -> &mut dyn std::fmt::Write
    {
        &mut self._OStr
    }
 
    fn	Field( &mut self, field: XField) 
    { 
        match field {
            XField::Str( s) => { let  	_ = write!( self._OStr, "\"{}\"", s); },
            XField::U64( n) => { let  	_ = write!( self._OStr, "{}", n); },
            XField::F64( f) => { 
                if f.is_nan() || f.is_infinite() {
                    let  	_ = write!( self._OStr, "\"null\"");
                } else {
                    let  	_ = write!( self._OStr, "{}", f);
                }
            },
            XField::Bool( b) => { let  	_ = write!( self._OStr, "{}", if b { "true" } else { "false" }); },
            XField::Null => { let  	_ = write!( self._OStr, "\"null\""); },
            XField::Arr( arr_func) => {
                let  	_ = write!( self._OStr, "[");
                let  	mut is_first = true;
                let  	mut item = XField::Null;
                while arr_func( &mut item) {
                    if !is_first { let  	_ = write!( self._OStr, ", "); }
                    let  	mut next_item = XField::Null;
                    std::mem::swap( &mut item, &mut next_item);
                    self.Field( next_item);
                    is_first = false;
                }
                let  	_ = write!( self._OStr, "]");
            },
        }
    }
 
    fn	KeyField( &mut self, key: &str, value: XField< '_>) -> bool
    {
        let  	_ = self.LineFeed();
        self._EntryFlg = true;
        
        if !key.is_empty() {
            let  	_ = write!( self._OStr, "\"{}\": ", key);
        }
        
        self.Field( value);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
