//-- numbers.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::shard::Parser;
use	crate::flux::{ IFluxOutSource, fluxout::FieldOut };
use	crate::flux::fluxin::FieldIn;
use	crate::shard::{ IGrammar, IForge };
use	crate::silo::{ U8, U32, U64 };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct UIntShard;
pub const UInt: &UIntShard = &UIntShard;

//---------------------------------------------------------------------------------------------------------------------------------

impl IFluxOutSource for UIntShard
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>)
    {
        *field = FieldOut::String( "UInt".to_string());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for UIntShard
{
    fn	Match( &self, parser: &mut Parser, mut sink: FieldIn< '_>)
    {
        let  	origMark = parser.Forge().Mark();
        let  	mut m = origMark;
        let  	mut matched = false;
        
        loop {
            let  	curr = parser.GetAt( m);
            if curr >= U8( b'0') && curr <= U8( b'9') {
                matched = true;
                if let  	Some( nextM) = parser.Incr( m) {
                    m = nextM;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if matched {
            sink.Resolve();
            if matches!( sink, FieldIn::U64( _) | FieldIn::FluxSink( _)) {
                let  	bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
                if let  	Ok( s) = std::str::from_utf8( bytes) {
                    if let  	Ok( val) = s.parse::<u64>() {
                        if let  	FieldIn::U64( dst) = sink {
                            *dst = U64( val);
                        } else if let  	FieldIn::FluxSink( flx) = sink {
                            let  	mut temp = U64( val);
                            flx.FromFieldIn( FieldIn::U64( &mut temp));
                        }
                    }
                }
            }
            parser.Forge().Deposit( Some( m));
        } else {
            parser.Forge().Deposit( None);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for UIntShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "UInt");
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Debug for UIntShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "UInt");
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct IntShard;
pub const Int: &IntShard = &IntShard;

impl IFluxOutSource for IntShard
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>) { *field = FieldOut::String( "Int".to_string()); }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for IntShard
{
    fn	Match( &self, parser: &mut Parser, mut sink: FieldIn< '_>)
    {
        let  	origMark = parser.Forge().Mark();
        let  	mut m = origMark;
        
        let  	curr = parser.GetAt( m);
        if curr == U8( b'-') || curr == U8( b'+') {
            if let  	Some( nextM) = parser.Incr( m) {
                m = nextM;
            } else {
                parser.Forge().Deposit( None);
                return;
            }
        }
        
        let  	mut matched = false;
        loop {
            let  	curr = parser.GetAt( m);
            if curr >= U8( b'0') && curr <= U8( b'9') {
                matched = true;
                if let  	Some( nextM) = parser.Incr( m) {
                    m = nextM;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if matched {
            sink.Resolve();
            if matches!( sink, FieldIn::U64( _) | FieldIn::FluxSink( _)) {
                let  	bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
                if let  	Ok( s) = std::str::from_utf8( bytes) {
                    let  	s_trim = s.trim_start_matches('+');
                    let  	sign = if s_trim.starts_with('-') { -1 } else { 1 };
                    let  	s_num = s_trim.trim_start_matches('-');
                    if let  	Ok( val) = s_num.parse::<u64>() {
                        let  	final_val = if sign == -1 { ( -( val as i64)) as u64 } else { val };
                        if let  	FieldIn::U64( dst) = sink {
                            *dst = U64( final_val);
                        } else if let  	FieldIn::FluxSink( flx) = sink {
                            let  	mut temp = U64( final_val);
                            flx.FromFieldIn( FieldIn::U64( &mut temp));
                        }
                    }
                }
            }
            parser.Forge().Deposit( Some( m));
        } else {
            parser.Forge().Deposit( None);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for IntShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Int") } }
impl fmt::Debug for IntShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Int") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct HexShard;
pub const Hex: &HexShard = &HexShard;

impl IFluxOutSource for HexShard
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>) { *field = FieldOut::String( "Hex".to_string()); }
}
impl IGrammar for HexShard
{
    fn	Match( &self, parser: &mut Parser, mut sink: FieldIn< '_>)
    {
        let  	origMark = parser.Forge().Mark();
        let  	mut currentMark = origMark;
        let  	mut curr = parser.GetAt( currentMark);
        if curr == U8( b'+') || curr == U8( b'-') {
            if let  	Some( nextMark) = parser.Incr( currentMark) {
                currentMark = nextMark;
                curr = parser.GetAt( currentMark);
            } else {
                parser.Forge().Deposit( None);
                return;
            }
        }
        
        let  	mut mark_after_prefix = currentMark;
        if curr == U8( b'0') {
            if let  	Some( nextMark) = parser.Incr( currentMark) {
                let  	curr2 = parser.GetAt( nextMark);
                if curr2 == U8( b'x') || curr2 == U8( b'X') {
                    if let  	Some( nextMark2) = parser.Incr( nextMark) {
                        mark_after_prefix = nextMark2;
                    }
                }
            }
        }
        currentMark = mark_after_prefix;
        
        let  	mut matched = false;
        loop {
            let  	curr = parser.GetAt( currentMark);
            if ( curr >= U8( b'0') && curr <= U8( b'9')) || ( curr >= U8( b'a') && curr <= U8( b'f')) || ( curr >= U8( b'A') && curr <= U8( b'F')) {
                matched = true;
                if let  	Some( nextMark) = parser.Incr( currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if matched {
            sink.Resolve();
            if matches!( sink, FieldIn::U64( _) | FieldIn::FluxSink( _)) {
                let  	bytes = parser.InStream().BytesAt( origMark, U32( currentMark.0 - origMark.0));
                if let  	Ok( s) = std::str::from_utf8( bytes) {
                    let  	mut s_trim = s.trim_start_matches(|c| c == '+' || c == '-');
                    let  	sign = if s.starts_with('-') { -1 } else { 1 };
                    if s_trim.starts_with("0x") || s_trim.starts_with("0X") {
                        s_trim = &s_trim[2..];
                    }
                    if let  	Ok( val) = u64::from_str_radix( s_trim, 16) {
                        let  	final_val = if sign == -1 { ( -( val as i64)) as u64 } else { val };
                        if let  	FieldIn::U64( dst) = sink {
                            *dst = U64( final_val);
                        } else if let  	FieldIn::FluxSink( flx) = sink {
                            let  	mut temp = U64( final_val);
                            flx.FromFieldIn( FieldIn::U64( &mut temp));
                        }
                    }
                }
            }
            let  	res = Some( currentMark);
            parser.Forge().Deposit( res);
        } else {
            parser.Forge().Deposit( None);
        }
    }
}
impl fmt::Display for HexShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Hex") } }
impl fmt::Debug for HexShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Hex") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct RealShard;
pub const Real: &RealShard = &RealShard;

impl IFluxOutSource for RealShard
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>) { *field = FieldOut::String( "Real".to_string()); }
}
impl IGrammar for RealShard
{
    fn	Match( &self, parser: &mut Parser, mut sink: FieldIn< '_>)
    {
        let  	origMark = parser.Forge().Mark();
        let  	mut m = origMark;
        
        // Match optional sign
        let  	curr = parser.GetAt( m);
        if curr == U8( b'-') || curr == U8( b'+') {
            if let  	Some( nextM) = parser.Incr( m) {
                m = nextM;
            } else {
                parser.Forge().Deposit( None);
                return;
            }
        }
        
        let  	mut matched_digits = false;
        loop {
            let  	curr = parser.GetAt( m);
            if curr >= U8( b'0') && curr <= U8( b'9') {
                matched_digits = true;
                if let  	Some( nextM) = parser.Incr( m) { m = nextM; } else { break; }
            } else { break; }
        }
        
        if parser.GetAt( m) == U8( b'.') {
            if let  	Some( nextM) = parser.Incr( m) {
                m = nextM;
                matched_digits = false;
                loop {
                    let  	curr = parser.GetAt( m);
                    if curr >= U8( b'0') && curr <= U8( b'9') {
                        matched_digits = true;
                        if let  	Some( nextM) = parser.Incr( m) { m = nextM; } else { break; }
                    } else { break; }
                }
            }
        }
        
        if !matched_digits {
            parser.Forge().Deposit( None);
            return;
        }
        
        let  	curr = parser.GetAt( m);
        if curr == U8( b'e') || curr == U8( b'E') {
            if let  	Some( nextM) = parser.Incr( m) {
                m = nextM;
                let  	curr = parser.GetAt( m);
                if curr == U8( b'-') || curr == U8( b'+') {
                    if let  	Some( nextM) = parser.Incr( m) { m = nextM; }
                }
                
                let  	mut matched_exp = false;
                loop {
                    let  	curr = parser.GetAt( m);
                    if curr >= U8( b'0') && curr <= U8( b'9') {
                        matched_exp = true;
                        if let  	Some( nextM) = parser.Incr( m) { m = nextM; } else { break; }
                    } else { break; }
                }
                if !matched_exp {
                    parser.Forge().Deposit( None);
                    return;
                }
            }
        }
        
        sink.Resolve();
        if matches!( sink, FieldIn::F64( _) | FieldIn::U64( _) | FieldIn::FluxSink( _)) {
            let  	bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
            if let  	Ok( s) = std::str::from_utf8( bytes) {
                if let  	Ok( val) = s.parse::<f64>() {
                    if let  	FieldIn::F64( dst) = sink {
                        *dst = val;
                    } else if let  	FieldIn::U64( dst) = sink {
                        *dst = U64( val as u64);
                    } else if let  	FieldIn::FluxSink( flx) = sink {
                        let  	mut temp = val;
                        flx.FromFieldIn( FieldIn::F64( &mut temp));
                    }
                }
            }
        }
        
        parser.Forge().Deposit( Some( m));
    }
}
impl fmt::Display for RealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Real") } }
impl fmt::Debug for RealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Real") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct HexRealShard;
pub const HexReal: &HexRealShard = &HexRealShard;

impl IFluxOutSource for HexRealShard
{
    fn	ToFieldOut< 'b>( &'b self, field: &mut FieldOut< 'b>) { *field = FieldOut::String( "HexReal".to_string()); }
}
impl IGrammar for HexRealShard
{
    fn	Match( &self, parser: &mut Parser, mut sink: FieldIn< '_>)
    {
        let  	origMark = parser.Forge().Mark();
        let  	mut m = origMark;
        
        // Match optional sign
        let  	curr = parser.GetAt( m);
        if curr == U8( b'-') || curr == U8( b'+') {
            if let  	Some( nextM) = parser.Incr( m) {
                m = nextM;
            } else {
                parser.Forge().Deposit( None);
                return;
            }
        }
        
        // Match 0x prefix
        let  	curr = parser.GetAt( m);
        if curr == U8( b'0') {
            if let  	Some( nextM) = parser.Incr( m) {
                m = nextM;
                let  	curr = parser.GetAt( m);
                if curr == U8( b'x') || curr == U8( b'X') {
                    if let  	Some( nextM) = parser.Incr( m) {
                        m = nextM;
                    } else {
                        parser.Forge().Deposit( None);
                        return;
                    }
                } else {
                    parser.Forge().Deposit( None);
                    return;
                }
            } else {
                parser.Forge().Deposit( None);
                return;
            }
        } else {
            parser.Forge().Deposit( None);
            return;
        }
        
        let  	mut matched_digits = false;
        loop {
            let  	curr = parser.GetAt( m);
            if ( curr >= U8( b'0') && curr <= U8( b'9')) ||
               ( curr >= U8( b'a') && curr <= U8( b'f')) ||
               ( curr >= U8( b'A') && curr <= U8( b'F')) {
                matched_digits = true;
                if let  	Some( nextM) = parser.Incr( m) { m = nextM; } else { break; }
            } else { break; }
        }
        
        if parser.GetAt( m) == U8( b'.') {
            if let  	Some( nextM) = parser.Incr( m) {
                m = nextM;
                matched_digits = false; // Reset to ensure we match digits after point
                loop {
                    let  	curr = parser.GetAt( m);
                    if ( curr >= U8( b'0') && curr <= U8( b'9')) ||
                       ( curr >= U8( b'a') && curr <= U8( b'f')) ||
                       ( curr >= U8( b'A') && curr <= U8( b'F')) {
                        matched_digits = true;
                        if let  	Some( nextM) = parser.Incr( m) { m = nextM; } else { break; }
                    } else { break; }
                }
            }
        }
        
        if !matched_digits {
            parser.Forge().Deposit( None);
            return;
        }
        
        let  	curr = parser.GetAt( m);
        if curr == U8( b'p') || curr == U8( b'P') {
            if let  	Some( nextM) = parser.Incr( m) {
                m = nextM;
                let  	curr = parser.GetAt( m);
                if curr == U8( b'-') || curr == U8( b'+') {
                    if let  	Some( nextM) = parser.Incr( m) { m = nextM; }
                }
                
                let  	mut matched_exp = false;
                loop {
                    let  	curr = parser.GetAt( m);
                    if curr >= U8( b'0') && curr <= U8( b'9') {
                        matched_exp = true;
                        if let  	Some( nextM) = parser.Incr( m) { m = nextM; } else { break; }
                    } else { break; }
                }
                if !matched_exp {
                    parser.Forge().Deposit( None);
                    return;
                }
            }
        }
        
        sink.Resolve();
        // Parsing HexReal string into f64 isn't natively supported by std::str::parse, so we just populate string for now if it's String.
        // Actually, we will just parse it to F64 if possible, else skip.
        if matches!( sink, FieldIn::F64( _) | FieldIn::FluxSink( _)) {
            let  	bytes = parser.InStream().BytesAt( origMark, U32( m.0 - origMark.0));
            if let  	Ok( _s) = std::str::from_utf8( bytes) {
                // TODO: hex float parsing
            }
        }
        
        parser.Forge().Deposit( Some( m));
    }
}
impl fmt::Display for HexRealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "HexReal") } }
impl fmt::Debug for HexRealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "HexReal") } }

//---------------------------------------------------------------------------------------------------------------------------------
