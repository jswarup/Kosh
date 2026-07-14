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

fn	Match( &self, parser: &mut crate::shard::Parser, sink: crate::flux::fluxin::FieldIn< '_>)
    {
        match self._Op {
            BinOp::Bor => {
                let  	m1 = parser.Forge().Mark();
                let  	leftRes = self._Left.Parse( parser, m1, crate::flux::fluxin::FieldIn::Null);
                if leftRes.is_some() {
                    return;
                }
                
                let  	m2 = parser.Forge().Mark();
                self._Right.Parse( parser, m2, crate::flux::fluxin::FieldIn::Null);
            }
            BinOp::Less => {
                let  	m1 = parser.Forge().Mark();
                let  	leftRes = self._Left.Parse( parser, m1, crate::flux::fluxin::FieldIn::Null);
                if let Some( newM) = leftRes {
                    self._Right.Parse( parser, newM, crate::flux::fluxin::FieldIn::Null);
                }
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

