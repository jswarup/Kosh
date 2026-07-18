//-- leaves.rs -------------------------------------------------------------------------------------------------------------------------


use	crate::shard::Parser;
use	crate::flux::{ IFluxImportSource };
use	crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use	crate::flux::fluximport::FieldImp;
use	crate::shard::{ Charset, IGrammar };
use	crate::silo::{ U32, U8 };

//---------------------------------------------------------------------------------------------------------------------------------


//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'r, T: IGrammar + ?Sized> IGrammar for &'r T
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        (**self).Match( parser)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Charset
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	mark = parser.CurrMark();
        let  	curr = parser.GetAt( mark);
        if self.Get( curr.0) {
            parser.SetCurrMark( mark + U32( 1));
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for char
{
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for char
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	mark = parser.CurrMark();
        let  	curr = parser.GetAt( mark);
        if curr == U8( *self as u8) {
            parser.SetCurrMark( mark + U32( 1));
            true
        } else {
            false
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for str
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	mark = parser.CurrMark();
        let  	key = self.as_bytes();
        let  	mut currentMark = mark;

        for &b in key {
            let  	stream = parser.InStream();
            let  	curr = stream.At( currentMark);
            if curr.0 != b {
                return false;
            }
            if let  	Some( next) = parser.Incr( currentMark) {
                currentMark = next;
            } else {
                return false;
            }
        }

        parser.SetCurrMark( currentMark);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Str
{
}

pub const Str: Str = Str {};

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxExportSource for Str
{
    fn	FetchFieldExp ( &self, _field: &mut FieldExp )
    {
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxImportSource for Str {
    fn FetchFieldImp(&mut self, _field: &mut FieldImp) {
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Str
{
    fn	Match( &self, parser: &mut Parser) -> bool
    {
        let  	mark = parser.CurrMark();
        let  	mut m = mark;
        let  	curr = parser.GetAt( m);
        if curr != U8( b'"') {
            return false;
        }

        if let  	Some( next) = parser.Incr( m) {
            m = next;
            let  	mut escape = false;
            loop {
                let  	c = parser.GetAt( m);
                if c == U8( 0) && m >= parser.InStream().Size() {
                    return false;
                }

                if escape {
                    escape = false;
                } else if c == U8( b'\\') {
                    escape = true;
                } else if c == U8( b'"') {
                    if let  	Some( nxt) = parser.Incr( m) { 
                        parser.SetCurrMark( nxt);
                        return true;
                    } else {
                        return false;
                    }
                }

                if let  	Some( nxt) = parser.Incr( m) {
                    m = nxt;
                } else {
                    return false;
                }
            }
        }
        false
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxImportSource!( char);

//---------------------------------------------------------------------------------------------------------------------------------
