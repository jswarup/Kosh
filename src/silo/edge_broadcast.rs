//--- EdgeBroadcast --------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, EdgeConnect, IEdgeConnect, Stash, U32 };

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IEdgeBroadcast
{
    fn	SzGroup( &self) -> U32;
    fn	GroupId( &self, c1: U32) -> U32;
    fn	FirstId( &self, g1: U32) -> U32;
    fn	SnitchNodeGroupIds( &mut self) -> Buff< U32>;

    fn	DoBroadcast< F>( &mut self, c1: U32, nextDests: F)
    where
        Self: Sized,
        F: FnMut( U32, U32, bool, &mut Stash< U32>);

    fn	DoBroadcastAll< F>( &mut self, nextDests: F)
    where
        Self: Sized,
        F: FnMut( U32, U32, bool, &mut Stash< U32>);

    fn	DoPartition< F>( &mut self, edgeConn: &EdgeConnect, mut nodeReport: F)
    where
        Self: Sized,
        F: FnMut( U32, U32, bool) -> bool,
    {
        self.DoBroadcastAll( |elemId, grId, nwFlg, nextSeqStk| {
            let  	res = nodeReport( elemId, grId, nwFlg);
            if !res {
                return;
            }
            edgeConn.NodeTraverse( elemId, |nextElem| {
                nextSeqStk.Push( nextElem);
            });
        });
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct EdgeBroadcast
{
    _NodeGroupIds: Buff< U32>,
    _FirstNodeIds: Stash< U32>,
}

impl EdgeBroadcast 
{
    pub fn	New( mxVert: U32) -> Self
    {
        Self {
            _NodeGroupIds: Buff::Create( mxVert, |_| U32::_X),
            _FirstNodeIds: Stash::New(),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IEdgeBroadcast for EdgeBroadcast
{
    fn	SzGroup( &self) -> U32
    {
        self._FirstNodeIds.Size()
    }
 
    fn	GroupId( &self, c1: U32) -> U32
    {
        self._NodeGroupIds[ c1.Grab( 0, 31)]
    }

    fn	FirstId( &self, g1: U32) -> U32
    {
        self._FirstNodeIds[ g1]
    }

    fn	SnitchNodeGroupIds( &mut self) -> Buff< U32>
    {
        let  	mut ret = Buff::< U32>::New();
        self._NodeGroupIds.SwapBuff( &mut ret);
        return ret;
    }

    fn	DoBroadcast< F>( &mut self, c1: U32, mut nextDests: F)
    where
        Self: Sized,
        F: FnMut( U32, U32, bool, &mut Stash< U32>),
    {
        let  	mut curGroup = self.GroupId( c1);
        if curGroup != U32::_X {
            return;
        }

        let  	mut nextStack = Stash::< U32>::New();
        nextStack.Push( c1);

        while nextStack.Size() > U32::_0 {
            let  	cId = nextStack.Pop().unwrap();
            let  	curSz = nextStack.Size();
            let  	newFlg = curGroup == U32::_X;

            if newFlg {
                self._FirstNodeIds.Push( cId);
                curGroup = self._FirstNodeIds.Size() - U32::_1;
            }
            self._NodeGroupIds[ cId.Grab( 0, 31)] = curGroup;

            nextDests( cId, curGroup, newFlg, &mut nextStack);

            let  	newSz = nextStack.Size();
            let  	mut cur_idx = curSz;
            for i in curSz.0..newSz.0 {
                let  	node = nextStack[ U32( i)];
                let  	grId = self.GroupId( node);
                if grId == curGroup {
                    continue;
                }
                assert!( grId == U32::_X);
                nextStack[ cur_idx] = node;
                self._NodeGroupIds[ node.Grab( 0, 31)] = curGroup;
                cur_idx = cur_idx + U32::_1;
            }
            nextStack.PopToSize( cur_idx);
        }
    }

    fn	DoBroadcastAll< F>( &mut self, mut nextDests: F)
    where
        Self: Sized,
        F: FnMut( U32, U32, bool, &mut Stash< U32>),
    {
        for i in 0..self._NodeGroupIds.len() {
            self.DoBroadcast( U32( i as u32), &mut nextDests);
        }
    }
}