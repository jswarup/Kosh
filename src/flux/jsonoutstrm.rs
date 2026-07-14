//-- jsonoutstrm.rs -------------------------------------------------------------------------------------------------------------------
use	std::{ fmt, mem::swap };

use	crate::flux::fluxout::{ IFluxOutSink, FieldOut };
use	crate::silo::U32;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct JsonOutStream< W: fmt::Write>
{
    _OStr: W,
    _Depth: U32,
    _EntryFlg: bool,
    _MultiLineFlg: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< W: fmt::Write> JsonOutStream< W>
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

    fn	LineFeed( &mut self) -> fmt::Result
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

    pub fn	KeyField( &mut self, key: &str, value: FieldOut< '_>) -> bool
    {
        if matches!( value, FieldOut::Null) {
            return false;
        }
        let  	_ = self.LineFeed();
        self._EntryFlg = true;

        if !key.is_empty() {
            let  	_ = write!( self._OStr, "\"{}\": ", key);
        }

        self.FromFieldOut( value);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< W: fmt::Write> IFluxOutSink for JsonOutStream< W>
{
    fn	FromFieldOut( &mut self, field: FieldOut)
    {
        match field {
            FieldOut::Str( s) => { let  	_ = write!( self._OStr, "\"{}\"", s); },
            FieldOut::String( s) => { let  	_ = write!( self._OStr, "\"{}\"", s); },
            FieldOut::U64( n) => { let  	_ = write!( self._OStr, "{}", n); },
            FieldOut::F64( f) => {
                if f.is_nan() || f.is_infinite() {
                    let  	_ = write!( self._OStr, "\"null\"");
                } else {
                    let  	_ = write!( self._OStr, "{}", f);
                }
            },
            FieldOut::Bool( b) => { let  	_ = write!( self._OStr, "{}", if b { "true" } else { "false" }); },
            FieldOut::Null => { let  	_ = write!( self._OStr, "\"null\""); },
            FieldOut::Arr( mut arr_func) => {
                let  	_ = write!( self._OStr, "[");
                let  	mut is_first = true;
                let  	mut item = FieldOut::Null;
                while arr_func( &mut item) {
                    if !is_first {
                        let  	_ = write!( self._OStr, ", ");
                    }
                    let  	mut next_item = FieldOut::Null;
                    swap( &mut item, &mut next_item);
                    self.FromFieldOut( next_item);
                    is_first = false;
                }
                let  	_ = write!( self._OStr, "]");
            },
            FieldOut::Obj( mut obj_func) => {
                let  	_ = write!( self._OStr, "{{");
                self._Depth += 1;

                self._EntryFlg = false;

                let  	mut key = String::new();
                let  	mut item = FieldOut::Null;
                while obj_func( &mut key, &mut item) {
                    let  	mut next_item = FieldOut::Null;
                    swap( &mut item, &mut next_item);
                    self.KeyField( &key, next_item);
                    key.clear();
                }
                if self._Depth > 0 { self._Depth -= 1; }
                self._EntryFlg = false;
                let  	_ = self.LineFeed();
                self._EntryFlg = true;
                let  	_ = write!( self._OStr, "}}");
            },
            FieldOut::FluxSource( f) => {
                let  	mut field = FieldOut::Null;
                f.ToFieldOut( &mut field);
                self.FromFieldOut( field);
            },
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
