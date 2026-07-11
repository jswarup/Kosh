//-- repeatshard.rs -----------------------------------------------------------------------------------------------------------------
use	crate::silo::{ U32, IVoidPtrExt };
use	crate::stalks::{ DynINode, INode };
use	crate::flux::{ IXFluxSource, xflux::XField };
use	std::fmt;
use	crate::shard::{ IGrammar, Parser };

//---------------------------------------------------------------------------------------------------------------------------------

pub struct RepeatShard< 'a>
{
    pub _Child: &'a DynINode< 'a>,
    pub _USeg: crate::silo::USeg,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IXFluxSource for RepeatShard< 'a>
{
    fn	ToXField< 'b>( &'b self, field: &mut XField< 'b>)
{
        let  	mut step = 0u32;
        let  	node = self;
        *field = XField::Obj( Box::new( move |key, item| {
            if step == 0 {
                *key = "Child".to_string();
                node._Child.ToXField( item);
                step += 1;
                true
            } else if step == 1 {
                *key = "Repeat".to_string();
                *item = XField::FluxSource( &node._USeg);
                step += 1;
                true
            } else { false }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> INode< 'a> for RepeatShard< 'a>
{
    fn	_Size( &self) -> U32 { U32( 1) }
    fn	_At( &self, idx: U32) -> &DynINode< 'a>
{
        if idx.0 == 0 {
            self._Child
        } else {
            panic!( "At called on RepeatShard with index > 0")
        }
    }

    fn	MatchGrammar( &self, parser: *mut (), marker: U32) -> (bool, U32)
{
        let  	p = parser.MutRef::< Parser< '_>>();
        self.Match( p, marker)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> IGrammar for RepeatShard< 'a>
{
    fn	Match< 'p>( &'p self, parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
{
        let  	mut count = U32( 0);
        let  	first = self._USeg.First();
        let  	last = if self._USeg.IsEmpty() { U32( std::u32::MAX) } else { self._USeg.Last() };
        let  	mut currMark = marker;

        while count < last {
            let (matched, m) = self._Child.Match( parser, currMark);
            if matched {
                if m == currMark {
                    count += U32( 1);
                    break;
                }
                currMark = m;
                count += U32( 1);
            } else {
                break;
            }
        }

        if count >= first {
            (true, currMark)
        } else {
            (false, marker)
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Display for RepeatShard< 'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
{
        write!( f, "Repeat( {:?})", self._USeg)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< 'a> fmt::Debug for RepeatShard< 'a>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
{
        fmt::Display::fmt( self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
