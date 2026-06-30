//-- jsonoutstrm.rs -------------------------------------------------------------------------------------------------------------------

use	crate::flux::xflux::{ IXFlux, XField };
use	crate::silo::U32;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JsonOutStream< W: std::fmt::Write>
{
    _OStr: W,
    _Depth: U32,
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
            _Depth: U32( 0),
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
            for _ in 0..(self._Depth.0 * 2) {
                write!( self._OStr, " ")?;
            }
        } else {
            write!( self._OStr, " ")?;
        }
        Ok(())
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	KeyField( &mut self, key: &str, value: XField< '_>) -> bool
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

impl< W: std::fmt::Write> IXFlux for JsonOutStream< W>
{
    fn	Field( &mut self, field: XField) 
    { 
        match field {
            XField::Str( s) => { let  	_ = write!( self._OStr, "\"{}\"", s); },
            XField::String( s) => { let  	_ = write!( self._OStr, "\"{}\"", s); },
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
            XField::Arr( mut arr_func) => {
                let  	_ = write!( self._OStr, "[");
                let  	mut is_first = true;
                let  	mut item = XField::Null;
                while arr_func( &mut item) {
                    if !is_first {
                        let  	_ = write!( self._OStr, ", "); 
                    }
                    let  	mut next_item = XField::Null;
                    std::mem::swap( &mut item, &mut next_item);
                    self.Field( next_item);
                    is_first = false;
                }
                let  	_ = write!( self._OStr, "]");
            },
            XField::Obj( mut obj_func) => {
                let  	_ = write!( self._OStr, "{{");
                self._Depth += 1;
                
                self._EntryFlg = false;

                let  	mut key = String::new();
                let  	mut item = XField::Null;
                while obj_func( &mut key, &mut item) {
                    let  	mut next_item = XField::Null;
                    std::mem::swap( &mut item, &mut next_item);
                    self.KeyField( &key, next_item);
                    key.clear();
                }
                if self._Depth > 0 { self._Depth -= 1; }
                self._EntryFlg = false;
                let  	_ = self.LineFeed();
                self._EntryFlg = true;
                let  	_ = write!( self._OStr, "}}");
            },
            XField::Fluxable( f) => {
                let  	mut field = XField::Null;
                f.ToXFlux( &mut field);
                self.Field( field);
            },
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
