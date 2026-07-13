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
    fn	Match<'p, F: IForge<'p>>(&self, forge: F) -> F
    {
        match self._Op {
            BinOp::Bor => {
                let  	orig_mark = forge.Mark();
                let  	mut f = self._Left.Match( forge);
                if f.Ok() {
                    return f;
                }
                f = f.Success( orig_mark); 
                let  	f = self._Right.Match( f);
                if f.Ok() {
                    return f;
                }
                f.Success( orig_mark).Failure()
            }
            BinOp::Less => {
                let  	orig_mark = forge.Mark();
                let  	f = self._Left.Match( forge);
                if f.Ok() {
                    let  	f = self._Right.Match( f);
                    if f.Ok() {
                        return f;
                    }
                    return f.Success( orig_mark).Failure();
                }
                f.Success( orig_mark).Failure()
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}


