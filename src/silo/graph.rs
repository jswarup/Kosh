use	crate::silo::{ Buff, Stash, U32, USeg };
use	std::fmt::Write;

//--- EdgeConnect -----------------------------------------------------------------------------------------------------------------

pub type EdgeIndex = [U32; 2];

pub struct EdgeConnect
{
    _EdgeStk: Stash< EdgeIndex>,
}

impl EdgeConnect
{
    pub fn	New() -> Self
    {
        Self {
            _EdgeStk: Stash::New(),
        }
    }

    pub fn	SzEdge( &self) -> U32
    {
        self._EdgeStk.Size()
    }

    pub fn	EdgeAt( &self, k: U32) -> EdgeIndex
    {
        self._EdgeStk.Slice()[ k.0 as usize]
    }

    pub fn	RegisterEdge( &mut self, c1: U32, c2: U32, biDirFlg: bool)
    {
        self._EdgeStk.Push( [c1, c2]);
        if biDirFlg {
            self._EdgeStk.Push( [c2, c1]);
        }
    }

    pub fn	Compact( &mut self)
    {
        if self.SzEdge() == U32::_0 {
            return;
        }
        let  	slice = self._EdgeStk.SliceMut();
        slice.sort_unstable();

        let  	mut j = 1;
        for i in 1..slice.len() {
            if slice[ i] != slice[ i - 1] {
                slice[ j] = slice[ i];
                j += 1;
            }
        }
        self._EdgeStk.PopToSize( U32( j as u32));
    }

    pub fn	EdgeIndex( &self, c1: U32, c2: U32) -> U32
    {
        let  	id = [c1, c2];
        let  	slice = self._EdgeStk.Slice();
        match slice.binary_search( &id) {
            Ok( idx) => U32( idx as u32),
            Err( _) => U32::_X,
        }
    }

    pub fn	EdgeSeg( &self, c1: U32) -> USeg
    {
        let  	slice = self._EdgeStk.Slice();
        let  	lowIndex = [c1, U32::_0];
        let  	lb = slice.partition_point( |x| x < &lowIndex);

        let  	highIndex = [c1, U32::_X];
        let  	ub = slice.partition_point( |x| x <= &highIndex);

        USeg::New( U32( lb as u32), U32( ( ub - lb) as u32))
    }

    pub fn	NodeTraverse< F>( &self, nodeId: U32, mut nodeCall: F)
    where
        F: FnMut( U32),
    {
        let  	edSeg = self.EdgeSeg( nodeId);
        edSeg.Traverse( |edInd| {
            let  	ed = self.EdgeAt( edInd);
            nodeCall( ed[ 1]);
        });
    }

    pub fn	Traverse< F>( &self, beg: U32, end: U32, mut edgeCall: F)
    where
        F: FnMut( U32, U32, U32, bool),
    {
        let  	slice = self._EdgeStk.Slice();
        let  	mut it = beg.0;
        let  	end_val = end.0;
        while it < end_val {
            let  	c1 = slice[ it as usize][ 0];
            let  	highIndex = [c1, U32::_X];
            let  	uIt = slice[ it as usize .. end_val as usize].partition_point( |x| x <= &highIndex) as u32 + it;

            while it < uIt {
                let  	val = slice[ it as usize];
                let  	is_last = ( it + 1) == uIt;
                edgeCall( U32( it), val[ 0], val[ 1], is_last);
                it += 1;
            }
        }
    }

    pub fn	TraverseAll< F>( &self, edgeCall: F)
    where
        F: FnMut( U32, U32, U32, bool),
    {
        self.Traverse( U32::_0, self.SzEdge(), edgeCall);
    }

    pub fn	TraverseNode< F>( &self, c1: U32, mut nodeCall: F) -> U32
    where
        F: FnMut( U32, U32),
    {
        let  	edSeg = self.EdgeSeg( c1);
        edSeg.Traverse( |it| {
            let  	val = self.EdgeAt( it);
            nodeCall( it, val[ 1]);
        });
        edSeg.Size()
    }

