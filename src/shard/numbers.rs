//-- numbers.rs -----------------------------------------------------------------------------------------------------------------------

use std::fmt;
use crate::flux::{ IXFluxSource, xflux::XField };
use crate::shard::{ IGrammar, Parser };
use crate::silo::{ U32, U8 };
use crate::stalks::INode;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct UIntShard;
pub const UInt: &UIntShard = &UIntShard;

impl IXFluxSource for UIntShard
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
    {
        *field = XField::String( "UInt".to_string());
    }
}

impl< 'a> INode< 'a> for UIntShard
{
    fn	MatchGrammar( &self, parser: *mut (), marker: u32) -> Option< u32>
    {
        let  	parserRef = unsafe { &mut *( parser as *mut Parser< '_>) };
        
        return self.Match( parserRef, U32( marker)).map( |u| u.0);
    }
}

impl IGrammar for UIntShard
{
    fn	Match< 'p>( &'p self, parser: &mut Parser< 'p>, marker: U32) -> Option< U32>
    {
        let  	mut currentMark = marker;
        let  	mut matched = false;
        
        loop {
            let  	curr = parser.Curr( currentMark);
            if curr >= U8( b'0') && curr <= U8( b'9') {
                matched = true;
                if let Some( nextMark) = parser.Next( currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if matched {
            return Some( currentMark);
        }
        
        return None;
    }
}

impl fmt::Display for UIntShard
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        return write!( f, "UInt");
    }
}

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

impl IXFluxSource for IntShard {
    fn ToXField<'b>(&'b self, field: &mut XField<'b>) { *field = XField::String("Int".to_string()); }
}
impl<'a> INode<'a> for IntShard {
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let parserRef = unsafe { &mut *(parser as *mut Parser<'_>) };
        return self.Match(parserRef, U32(marker)).map(|u| u.0);
    }
}
impl IGrammar for IntShard {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        let mut currentMark = marker;
        let curr = parser.Curr(currentMark);
        if curr == U8(b'+') || curr == U8(b'-') {
            if let Some(nextMark) = parser.Next(currentMark) {
                currentMark = nextMark;
            } else {
                return None;
            }
        }
        let mut matched = false;
        loop {
            let curr = parser.Curr(currentMark);
            if curr >= U8(b'0') && curr <= U8(b'9') {
                matched = true;
                if let Some(nextMark) = parser.Next(currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if matched {
            return Some(currentMark);
        }
        return None;
    }
}
impl fmt::Display for IntShard { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Int") } }
impl fmt::Debug for IntShard { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Int") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct HexShard;
pub const Hex: &HexShard = &HexShard;

impl IXFluxSource for HexShard {
    fn ToXField<'b>(&'b self, field: &mut XField<'b>) { *field = XField::String("Hex".to_string()); }
}
impl<'a> INode<'a> for HexShard {
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let parserRef = unsafe { &mut *(parser as *mut Parser<'_>) };
        return self.Match(parserRef, U32(marker)).map(|u| u.0);
    }
}
impl IGrammar for HexShard {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        let mut currentMark = marker;
        let mut curr = parser.Curr(currentMark);
        if curr == U8(b'+') || curr == U8(b'-') {
            if let Some(nextMark) = parser.Next(currentMark) {
                currentMark = nextMark;
                curr = parser.Curr(currentMark);
            } else {
                return None;
            }
        }
        
        let mut mark_after_prefix = currentMark;
        if curr == U8(b'0') {
            if let Some(nextMark) = parser.Next(currentMark) {
                let curr2 = parser.Curr(nextMark);
                if curr2 == U8(b'x') || curr2 == U8(b'X') {
                    if let Some(nextMark2) = parser.Next(nextMark) {
                        mark_after_prefix = nextMark2;
                    }
                }
            }
        }
        currentMark = mark_after_prefix;
        
        let mut matched = false;
        loop {
            let curr = parser.Curr(currentMark);
            if (curr >= U8(b'0') && curr <= U8(b'9')) || (curr >= U8(b'a') && curr <= U8(b'f')) || (curr >= U8(b'A') && curr <= U8(b'F')) {
                matched = true;
                if let Some(nextMark) = parser.Next(currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        if matched {
            return Some(currentMark);
        }
        return None;
    }
}
impl fmt::Display for HexShard { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Hex") } }
impl fmt::Debug for HexShard { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Hex") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct RealShard;
pub const Real: &RealShard = &RealShard;

impl IXFluxSource for RealShard {
    fn ToXField<'b>(&'b self, field: &mut XField<'b>) { *field = XField::String("Real".to_string()); }
}
impl<'a> INode<'a> for RealShard {
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let parserRef = unsafe { &mut *(parser as *mut Parser<'_>) };
        return self.Match(parserRef, U32(marker)).map(|u| u.0);
    }
}
impl IGrammar for RealShard {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        let mut currentMark = marker;
        let curr = parser.Curr(currentMark);
        if curr == U8(b'+') || curr == U8(b'-') {
            if let Some(nextMark) = parser.Next(currentMark) {
                currentMark = nextMark;
            } else {
                return None;
            }
        }

        let mut has_integer_digits = false;
        loop {
            let curr = parser.Curr(currentMark);
            if curr >= U8(b'0') && curr <= U8(b'9') {
                has_integer_digits = true;
                if let Some(nextMark) = parser.Next(currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let mut has_fractional_digits = false;
        let curr = parser.Curr(currentMark);
        if curr == U8(b'.') {
            if let Some(nextMark) = parser.Next(currentMark) {
                currentMark = nextMark;
                loop {
                    let curr = parser.Curr(currentMark);
                    if curr >= U8(b'0') && curr <= U8(b'9') {
                        has_fractional_digits = true;
                        if let Some(nextMark) = parser.Next(currentMark) {
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
            return None;
        }

        let curr = parser.Curr(currentMark);
        if curr == U8(b'e') || curr == U8(b'E') {
            if let Some(nextMark) = parser.Next(currentMark) {
                let mut expMark = nextMark;
                let curr_exp = parser.Curr(expMark);
                if curr_exp == U8(b'+') || curr_exp == U8(b'-') {
                    if let Some(n) = parser.Next(expMark) {
                        expMark = n;
                    }
                }
                let mut has_exp_digits = false;
                loop {
                    let curr = parser.Curr(expMark);
                    if curr >= U8(b'0') && curr <= U8(b'9') {
                        has_exp_digits = true;
                        if let Some(n) = parser.Next(expMark) {
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

        Some(currentMark)
    }
}
impl fmt::Display for RealShard { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Real") } }
impl fmt::Debug for RealShard { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "Real") } }

//---------------------------------------------------------------------------------------------------------------------------------

pub struct HexRealShard;
pub const HexReal: &HexRealShard = &HexRealShard;

impl IXFluxSource for HexRealShard {
    fn ToXField<'b>(&'b self, field: &mut XField<'b>) { *field = XField::String("HexReal".to_string()); }
}
impl<'a> INode<'a> for HexRealShard {
    fn MatchGrammar(&self, parser: *mut (), marker: u32) -> Option<u32> {
        let parserRef = unsafe { &mut *(parser as *mut Parser<'_>) };
        return self.Match(parserRef, U32(marker)).map(|u| u.0);
    }
}
impl IGrammar for HexRealShard {
    fn Match<'p>(&'p self, parser: &mut Parser<'p>, marker: U32) -> Option<U32> {
        let mut currentMark = marker;
        let mut curr = parser.Curr(currentMark);
        if curr == U8(b'+') || curr == U8(b'-') {
            if let Some(nextMark) = parser.Next(currentMark) {
                currentMark = nextMark;
                curr = parser.Curr(currentMark);
            } else {
                return None;
            }
        }

        if curr == U8(b'0') {
            if let Some(nextMark) = parser.Next(currentMark) {
                let curr2 = parser.Curr(nextMark);
                if curr2 == U8(b'x') || curr2 == U8(b'X') {
                    if let Some(nextMark2) = parser.Next(nextMark) {
                        currentMark = nextMark2;
                    } else { return None; }
                } else { return None; }
            } else { return None; }
        } else { return None; }

        let mut has_integer_digits = false;
        loop {
            let curr = parser.Curr(currentMark);
            if (curr >= U8(b'0') && curr <= U8(b'9')) || (curr >= U8(b'a') && curr <= U8(b'f')) || (curr >= U8(b'A') && curr <= U8(b'F')) {
                has_integer_digits = true;
                if let Some(nextMark) = parser.Next(currentMark) {
                    currentMark = nextMark;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        let mut has_fractional_digits = false;
        let curr = parser.Curr(currentMark);
        if curr == U8(b'.') {
            if let Some(nextMark) = parser.Next(currentMark) {
                currentMark = nextMark;
                loop {
                    let curr = parser.Curr(currentMark);
                    if (curr >= U8(b'0') && curr <= U8(b'9')) || (curr >= U8(b'a') && curr <= U8(b'f')) || (curr >= U8(b'A') && curr <= U8(b'F')) {
                        has_fractional_digits = true;
                        if let Some(nextMark) = parser.Next(currentMark) {
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
            return None;
        }

        let curr = parser.Curr(currentMark);
        if curr == U8(b'p') || curr == U8(b'P') {
            if let Some(nextMark) = parser.Next(currentMark) {
                let mut expMark = nextMark;
                let curr_exp = parser.Curr(expMark);
                if curr_exp == U8(b'+') || curr_exp == U8(b'-') {
                    if let Some(n) = parser.Next(expMark) {
                        expMark = n;
                    }
                }
                let mut has_exp_digits = false;
                loop {
                    let curr = parser.Curr(expMark);
                    if curr >= U8(b'0') && curr <= U8(b'9') {
                        has_exp_digits = true;
                        if let Some(n) = parser.Next(expMark) {
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

        Some(currentMark)
    }
}
impl fmt::Display for HexRealShard { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "HexReal") } }
impl fmt::Debug for HexRealShard { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "HexReal") } }

//---------------------------------------------------------------------------------------------------------------------------------
