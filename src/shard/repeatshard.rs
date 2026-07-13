//-- repeatshard.rs -----------------------------------------------------------------------------------------------------------------

use	std::fmt;

use	crate::{
    flux::{ IXFluxSource, xflux::XField },
    shard::{ IGrammar, IForge, parser::ParseForge },
    silo::{ USeg, U32 },
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
    fn	Match<'p, F: IForge<'p>>(&self, forge: &mut F) -> bool
    {
        let  	mut count = U32( 0);
        let  	first = self._Op.First();
        let  	last = if self._Op.IsEmpty() { U32::_X } else { self._Op.Last() };
        
        let  	origMark = forge.Mark();

        while count < last {
            let currMark = forge.Mark();
            if self._Child.Match( forge) {
                let m = forge.Mark();
                if m == currMark {
                    count += U32( 1);
                    break;
                }
                count += U32( 1);
            } else {
                // If it fails, restore the marker to before the failed attempt just in case
                forge.SetMark(currMark);
                break;
            }
        }
        
        if count >= first {
            true
        } else {
            forge.SetMark( origMark);
            false
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
