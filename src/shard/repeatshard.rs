//-- repeatshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::{
    flux::{ IXFluxSource, xflux::XField },
    shard::{ IGrammar, IForge },
    silo::{ USeg, U32},
    stalks::UniNode,
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type RepeatShard< C> = UniNode< C, USeg>;

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> IXFluxSource for UniNode< C, USeg>
where
    C: IXFluxSource,
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
                *item = XField::FluxSource( &node._Op);
                step += 1;
                true
            } else {
                false
            }
        }));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> IGrammar for UniNode< C, USeg>
where
    C: IGrammar,
{
    fn	Match<'p, F: IForge<'p>>(&self, forge: F) -> F
    {
        let  	mut count = U32( 0);
        let  	first = self._Op.First();
        let  	last = if self._Op.IsEmpty() { U32::_X } else { self._Op.Last() };
        let  	origMark = forge.Mark();
        let  	mut currMark = origMark;
        let  	mut f = forge;

        while count < last {
            f = f.Success( currMark);
            f = self._Child.Match( f);
            if f.Ok() {
                let  	m = f.Mark();
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
            f.Success( currMark)
        } else {
            f.Success( origMark).Failure()
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> fmt::Display for UniNode< C, USeg>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        write!( f, "Repeat( {:?})", self._Op)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< C> fmt::Debug for UniNode< C, USeg>
{
    fn	fmt( &self, f: &mut fmt::Formatter< '_>) -> fmt::Result
    {
        fmt::Display::fmt( self, f)
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
