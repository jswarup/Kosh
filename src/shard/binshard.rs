//-- binshard.rs -----------------------------------------------------------------------------------------------------------------------
use	crate::flux::fluximport::FieldImp;
use	crate::shard::Parser;
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

fn	Match( &self, parser: &mut Parser)
    {
        match self._Op {
            BinOp::Bor => {
                let  	m1 = parser.CurrMark();
                let  	leftRes = parser.ParseGrammar( &self._Left, m1);
                if leftRes.is_some() {
                    return;
                }

                let  	m2 = parser.CurrMark();
                parser.ParseGrammar( &self._Right, m2);
            }
            BinOp::Less => {
                let  	m1 = parser.CurrMark();
                let  	leftRes = parser.ParseGrammar( &self._Left, m1);
                if let Some( newM) = leftRes {
                    parser.ParseGrammar( &self._Right, newM);
                }
            }
            _ => panic!( "Unsupported operator in BinShard Match"),
        }
    }
}

