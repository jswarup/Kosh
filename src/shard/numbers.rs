//-- numbers.rs -----------------------------------------------------------------------------------------------------------------------

use	std::fmt;
use	crate::flux::{ IXFluxSource, xflux::XField };
use	crate::shard::{ IGrammar, IForge };
use	crate::silo::U8;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct UIntShard;
pub const UInt: &UIntShard = &UIntShard;

//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxSource for UIntShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        *field = XField::String( "UInt".to_string());
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for UIntShard
{
    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F)
    {
        let  	origMark = forge.Mark();
        let  	mut currentMark = origMark;
        let  	mut matched = false;
        
        loop {
            let  	curr = forge.Parser().Curr( currentMark);
            if curr >= U8( b'0') && curr <= U8( b'9') {
                matched = true;
                if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if matched {
            forge.SetMark( currentMark);
            let  	res = Some( currentMark);
            forge.Deposit( res);
        } else {
            forge.Deposit( None);
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

impl IXFluxSource for IntShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>) { *field = XField::String( "Int".to_string()); }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IGrammar for IntShard
{
    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F)
    {
        let  	origMark = forge.Mark();
        let  	mut currentMark = origMark;
        let  	curr = forge.Parser().Curr( currentMark);
        if curr == U8( b'+') || curr == U8( b'-') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                currentMark = nextMark;
            } else {
                forge.Deposit( None);
                return;
            }
        }
        let  	mut matched = false;
        loop {
            let  	curr = forge.Parser().Curr( currentMark);
            if curr >= U8( b'0') && curr <= U8( b'9') {
                matched = true;
                if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if matched {
            forge.SetMark( currentMark);
            let  	res = Some( currentMark);
            forge.Deposit( res);
        } else {
            forge.Deposit( None);
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl fmt::Display for IntShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Int") } }
impl fmt::Debug for IntShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Int") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct HexShard;
pub const Hex: &HexShard = &HexShard;

impl IXFluxSource for HexShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>) { *field = XField::String( "Hex".to_string()); }
}
impl IGrammar for HexShard
{
    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F)
    {
        let  	origMark = forge.Mark();
        let  	mut currentMark = origMark;
        let  	mut curr = forge.Parser().Curr( currentMark);
        if curr == U8( b'+') || curr == U8( b'-') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                currentMark = nextMark;
                curr = forge.Parser().Curr( currentMark);
            } else {
                forge.Deposit( None);
                return;
            }
        }
        
        let  	mut mark_after_prefix = currentMark;
        if curr == U8( b'0') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                let  	curr2 = forge.Parser().Curr( nextMark);
                if curr2 == U8( b'x') || curr2 == U8( b'X') {
                    if let  	Some( nextMark2) = forge.Parser().Next( nextMark) {
                        mark_after_prefix = nextMark2;
                    }
                }
            }
        }
        currentMark = mark_after_prefix;
        
        let  	mut matched = false;
        loop {
            let  	curr = forge.Parser().Curr( currentMark);
            if ( curr >= U8( b'0') && curr <= U8( b'9')) || ( curr >= U8( b'a') && curr <= U8( b'f')) || ( curr >= U8( b'A') && curr <= U8( b'F')) {
                matched = true;
                if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if matched {
            forge.SetMark( currentMark);
            let  	res = Some( currentMark);
            forge.Deposit( res);
        } else {
            forge.Deposit( None);
        }
    }
}
impl fmt::Display for HexShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Hex") } }
impl fmt::Debug for HexShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Hex") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct RealShard;
pub const Real: &RealShard = &RealShard;

