//-- charset.rs -------------------------------------------------------------------------------------------------------------------
use	std::sync::LazyLock;
use	crate::silo::{ Arr, Buff, U8, U64 };
use	crate::segue::{IXFluxable, XField};


//---------------------------------------------------------------------------------------------------------------------------------

/// A 256-bit filter for `U8` characters — one bit per byte value.
/// Enables set algebra (union, intersection, negation) over character classes.
#[derive( Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Charset
{
    _Bits: [U64; Self::SZ],
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Charset
{
    const SZ: usize = 4;
    const SZ_BITS: u32 = 64;

    //-----------------------------------------------------------------------------------------------------------------------------

    pub const fn	New() -> Self
    { Self
    { _Bits: [U64::_0; Self::SZ] } }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	FromFilter( filter: fn( U8) -> bool) -> Self
    {
        let  	mut cs = Self::New();
        for i in 0u16..=255 {
            cs.Set( U8( i as u8), filter( U8( i as u8)));
        }
        cs
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	FromBoxet( spec: Arr< '_, U8>) -> Self
    {
        let  	mut cs = Self::New();
        let  	mut i = 0usize;
        while i < spec.len() {
            let  	first = spec[i];
            cs.SetChar( first);
            // peek for  '-' range
            if i + 2 < spec.len() && spec[i + 1] == U8( b'-') {
                let  	last = spec[i + 2];
                cs.SetByteRange( first, last, true);
                i += 3;
            } else {
                i += 1;
            }
        }
        cs
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Get< C: Into< U8>>( &self, c: C) -> bool
    {
        let  	c = c.into();
        let  	idx = ( c.AsUsize()) / Self::SZ_BITS as usize;
        let  	bit = ( c.AsU8() as u32) % Self::SZ_BITS;
        ( self._Bits[idx] & U64( 1u64 << bit)) != 0u64
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	SetChar< C: Into< U8>>( &mut self, c: C)
    {
        let  	c = c.into();
        let  	idx = ( c.AsUsize()) / Self::SZ_BITS as usize;
        let  	bit = ( c.AsU8() as u32) % Self::SZ_BITS;
        self._Bits[idx] |= U64( 1u64 << bit);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	ClearChar< C: Into< U8>>( &mut self, c: C)
    {
        let  	c = c.into();
        let  	idx = ( c.AsUsize()) / Self::SZ_BITS as usize;
        let  	bit = ( c.AsU8() as u32) % Self::SZ_BITS;
        self._Bits[idx] &= !U64( 1u64 << bit);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Set the bit for byte `c` to `v`.
    pub fn	Set< C: Into< U8>>( &mut self, c: C, v: bool)
    {
        if v { self.SetChar( c) } else { self.ClearChar( c) }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Set all bits in the inclusive range `start..=stop` to `value`.
    pub fn	SetByteRange< C: Into< U8>>( &mut self, start: C, stop: C, value: bool)
    {
        let  	start = start.into().AsU8();
        let  	stop = stop.into().AsU8();
        for c in start..=stop {
            self.Set( U8( c), value);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Flip all 256 bits (complement).
    pub fn	Negate( &mut self)
    {
        for w in self._Bits.iter_mut() {
            *w = !*w;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Return a negated copy.
    pub fn	Negative( &self) -> Self
    {
        let  	mut cpy = *self;
        cpy.Negate();
        cpy
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Check whether `self` and `other` share any set bit.
    pub fn	IsIntersect( &self, other: &Charset) -> bool
    {
        for i in 0..Self::SZ {
            if ( self._Bits[i] & other._Bits[i]) != 0u64 {
                return true;
            }
        }
        false
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// OR `other` into self.
    pub fn	UnionWith( &mut self, other: &Charset)
    {
        for i in 0..Self::SZ {
            self._Bits[i] |= other._Bits[i];
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// AND `other` into self.
    pub fn	IntersectWith( &mut self, other: &Charset)
    {
        for i in 0..Self::SZ {
            self._Bits[i] &= other._Bits[i];
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Return the union of self and `other`
    ///
    pub fn	Union( &self, other: &Charset) -> Self
    {
        let  	mut cpy = *self;
        cpy.UnionWith( other);
        cpy
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Intersect( &self, other: &Charset) -> Self
    {
        let  	mut cpy = *self;
        cpy.IntersectWith( other);
        cpy
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Lexicographic comparison of the four U64 words.
    pub fn	Compare( &self, other: &Charset) -> i32
    {
        match self.cmp( other) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Collect all byte-values whose bit is set.
    pub fn	ListChars( &self) -> Buff< U8>
    {
        let  	weight = self.Weight();
        let  	mut list = Buff::New( weight, U8( 0));
        let  	mut idx = 0usize;
        for i in 0..Self::SZ {
            let  	mut val = self._Bits[i].0;
            while val != 0 {
                let  	tz = val.trailing_zeros();
                list[idx] = U8( ( ( i as u32) * Self::SZ_BITS + tz) as u8);
                idx += 1;
                val &= val - 1;
            }
        }
        list
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Count of set bits (population count).
    pub fn	Weight( &self) -> u32
    {
        self._Bits.iter().map( |w| w.0.count_ones()).sum()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    // Predefined character classes
    pub fn	All() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::New().Negative());
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Digit() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_digit()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	NonDigit() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::Digit().Negative());
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Word() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || {
            let  	mut cs = Charset::New();
            cs.SetChar( b'_');
            cs.SetByteRange( b'a', b'z', true);
            cs.SetByteRange( b'A', b'Z', true);
            cs.SetByteRange( b'0', b'9', true);
            cs
        });
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	NonWord() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::Word().Negative());
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	AlphaNum() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_alphanumeric()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Ascii() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Blank() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || {
            let  	mut cs = Charset::New();
            cs.SetChar( b' ');
            cs.SetChar( b'\t');
            cs
        });
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	EndLine() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || {
            let  	mut cs = Charset::New();
            cs.SetChar( b'\n');
            cs
        });
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Cntrl() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_control()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Graph() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_graphic()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Print() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| {
            c.is_ascii_graphic() || c == b' '
        }));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Punct() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_punctuation()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Space() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_whitespace()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	NonSpace() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::Space().Negative());
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Alpha() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_alphabetic()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Upper() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_uppercase()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Lower() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_lowercase()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	XDigit() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::FromFilter( |c| c.is_ascii_hexdigit()));
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	Dot() -> &'static Charset
    {
        Charset::DotAll()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DotAll() -> &'static Charset
    {
        static VAL: LazyLock< Charset> = LazyLock::new( || Charset::New().Negative());
        &VAL
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    // Formatting helpers
    fn	PrettyPrintChar( c: U8, chrClsFlg: bool, out: &mut String)
    {
        let  	val = c.AsU8();
        match val {
            b'\t' => { out.push_str( "\\t"); return; }
            b'\n' => { out.push_str( "\\n"); return; }
            b'\r' => { out.push_str( "\\r"); return; }
            0x0C  => { out.push_str( "\\f"); return; }                 // form-feed
            0x07  => { out.push_str( "\\a"); return; }                 // bell
            0x0B  => { out.push_str( "\\v"); return; }                 // vertical tab
            _ => {}
        }
        let  	mut hex = false;
        let  	mut escape = false;
        if !chrClsFlg {
            if b"'\"=".contains( &val) {
                hex = true;
            }
            if b"^$ *+{}[].\\/|?".contains( &val) {
                escape = true;
            }
        } else {
            if b"^[]\\/- ".contains( &val) {
                escape = true;
            }
        }
        if !val.is_ascii_alphanumeric() && val != b'.' && val != b'$' && val != b'@' && val != b'_' {
            hex = true;
        }
        if escape {
            out.push( '\\');
            out.push( val as char);
        } else if hex {
            out.push_str( &format!( "\\x{:02X}", val));
        } else {
            out.push( val as char);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    /// Format as a bracket expression, optionally negated.
    fn	ToBoxetString( &self, negFlg: bool) -> String
    {
        let  	chars = if negFlg {
            self.Negative().ListChars()
        } else {
            self.ListChars()
        };
        let  	mut s = String::with_capacity( chars.len() * 2 + 4);
        s.push( '[');
        if negFlg {
            s.push( '^');
        }
        let  	mut i = 0usize;
        while i < chars.len() {
            let  	mut j = i + 1;
            while j < chars.len() && chars[j].AsU8() == chars[j - 1].AsU8() + 1 {
                j += 1;
            }
            let  	runLen = j - i;
            Self::PrettyPrintChar( chars[i], true, &mut s);
            if runLen > 2 {
                s.push( '-');
            }
            if runLen > 1 {
                Self::PrettyPrintChar( chars[i + runLen - 1], true, &mut s);
            }
            i += runLen;
        }
        s.push( ']');
        s
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	ToString( &self) -> String
    {
        let  	posStr = self.ToBoxetString( false);
        let  	negStr = self.ToBoxetString( true);
        if posStr.len() <= negStr.len() { posStr } else { negStr }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Display for Charset
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        if self.Compare( Charset::Word()) == 0 {
            return write!( f, "[[Word]]");
        }
        if self.Compare( Charset::NonWord()) == 0 {
            return write!( f, "[[NonWord]]");
        }
        write!( f, "{}", self.ToString())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::fmt::Debug for Charset
{
    fn	fmt( &self, f: &mut std::fmt::Formatter< '_>) -> std::fmt::Result
    {
        write!( f, "Charset({})", self)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::cmp::PartialOrd for Charset
{
    fn	partial_cmp( &self, other: &Self) -> Option< std::cmp::Ordering>
    {
        Some( self.cmp( other))
    }
}
impl std::cmp::Ord for Charset
{
    fn	cmp( &self, other: &Self) -> std::cmp::Ordering
    {
        self._Bits.cmp( &other._Bits)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl std::ops::Not for Charset
{
    type Output = Self;
    fn	not( self) -> Self::Output
    {
        self.Negative()
    }
}
impl std::ops::BitOr for Charset
{
    type Output = Self;
    fn	bitor( self, rhs: Self) -> Self::Output
    {
        self.Union( &rhs)
    }
}
impl std::ops::BitAnd for Charset
{
    type Output = Self;
    fn	bitand( self, rhs: Self) -> Self::Output
    {
        self.Intersect( &rhs)
    }
}
impl std::ops::BitOrAssign for Charset
{
    fn	bitor_assign( &mut self, rhs: Self)
    {
        self.UnionWith( &rhs);
    }
}
impl std::ops::BitAndAssign for Charset
{
    fn	bitand_assign( &mut self, rhs: Self)
    {
        self.IntersectWith( &rhs);
    }
}
 
//---------------------------------------------------------------------------------------------------------------------------------

impl IXFluxable for Charset
{
    fn	ToXFlux< 'a>( &'a self, field: &mut XField< 'a>) 
    {
        let  	s = self.ToString();
        *field = XField::String( s);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
