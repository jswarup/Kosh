//-- numbers.rs -----------------------------------------------------------------------------------------------------------------------

use    std::fmt;
use    crate::flux::{ IFluxImportSource };
use    crate::shard::Parser;
use    crate::flux::{ IFluxExportSource, fluxexport::FieldExp };
use    crate::flux::fluximport::FieldImp;
use    crate::shard::{ IGrammar, IForge };
use    crate::silo::{ U8, U32, U64 };

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
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

fn    MatchSign( parser: &mut Parser, m: U32) -> Option< U32>
{
    let      curr = parser.GetAt( m);
    if curr == U8( b'-') || curr == U8( b'+') {
        parser.Incr( m)
    } else {
        Some( m)
    }
}

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

fn    MatchHexPrefix( parser: &mut Parser, m: U32) -> Option< U32>
{
    if parser.GetAt( m) != U8( b'0') {
        return None;
    }
    let      m = parser.Incr( m)?;
    let      curr = parser.GetAt( m);
    if curr != U8( b'x') && curr != U8( b'X') {
        return None;
    }
    parser.Incr( m)
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct SignShard;
pub const Sign: &SignShard = &SignShard;

impl IFluxExportSource for SignShard { fn FetchFieldExp<'b>(&'b self, _field: &mut FieldExp<'b>) {} }
impl<'a> IFluxImportSource for SignShard { fn FetchFieldImp<'b>(&'b mut self, _field: &mut FieldImp<'b>) {} }

impl IGrammar for SignShard {
    fn Match(&self, parser: &mut Parser, _sink: FieldImp<'_>) {
        let mark = parser.CurrentMark();
        let res = MatchSign(parser, mark);
        parser.Forge().Deposit(res);
    }
}

pub struct DecDigitsShard;
pub const DecDigits: &DecDigitsShard = &DecDigitsShard;

impl IFluxExportSource for DecDigitsShard { fn FetchFieldExp<'b>(&'b self, _field: &mut FieldExp<'b>) {} }
impl<'a> IFluxImportSource for DecDigitsShard { fn FetchFieldImp<'b>(&'b mut self, _field: &mut FieldImp<'b>) {} }

impl IGrammar for DecDigitsShard {
    fn Match(&self, parser: &mut Parser, _sink: FieldImp<'_>) {
        let mark = parser.CurrentMark();
        let (m, matched) = MatchDecDigits(parser, mark);
        if matched { parser.Forge().Deposit(Some(m)); } else { parser.Forge().Deposit(None); }
    }
}

pub struct HexDigitsShard;
pub const HexDigits: &HexDigitsShard = &HexDigitsShard;

impl IFluxExportSource for HexDigitsShard { fn FetchFieldExp<'b>(&'b self, _field: &mut FieldExp<'b>) {} }
impl<'a> IFluxImportSource for HexDigitsShard { fn FetchFieldImp<'b>(&'b mut self, _field: &mut FieldImp<'b>) {} }

impl IGrammar for HexDigitsShard {
    fn Match(&self, parser: &mut Parser, _sink: FieldImp<'_>) {
        let mark = parser.CurrentMark();
        let (m, matched) = MatchHexDigits(parser, mark);
        if matched { parser.Forge().Deposit(Some(m)); } else { parser.Forge().Deposit(None); }
    }
}

pub struct HexPrefixShard;
pub const HexPrefix: &HexPrefixShard = &HexPrefixShard;

impl IFluxExportSource for HexPrefixShard { fn FetchFieldExp<'b>(&'b self, _field: &mut FieldExp<'b>) {} }
impl<'a> IFluxImportSource for HexPrefixShard { fn FetchFieldImp<'b>(&'b mut self, _field: &mut FieldImp<'b>) {} }

impl IGrammar for HexPrefixShard {
    fn Match(&self, parser: &mut Parser, _sink: FieldImp<'_>) {
        let mark = parser.CurrentMark();
        let res = MatchHexPrefix(parser, mark);
        parser.Forge().Deposit(res);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( UIntShard, UInt, "UInt");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for UIntShard
{
    fn    Match( &self, parser: &mut Parser, mut sink: FieldImp< '_>)
    {
        let      origMark = parser.CurrentMark();
        let      m = match parser.ParseGrammar( &DecDigits, origMark, FieldImp::Null) {
            Some(m) => m,
            None => { parser.Forge().Deposit( None); return; }
        };
        sink.Resolve();
        if matches!( sink, FieldImp::U64( _) | FieldImp::FluxSink( _)) {
            let      bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
            if let      Ok( s) = std::str::from_utf8( bytes) {
                if let      Ok( val) = s.parse::<u64>() {
                    if let      FieldImp::U64( dst) = sink {
                        *dst = U64( val);
                    } else if let      FieldImp::FluxSink( flx) = sink {
                        let      mut temp = U64( val);
                        flx.FromFieldImp( FieldImp::U64( &mut temp));
                    }
                }
            }
        }
        parser.Forge().Deposit( Some( m));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( IntShard, Int, "Int");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for IntShard
{
    fn    Match( &self, parser: &mut Parser, mut sink: FieldImp< '_>)
    {
        let      origMark = parser.CurrentMark();
        let      m = match parser.ParseGrammar( &Sign, origMark, FieldImp::Null) {
            Some(m) => m,
            None => { parser.Forge().Deposit( None); return; }
        };
        let      m = match parser.ParseGrammar( &DecDigits, m, FieldImp::Null) {
            Some(m) => m,
            None => { parser.Forge().Deposit( None); return; }
        };
        sink.Resolve();
        if matches!( sink, FieldImp::U64( _) | FieldImp::FluxSink( _)) {
            let      bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
            if let      Ok( s) = std::str::from_utf8( bytes) {
                let      sTrim = s.trim_start_matches( '+');
                let      sign = if sTrim.starts_with( '-') { -1 } else { 1 };
                let      sNum = sTrim.trim_start_matches( '-');
                if let      Ok( val) = sNum.parse::<u64>() {
                    let      finalVal = if sign == -1 { ( -( val as i64)) as u64 } else { val };
                    if let      FieldImp::U64( dst) = sink {
                        *dst = U64( finalVal);
                    } else if let      FieldImp::FluxSink( flx) = sink {
                        let      mut temp = U64( finalVal);
                        flx.FromFieldImp( FieldImp::U64( &mut temp));
                    }
                }
            }
        }
        parser.Forge().Deposit( Some( m));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( HexShard, Hex, "Hex");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for HexShard
{
    fn    Match( &self, parser: &mut Parser, mut sink: FieldImp< '_>)
    {
        let      origMark = parser.CurrentMark();
        let      m = match parser.ParseGrammar( &Sign, origMark, FieldImp::Null) {
            Some(m) => m,
            None => { parser.Forge().Deposit( None); return; }
        };

        // Skip optional 0x prefix (keep original behaviour)
        let      mut mDigits = m;
        if parser.GetAt( mDigits) == U8( b'0') {
            if let      Some( next) = parser.Incr( mDigits) {
                let      c = parser.GetAt( next);
                if c == U8( b'x') || c == U8( b'X') {
                    if let      Some( afterX) = parser.Incr( next) {
                        mDigits = afterX;
                    }
                }
            }
        }

        let      m = match parser.ParseGrammar( &HexDigits, mDigits, FieldImp::Null) {
            Some(m) => m,
            None => { parser.Forge().Deposit( None); return; }
        };
        sink.Resolve();
        if matches!( sink, FieldImp::U64( _) | FieldImp::FluxSink( _)) {
            let      bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
            if let      Ok( s) = std::str::from_utf8( bytes) {
                let      sign = if s.starts_with( '-') { -1 } else { 1 };
                let      mut sTrim = s.trim_start_matches( |c| c == '+' || c == '-');
                if sTrim.starts_with( "0x") || sTrim.starts_with( "0X") {
                    sTrim = &sTrim[2..];
                }
                if let      Ok( val) = u64::from_str_radix( sTrim, 16) {
                    let      finalVal = if sign == -1 { ( -( val as i64)) as u64 } else { val };
                    if let      FieldImp::U64( dst) = sink {
                        *dst = U64( finalVal);
                    } else if let      FieldImp::FluxSink( flx) = sink {
                        let      mut temp = U64( finalVal);
                        flx.FromFieldImp( FieldImp::U64( &mut temp));
                    }
                }
            }
        }
        parser.Forge().Deposit( Some( m));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( RealShard, Real, "Real");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for RealShard
{
    fn    Match( &self, parser: &mut Parser, mut sink: FieldImp< '_>)
    {
        let      origMark = parser.CurrentMark();
        let      m = match parser.ParseGrammar( &Sign, origMark, FieldImp::Null) {
            Some(m) => m,
            None => { parser.Forge().Deposit( None); return; }
        };

        let      (mut m, mut matchedDigits) = (m, false);
        if let Some(nextM) = parser.ParseGrammar( &DecDigits, m, FieldImp::Null) {
            m = nextM;
            matchedDigits = true;
        }

        if parser.GetAt( m) == U8( b'.') {
            if let      Some( nextM) = parser.Incr( m) {
                m = nextM;
                if let Some(nextDigits) = parser.ParseGrammar( &DecDigits, m, FieldImp::Null) {
                    m = nextDigits;
                    matchedDigits = true;
                }
            }
        }

        if !matchedDigits {
            parser.Forge().Deposit( None);
            return;
        }

        // Optional exponent
        let      curr = parser.GetAt( m);
        if curr == U8( b'e') || curr == U8( b'E') {
            if let      Some( nextM) = parser.Incr( m) {
                m = nextM;
                let      curr = parser.GetAt( m);
                if curr == U8( b'-') || curr == U8( b'+') {
                    if let      Some( nextM) = parser.Incr( m) { m = nextM; }
                }
                let      newM = match parser.ParseGrammar( &DecDigits, m, FieldImp::Null) {
                    Some(nm) => nm,
                    None => { parser.Forge().Deposit( None); return; }
                };
                m = newM;
            }
        }

        sink.Resolve();
        if matches!( sink, FieldImp::F64( _) | FieldImp::U64( _) | FieldImp::FluxSink( _)) {
            let      bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
            if let      Ok( s) = std::str::from_utf8( bytes) {
                if let      Ok( val) = s.parse::<f64>() {
                    if let      FieldImp::F64( dst) = sink {
                        *dst = val;
                    } else if let      FieldImp::U64( dst) = sink {
                        *dst = U64( val as u64);
                    } else if let      FieldImp::FluxSink( flx) = sink {
                        let      mut temp = val;
                        flx.FromFieldImp( FieldImp::F64( &mut temp));
                    }
                }
            }
        }
        parser.Forge().Deposit( Some( m));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

ImplNumberShard!( HexRealShard, HexReal, "HexReal");

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for HexRealShard
{
    fn    Match( &self, parser: &mut Parser, mut sink: FieldImp< '_>)
    {
        let      origMark = parser.CurrentMark();
        let      m = match parser.ParseGrammar( &Sign, origMark, FieldImp::Null) {
            Some(m) => m,
            None => { parser.Forge().Deposit( None); return; }
        };

        // Required 0x prefix
        let      m = match MatchHexPrefix( parser, m) {
            Some( m) => m,
            None => {
                parser.Forge().Deposit( None);
                return;
            }
        };

        let      ( mut m, mut matchedDigits) = (m, false);
        if let Some(nextM) = parser.ParseGrammar( &HexDigits, m, FieldImp::Null) {
            m = nextM;
            matchedDigits = true;
        }

        if parser.GetAt( m) == U8( b'.') {
            if let      Some( nextM) = parser.Incr( m) {
                m = nextM;
                if let Some(nextDigits) = parser.ParseGrammar( &HexDigits, m, FieldImp::Null) {
                    m = nextDigits;
                    matchedDigits = true;
                }
            }
        }

        if !matchedDigits {
            parser.Forge().Deposit( None);
            return;
        }

        // Optional binary exponent
        let      curr = parser.GetAt( m);
        if curr == U8( b'p') || curr == U8( b'P') {
            if let      Some( nextM) = parser.Incr( m) {
                m = nextM;
                let      curr = parser.GetAt( m);
                if curr == U8( b'-') || curr == U8( b'+') {
                    if let      Some( nextM) = parser.Incr( m) { m = nextM; }
                }
                let      newM = match parser.ParseGrammar( &DecDigits, m, FieldImp::Null) {
                    Some(nm) => nm,
                    None => { parser.Forge().Deposit( None); return; }
                };
                m = newM;
            }
        }

        sink.Resolve();
        if matches!( sink, FieldImp::F64( _) | FieldImp::FluxSink( _)) {
            let      bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
            if let      Ok( _s) = std::str::from_utf8( bytes) {
                // TODO: hex float parsing
            }
        }
        parser.Forge().Deposit( Some( m));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxImportSource!( UIntShard);
crate::ImplFluxImportSource!( IntShard);
crate::ImplFluxImportSource!( HexShard);
crate::ImplFluxImportSource!( RealShard);
crate::ImplFluxImportSource!( HexRealShard);

//---------------------------------------------------------------------------------------------------------------------------------