impl IXFluxSource for RealShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>) { *field = XField::String( "Real".to_string()); }
}
impl IGrammar for RealShard
{
    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F)
    {
        let  	origMark = forge.Mark();
        let  	mut currentMark = origMark;
        let  	curr = forge.Parser().Curr( currentMark);
        if curr == U8( b'+') || curr == U8( b'-') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                currentMark = nextMark;
            } else {
                forge.Deposit( None);
                return;
            }
        }

        let  	mut has_integer_digits = false;
        loop {
            let  	curr = forge.Parser().Curr( currentMark);
            if curr >= U8( b'0') && curr <= U8( b'9') {
                has_integer_digits = true;
                if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let  	mut has_fractional_digits = false;
        let  	curr = forge.Parser().Curr( currentMark);
        if curr == U8( b'.') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                currentMark = nextMark;
                loop {
                    let  	curr = forge.Parser().Curr( currentMark);
                    if curr >= U8( b'0') && curr <= U8( b'9') {
                        has_fractional_digits = true;
                        if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                            currentMark = nextMark;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        if !has_integer_digits && !has_fractional_digits {
            forge.Deposit( None);
            return;
        }

        let  	curr = forge.Parser().Curr( currentMark);
        if curr == U8( b'e') || curr == U8( b'E') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                let  	mut expMark = nextMark;
                let  	curr_exp = forge.Parser().Curr( expMark);
                if curr_exp == U8( b'+') || curr_exp == U8( b'-') {
                    if let  	Some( n) = forge.Parser().Next( expMark) {
                        expMark = n;
                    }
                }
                let  	mut has_exp_digits = false;
                loop {
                    let  	curr = forge.Parser().Curr( expMark);
                    if curr >= U8( b'0') && curr <= U8( b'9') {
                        has_exp_digits = true;
                        if let  	Some( n) = forge.Parser().Next( expMark) {
                            expMark = n;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if has_exp_digits {
                    currentMark = expMark;
                }
            }
        }
        forge.SetMark( currentMark);
        let  	res = Some( currentMark);
        forge.Deposit( res);
    }
}
impl fmt::Display for RealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Real") } }
impl fmt::Debug for RealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "Real") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct HexRealShard;
pub const HexReal: &HexRealShard = &HexRealShard;

impl IXFluxSource for HexRealShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>) { *field = XField::String( "HexReal".to_string()); }
}
impl IGrammar for HexRealShard
{
    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F)
    {
        let  	origMark = forge.Mark();
        let  	mut currentMark = origMark;
        let  	mut curr = forge.Parser().Curr( currentMark);
        if curr == U8( b'+') || curr == U8( b'-') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                currentMark = nextMark;
                curr = forge.Parser().Curr( currentMark);
            } else {
                forge.Deposit( None);
                return;
            }
        }

        if curr == U8( b'0') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                let  	curr2 = forge.Parser().Curr( nextMark);
                if curr2 == U8( b'x') || curr2 == U8( b'X') {
                    if let  	Some( nextMark2) = forge.Parser().Next( nextMark) {
                        currentMark = nextMark2;
                    } else {
                        forge.Deposit( None);
                        return;
                    }
                } else {
                    forge.Deposit( None);
                    return;
                }
            } else {
                forge.Deposit( None);
                return;
            }
        } else {
            forge.Deposit( None);
            return;
        }

        let  	mut has_integer_digits = false;
        loop {
            let  	curr = forge.Parser().Curr( currentMark);
            if ( curr >= U8( b'0') && curr <= U8( b'9')) || ( curr >= U8( b'a') && curr <= U8( b'f')) || ( curr >= U8( b'A') && curr <= U8( b'F')) {
                has_integer_digits = true;
                if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let  	mut has_fractional_digits = false;
        let  	curr = forge.Parser().Curr( currentMark);
        if curr == U8( b'.') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                currentMark = nextMark;
                loop {
                    let  	curr = forge.Parser().Curr( currentMark);
                    if ( curr >= U8( b'0') && curr <= U8( b'9')) || ( curr >= U8( b'a') && curr <= U8( b'f')) || ( curr >= U8( b'A') && curr <= U8( b'F')) {
                        has_fractional_digits = true;
                        if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                            currentMark = nextMark;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        if !has_integer_digits && !has_fractional_digits {
            forge.Deposit( None);
            return;
        }

        let  	curr = forge.Parser().Curr( currentMark);
        if curr == U8( b'p') || curr == U8( b'P') {
            if let  	Some( nextMark) = forge.Parser().Next( currentMark) {
                let  	mut expMark = nextMark;
                let  	curr_exp = forge.Parser().Curr( expMark);
                if curr_exp == U8( b'+') || curr_exp == U8( b'-') {
                    if let  	Some( n) = forge.Parser().Next( expMark) {
                        expMark = n;
                    }
                }
                let  	mut has_exp_digits = false;
                loop {
                    let  	curr = forge.Parser().Curr( expMark);
                    if curr >= U8( b'0') && curr <= U8( b'9') {
                        has_exp_digits = true;
                        if let  	Some( n) = forge.Parser().Next( expMark) {
                            expMark = n;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                if has_exp_digits {
                    currentMark = expMark;
                }
            }
        }
        forge.SetMark( currentMark);
        let  	res = Some( currentMark);
        forge.Deposit( res);
    }
}
impl fmt::Display for HexRealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "HexReal") } }
impl fmt::Debug for HexRealShard { fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result { write!( f, "HexReal") } }

//---------------------------------------------------------------------------------------------------------------------------------
