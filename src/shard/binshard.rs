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
                let  	leftRes = {
                    let  	mark = forge.Mark();
                    let  	mut leftForge = <L as IGrammar>::Forge::New();
                    leftForge.SetMark( mark);
                    parser.PushForge( forge as *mut _ as *mut dyn IForge);
                    self._Left.Match( parser, &mut leftForge);
                    parser.PopForge();
                    leftForge.Result()
                };
                if leftRes.is_some() {
                    forge.Deposit( leftRes);
                    return;
                }
                
                let  	rightRes = {
                    let  	mark = forge.Mark();
                    let  	mut rightForge = <R as IGrammar>::Forge::New();
                    rightForge.SetMark( mark);
                    parser.PushForge( forge as *mut _ as *mut dyn IForge);
                    self._Right.Match( parser, &mut rightForge);
                    parser.PopForge();
                    rightForge.Result()
                };
                forge.Deposit( rightRes);
            }
            BinOp::Less => {
                let  	leftRes = {
                    let  	mark = forge.Mark();
                    let  	mut leftForge = <L as IGrammar>::Forge::New();
                    leftForge.SetMark( mark);
                    parser.PushForge( forge as *mut _ as *mut dyn IForge);
                    self._Left.Match( parser, &mut leftForge);
                    parser.PopForge();
                    leftForge.Result()
                };
                if let Some( newM) = leftRes {
                    let  	rightRes = {
                        let  	mut rightForge = <R as IGrammar>::Forge::New();
                        rightForge.SetMark( newM);
                        parser.PushForge( forge as *mut _ as *mut dyn IForge);
                        self._Right.Match( parser, &mut rightForge);
                        parser.PopForge();
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

