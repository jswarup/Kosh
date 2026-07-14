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
                    let  	mark = forge.Mark();
                    let  	mut leftForge = self._Left.Forge( forge.Parser());
                    leftForge.SetMark( mark);
                    self._Left.Match( &mut leftForge);
                    leftForge.Result()
                };
                if leftRes.is_some() {
                    forge.Deposit( leftRes);
                    return;
                }
                
                let  	rightRes = {
                    let  	mark = forge.Mark();
                    let  	mut rightForge = self._Right.Forge( forge.Parser());
                    rightForge.SetMark( mark);
                    self._Right.Match( &mut rightForge);
                    rightForge.Result()
                };
                forge.Deposit( rightRes);
            }
            BinOp::Less => {
                let  	leftRes = {
                    let  	mark = forge.Mark();
                    let  	mut leftForge = self._Left.Forge( forge.Parser());
                    leftForge.SetMark( mark);
                    self._Left.Match( &mut leftForge);
                    leftForge.Result()
                };
                if let Some( newM) = leftRes {
                    let  	rightRes = {
                        let  	mut rightForge = self._Right.Forge( forge.Parser());
                        rightForge.SetMark( newM);
                        self._Right.Match( &mut rightForge);
                        rightForge.Result()
                    };
                    forge.Deposit( rightRes);
                } else {
                    forge.Deposit( None);
                }
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

