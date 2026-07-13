//-- binshard.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::{
    shard::{ IGrammar, IForge },
    stalks::{ BinNode, BinOp },
    silo::U32,
};

//---------------------------------------------------------------------------------------------------------------------------------

pub type BinShard< L, R> = BinNode< L, R>;

//---------------------------------------------------------------------------------------------------------------------------------

impl< L, R> IGrammar for BinNode< L, R>
where
    L: IGrammar,
    R: IGrammar,
{
    fn	Match< 'p, F: IForge< 'p>>(&self, forge: &mut F) -> Option< U32>
    {
        match self._Op {
            BinOp::Bor => {
                let  	res = {
                    let  	mut left_forge = self._Left.Forge( forge);
                    let  	res = self._Left.Match( &mut left_forge);
                    left_forge.Deposit( res);
                    res
                };
                if res.is_some() {
                    forge.Deposit( res);
                    return res;
                }
                
                let  	res = {
                    let  	mut right_forge = self._Right.Forge( forge);
                    let  	res = self._Right.Match( &mut right_forge);
                    right_forge.Deposit( res);
                    res
                };
                forge.Deposit( res);
                res
            }
            BinOp::Less => {
                let  	m1_res = {
                    let  	mut left_forge = self._Left.Forge( forge);
                    let  	res = self._Left.Match( &mut left_forge);
                    left_forge.Deposit( res);
                    res
                };
                if m1_res.is_none() {
                    forge.Deposit( None);
                    return None;
                }
                
                let  	m2 = {
                    let  	mut right_forge = self._Right.Forge( forge);
                    let  	res = self._Right.Match( &mut right_forge);
                    right_forge.Deposit( res);
                    res
                };
                forge.Deposit( m2);
                m2
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

