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

fn	Match( &self, parser: &mut crate::shard::Parser)
    {
        match self._Op {
            BinOp::Bor => {
                let  	m1 = parser.Forge().Mark();
                let  	leftRes = self._Left.Parse( parser, m1);
                if leftRes.is_some() {
                    return;
                }
                
                let  	m2 = parser.Forge().Mark();
                self._Right.Parse( parser, m2);
            }
            BinOp::Less => {
                let  	m1 = parser.Forge().Mark();
                let  	leftRes = self._Left.Parse( parser, m1);
                if let Some( newM) = leftRes {
                    self._Right.Parse( parser, newM);
                }
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

