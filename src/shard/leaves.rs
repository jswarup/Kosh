//-- leaves.rs -------------------------------------------------------------------------------------------------------------------------


use	crate::shard::Parser;
use	crate::flux::{ IFluxImportSource };
use	crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use	crate::flux::fluximport::FieldImp;
use	crate::shard::{ Charset, IGrammar, IForge };
use	crate::silo::{ U32, U8 };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct StrShard< 'a>
{
    pub _Val: &'a str,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IFluxExportSource for StrShard< 'a>
{
    fn	FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
    {
        *field = FieldExp::Str( self._Val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl<'a> IFluxImportSource for StrShard<'a> {
    fn FetchFieldImp<'b>(&'b mut self, field: &mut FieldImp<'b>) {
        IFluxImportSource::FetchFieldImp(&mut self._Val, field);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for StrShard< 'a>
{

    fn	Match( &self, parser: &mut Parser)
    {
        self._Val.Match( parser);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a, 'r, T: IGrammar + ?Sized> IGrammar for &'r T
{
    fn	Match( &self, parser: &mut Parser)
    {
        (**self).Match( parser);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for Charset
{
    fn	Match( &self, parser: &mut Parser)
    {
        let  	mark = parser.CurrMark();
        let  	curr = parser.GetAt( mark);
        if self.Get( curr.0) {
            let  	res = Some( mark + U32( 1));
            parser.Deposit( res);
        } else {
            parser.Deposit( None);
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
    fn	Match( &self, parser: &mut Parser)
    {
        let  	mark = parser.CurrMark();
        let  	curr = parser.GetAt( mark);
        if curr == U8( *self as u8) {
            let  	res = Some( mark + U32( 1));
            parser.Deposit( res);
        } else {
            parser.Deposit( None);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for str
{
    fn	Match( &self, parser: &mut Parser)
    {
        let  	mark = parser.CurrMark();
        let  	key = self.as_bytes();
        let  	mut currentMark = mark;

        for &b in key {
            let  	stream = parser.InStream();
            let  	curr = stream.At( currentMark);
            if curr.0 != b {
                parser.Deposit( None);
                return;
            }
            if let  	Some( next) = parser.Incr( currentMark) {
                currentMark = next;
            } else {
                parser.Deposit( None);
                return;
            }
        }

        parser.Deposit( Some( currentMark));
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
    fn	Match( &self, parser: &mut Parser)
    {
        let  	mark = parser.CurrMark();
        let  	mut m = mark;
        let  	curr = parser.GetAt( m);
        if curr != U8( b'"') {
            parser.Deposit( None);
            return;
        }

        if let  	Some( next) = parser.Incr( m) {
            m = next;
            let  	mut escape = false;
            loop {
                let  	c = parser.GetAt( m);
                if c == U8( 0) && m >= parser.InStream().Size() {
                    parser.Deposit( None);
                    return;
                }

                if escape {
                    escape = false;
                } else if c == U8( b'\\') {
                    escape = true;
                } else if c == U8( b'"') {
                    if let  	Some( nxt) = parser.Incr( m) { 
                        parser.Deposit( Some( nxt));
                        return;
                    } else {
                        parser.Deposit( None);
                        return;
                    }
                }

                if let  	Some( nxt) = parser.Incr( m) {
                    m = nxt;
                } else {
                    parser.Deposit( None);
                    return;
                }
            }
        }
        parser.Deposit( None);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxImportSource!( char);

//---------------------------------------------------------------------------------------------------------------------------------
