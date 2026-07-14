//-- numbers.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::shard::Parser;
use	crate::flux::{ IFluxOutSource, fluxout::FieldOut };
use	crate::flux::fluxin::FieldIn;
use	crate::shard::{ IGrammar, IForge };
use	crate::silo::U8;

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
    fn	Match( &self, parser: &mut Parser, sink: FieldIn< '_>)
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
    fn	Match( &self, parser: &mut Parser, sink: FieldIn< '_>)
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
    fn	Match( &self, parser: &mut Parser, sink: FieldIn< '_>)
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
    fn	Match( &self, parser: &mut Parser, sink: FieldIn< '_>)
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
    fn	Match( &self, parser: &mut Parser, sink: FieldIn< '_>)
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
        
        parser.Forge().Deposit( Some( m));
    }
}
impl fmt::Display for HexRealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "HexReal") } }
impl fmt::Debug for HexRealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "HexReal") } }

//---------------------------------------------------------------------------------------------------------------------------------
