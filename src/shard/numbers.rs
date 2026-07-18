//-- numbers.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use	crate::flux::fluximport::FieldImp;
use	crate::shard::{ Parser, IGrammar };
use	crate::silo::{ U8, U32, U64 };

//---------------------------------------------------------------------------------------------------------------------------------

macro_rules! ImplNumberShard
{
    ( $shard:ident, $cnst:ident, $label:literal ) =>
    {
        pub struct $shard;
        pub const $cnst: &$shard = &$shard;

        impl IFluxExportSource for $shard
        {
            fn    FetchFieldExp< 'b>( &'b self, field: &mut FieldExp< 'b>)
            {
                *field = FieldExp::String( $label.to_string());
            }
        }

        impl fmt::Display for $shard { fn    fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "{}", $label) } }
        impl fmt::Debug for $shard { fn    fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "{}", $label) } }

        crate::ImplFluxImportSource!( $shard);
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

fn    MatchSign( parser: &mut Parser, m: U32) -> U32
{
    let      curr = parser.GetAt( m);
    if curr == U8( b'-') || curr == U8( b'+') { parser.Incr( m).unwrap_or( m) } else { m }
}

//---------------------------------------------------------------------------------------------------------------------------------

fn    MatchDecDigits( parser: &mut Parser, mut m: U32) -> ( U32, bool)
{
    let      mut matched = false;
    loop {
        let      curr = parser.GetAt( m);
        if curr >= U8( b'0') && curr <= U8( b'9') {
            matched = true;
            if let      Some( nextM) = parser.Incr( m) { m = nextM; } else { break; }
        } else { break; }
    }
    ( m, matched)
}

//---------------------------------------------------------------------------------------------------------------------------------

fn    MatchHexDigits( parser: &mut Parser, mut m: U32) -> ( U32, bool)
{
    let      mut matched = false;
    loop {
        let      curr = parser.GetAt( m);
        if ( curr >= U8( b'0') && curr <= U8( b'9')) || ( curr >= U8( b'a') && curr <= U8( b'f')) || ( curr >= U8( b'A') && curr <= U8( b'F')) {
            matched = true;
            if let      Some( nextM) = parser.Incr( m) { m = nextM; } else { break; }
        } else { break; }
    }
    ( m, matched)
}

//---------------------------------------------------------------------------------------------------------------------------------

fn    MatchHexPrefix( parser: &mut Parser, m: U32) -> Option< U32>
{
    if parser.GetAt( m) != U8( b'0') { return None; }
    let      m = parser.Incr( m)?;
    let      curr = parser.GetAt( m);
    if curr != U8( b'x') && curr != U8( b'X') { return None; }
    parser.Incr( m)
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( UIntShard, UInt, "UInt");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for UIntShard
{
    fn    Match( &self, parser: &mut Parser) -> bool
    {
        let      origMark = parser.CurrMark();
        let      ( m, matched) = MatchDecDigits( parser, origMark);
        if !matched { return false; }
        let      bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0)); 
        parser.SetCurrMark( m);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( IntShard, Int, "Int");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for IntShard
{
    fn    Match( &self, parser: &mut Parser) -> bool
    {
        let      origMark = parser.CurrMark();
        let      mSign = MatchSign( parser, origMark);
        let      ( m, matched) = MatchDecDigits( parser, mSign);
        if !matched { 
            return false;
        } 
        parser.SetCurrMark( m);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( HexShard, Hex, "Hex");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for HexShard
{
    fn    Match( &self, parser: &mut Parser) -> bool
    {
        let      origMark = parser.CurrMark();
        let      m = MatchSign( parser, origMark);
        // Advance past optional 0x/0X prefix
        let      mDigits = MatchHexPrefix( parser, m).unwrap_or( m);
        let      ( m, matched) = MatchHexDigits( parser, mDigits);
        if !matched { 
            return false;
        } 
        parser.SetCurrMark( m);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( RealShard, Real, "Real");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for RealShard
{
    fn    Match( &self, parser: &mut Parser) -> bool
    {
        let      origMark = parser.CurrMark();
        let      mut m = MatchSign( parser, origMark);
        let      mut matchedDigits = false;
        let      ( nextM, d) = MatchDecDigits( parser, m);
        if d { m = nextM; matchedDigits = true; }
        if parser.GetAt( m) == U8( b'.') {
            if let      Some( nextM) = parser.Incr( m) {
                m = nextM;
                let      ( nextM, d) = MatchDecDigits( parser, m);
                if d { m = nextM; matchedDigits = true; }
            }
        }
        if !matchedDigits { return false; }
        // Optional exponent
        let      curr = parser.GetAt( m);
        if curr == U8( b'e') || curr == U8( b'E') {
            if let      Some( nextM) = parser.Incr( m) {
                m = nextM;
                let      curr = parser.GetAt( m);
                if curr == U8( b'-') || curr == U8( b'+') {
                    if let      Some( nextM) = parser.Incr( m) { m = nextM; }
                }
                let      ( nextM, matched) = MatchDecDigits( parser, m);
                if !matched { return false; }
                m = nextM;
            }
        } 
        parser.SetCurrMark( m);
        true
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
