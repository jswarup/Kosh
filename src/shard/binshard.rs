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
    fn	Match< 'p, F: IForge< 'p>>( &self, forge: &mut F)
    {
        match self._Op {
            BinOp::Bor => {
                let  	leftRes = {
                    let  	mut leftForge = self._Left.Forge( forge);
                    self._Left.Match( &mut leftForge);
                    leftForge.Result()
                };
                if leftRes.is_some() {
                    forge.Deposit( leftRes);
                    return;
                }
                
                let  	rightRes = {
                    let  	mut rightForge = self._Right.Forge( forge);
                    self._Right.Match( &mut rightForge);
                    rightForge.Result()
                };
                forge.Deposit( rightRes);
            }
            BinOp::Less => {
                let  	leftRes = {
                    let  	mut leftForge = self._Left.Forge( forge);
                    self._Left.Match( &mut leftForge);
                    leftForge.Result()
                };
                if leftRes.is_none() {
                    forge.Deposit( None);
                    return;
                }
                
                let  	rightRes = {
                    let  	mut rightForge = self._Right.Forge( forge);
                    self._Right.Match( &mut rightForge);
                    rightForge.Result()
                };
                forge.Deposit( rightRes);
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

