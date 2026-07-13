//-- binshard.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::{
    shard::{ IGrammar, IForge },
    stalks::{ BinNode, BinOp },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type BinShard< L, R> = BinNode< L, R>;

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IGrammar for BinNode< L, R>
where
    L: IGrammar,
    R: IGrammar,
{
    fn	Match<'p, F: IForge<'p>>(&self, forge: &mut F) -> bool
    {
        match self._Op {
            BinOp::Bor => {
                let  	orig_mark = forge.Mark();
                if self._Left.Match( forge) {
                    return true;
                }
                forge.SetMark( orig_mark);
                if self._Right.Match( forge) {
                    return true;
                }
                false
            }
            BinOp::Less => {
                let  	orig_mark = forge.Mark();
                if self._Left.Match( forge) {
                    // Less (concatenation) means left then right
                    if self._Right.Match( forge) {
                        return true;
                    }
                }
                forge.SetMark( orig_mark);
                false
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

