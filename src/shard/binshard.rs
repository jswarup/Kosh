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

fn	Match( &self, parser: &mut crate::shard::Parser, sink: Option<crate::flux::zflux::ZField< '_>>)
    {
        match self._Op {
            BinOp::Bor => {
                let  	m1 = parser.Forge().Mark();
                let  	leftRes = self._Left.Parse( parser, m1, None);
                if leftRes.is_some() {
                    return;
                }
                
                let  	m2 = parser.Forge().Mark();
                self._Right.Parse( parser, m2, None);
            }
            BinOp::Less => {
                let  	m1 = parser.Forge().Mark();
                let  	leftRes = self._Left.Parse( parser, m1, None);
                if let Some( newM) = leftRes {
                    self._Right.Parse( parser, newM, None);
                }
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

