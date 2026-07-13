//-- binshard.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::{
    shard::{ IGrammar, Parser },
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
    fn	Match<'p>(&self, parser: &mut Parser< 'p>, marker: U32) -> (bool, U32)
    {
        match self._Op {
            BinOp::Bor => {
                let  	(matched, m) = self._Left.Match( parser, marker);
                if matched {
                    return (true, m);
                }
                let  	(matched, m) = self._Right.Match( parser, marker);
                if matched {
                    return (true, m);
                }
                
                (false, marker)
            }
            BinOp::Less => {
                let  	(matched, m) = self._Left.Match( parser, marker);
                if matched {
                    let  	(matched_right, m2) = self._Right.Match( parser, m);
                    if matched_right {
                        return (true, m2);
                    }
                }
                
                (false, marker)
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}


