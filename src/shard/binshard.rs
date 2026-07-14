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
    type Forge = crate::shard::parser::BaseForge;

    fn	Match( &self, parser: &mut crate::shard::Parser, forge: &mut Self::Forge)
    {
        match self._Op {
            BinOp::Bor => {
                let  	m1 = forge.Mark();
                let  	leftRes = self._Left.Parse( parser, forge, m1);
                if leftRes.is_some() {
                    forge.Deposit( leftRes);
                    return;
                }
                
                let  	m2 = forge.Mark();
                let  	rightRes = self._Right.Parse( parser, forge, m2);
                forge.Deposit( rightRes);
            }
            BinOp::Less => {
                let  	m1 = forge.Mark();
                let  	leftRes = self._Left.Parse( parser, forge, m1);
                if let Some( newM) = leftRes {
                    let  	rightRes = self._Right.Parse( parser, forge, newM);
                    forge.Deposit( rightRes);
                } else {
                    forge.Deposit( None);
                }
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