    pub fn	DumpDot( &self, ostr: &mut String)
    {
        let  	vertFlg = false;
        ostr.push_str( "digraph graphname { \n");
        if !vertFlg {
            ostr.push_str( " rankdir=LR;");
        }
        ostr.push_str( "concentrate=true;\n");
        self.TraverseAll( |_, c1, c2, _| {
            let  	_ = write!( ostr, "{} -> {}  ;\n", c1.0, c2.0);
        });
        ostr.push_str( "}\n");
    }
}

//--- GraphBroadcast --------------------------------------------------------------------------------------------------------------

pub struct GraphBroadcast
{
    _NodeGroupIds: Buff< U32>,
    _FirstNodeIds: Stash< U32>,
}

impl GraphBroadcast
{
    pub fn	New() -> Self
    {
        Self {
            _NodeGroupIds: Buff::New(),
            _FirstNodeIds: Stash::New(),
        }
    }

    pub fn	DoInit( &mut self, mxVert: U32)
    {
        self._NodeGroupIds = Buff::Create( mxVert, |_| U32::_X);
    }

    pub fn	SzGroup( &self) -> U32
    {
        self._FirstNodeIds.Size()
    }

    pub fn	GroupId( &self, c1: U32) -> U32
    {
        self._NodeGroupIds[ c1.0 as usize]
    }

    pub fn	FirstId( &self, g1: U32) -> U32
    {
        self._FirstNodeIds.Slice()[ g1.0 as usize]
    }

    pub fn	SnitchNodeGroupIds( &mut self) -> Buff< U32>
    {
        let  	mut ret = Buff::New();
        self._NodeGroupIds.SwapBuff( &mut ret);
        ret
    }

    pub fn	DoBroadcast< F>( &mut self, c1: U32, mut nextDests: F)
    where
        F: FnMut( U32, U32, bool, &mut Stash< U32>),
    {
        let  	mut curGroup = self.GroupId( c1);
        if curGroup != U32::_X {
            return;
        }

        let  	mut nextStack = Stash::New();
        nextStack.Push( c1);

        while nextStack.Size() > U32::_0 {
            let  	cId = nextStack.Pop().unwrap();
            let  	curSz = nextStack.Size();
            let  	newFlg = curGroup == U32::_X;

            if newFlg {
                self._FirstNodeIds.Push( cId);
                curGroup = self._FirstNodeIds.Size() - U32::_1;
            }
            self._NodeGroupIds[ cId.0 as usize] = curGroup;

            nextDests( cId, curGroup, newFlg, &mut nextStack);

            let  	newSz = nextStack.Size();
            let  	mut cur_idx = curSz;
            for i in curSz.0..newSz.0 {
                let  	node = nextStack.Slice()[ i as usize];
                let  	grId = self.GroupId( node);
                if grId == curGroup {
                    continue;
                }
                assert!( grId == U32::_X);
                nextStack.SliceMut()[ cur_idx.0 as usize] = node;
                self._NodeGroupIds[ node.0 as usize] = curGroup;
                cur_idx = cur_idx + U32::_1;
            }
            nextStack.PopToSize( cur_idx);
        }
    }

    pub fn	DoBroadcastAll< F>( &mut self, mut nextDests: F)
    where
        F: FnMut( U32, U32, bool, &mut Stash< U32>),
    {
        // Buff::Size might return U32, but let's use len() if it implements Deref to slice. 
        // Wait, self._NodeGroupIds is a Buff, we can use self._NodeGroupIds.len()
        for i in 0..self._NodeGroupIds.len() {
            self.DoBroadcast( U32( i as u32), &mut nextDests);
        }
    }

    pub fn	DoPartition< F>( &mut self, edgeConn: &EdgeConnect, mut nodeReport: F)
    where
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
