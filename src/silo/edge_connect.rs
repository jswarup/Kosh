//--- EdgeConnect -----------------------------------------------------------------------------------------------------------------

use	crate::silo::{ Stash, U32, USeg };
use	std::fmt::Write;

//---------------------------------------------------------------------------------------------------------------------------------

pub type EdgeIndex = [U32; 2];

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IEdgeConnect
{
    fn	SzEdge( &self) -> U32;
    fn	EdgeAt( &self, k: U32) -> EdgeIndex;
    fn	RegisterEdge( &mut self, c1: U32, c2: U32, biDirFlg: bool);
    fn	Compact( &mut self);
    fn	EdgeIndex( &self, c1: U32, c2: U32) -> U32;
    fn	EdgeSeg( &self, c1: U32) -> USeg;

    fn	NodeTraverse< F>( &self, nodeId: U32, mut nodeCall: F)
    where
        Self: Sized,
        F: FnMut( U32),
    {
        let  	edSeg = self.EdgeSeg( nodeId);
        edSeg.Traverse( |edInd| {
            let  	ed = self.EdgeAt( edInd);
            nodeCall( ed[ 1]);
        });
    }

    fn	Traverse< F>( &self, beg: U32, end: U32, edgeCall: F)
    where
        Self: Sized,
        F: FnMut( U32, U32, U32, bool);

    fn	TraverseAll< F>( &self, edgeCall: F)
    where
        Self: Sized,
        F: FnMut( U32, U32, U32, bool),
    {
        self.Traverse( U32::_0, self.SzEdge(), edgeCall);
    }

    fn	TraverseNode< F>( &self, c1: U32, mut nodeCall: F) -> U32
    where
        Self: Sized,
        F: FnMut( U32, U32),
    {
        let  	edSeg = self.EdgeSeg( c1);
        edSeg.Traverse( |it| {
            let  	val = self.EdgeAt( it);
            nodeCall( it, val[ 1]);
        });
        return edSeg.Size();
    }

    fn	DumpDot( &self, ostr: &mut String);
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct EdgeConnect
{
    _EdgeStk: Stash< EdgeIndex>,
}

impl EdgeConnect
{
    pub fn	New() -> Self
    {
        Self {
            _EdgeStk: Stash::< EdgeIndex>::New(),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IEdgeConnect for EdgeConnect
{
    fn	SzEdge( &self) -> U32
    {
        self._EdgeStk.Size()
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EdgeAt( &self, k: U32) -> EdgeIndex
    {
        self._EdgeStk[k]
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	RegisterEdge( &mut self, c1: U32, c2: U32, biDirFlg: bool)
    {
        self._EdgeStk.Push( [c1, c2]);
        if biDirFlg {
            self._EdgeStk.Push( [c2, c1]);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Compact( &mut self)
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
        self._EdgeStk.PopToSize( j);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EdgeIndex( &self, c1: U32, c2: U32) -> U32
    {
        let  	id = [c1, c2];
        match self._EdgeStk.Arr().BinarySearch( &id) {
            Ok( idx) => idx,
            Err( _)  => U32::_X,
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	EdgeSeg( &self, c1: U32) -> USeg
    {
        let  	arr = self._EdgeStk.Arr();
        let  	lowIndex = [c1, U32::_0];
        let  	lb = arr.USeg().LowerBound( |i| arr[i] < lowIndex);

        let  	highIndex = [c1, U32::_X];
        let  	ub = arr.USeg().LowerBound( |i| arr[i] <= highIndex);

        return USeg::New( lb, ub - lb);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	Traverse< F>( &self, beg: U32, end: U32, mut edgeCall: F)
    where
        Self: Sized,
        F: FnMut( U32, U32, U32, bool),
    {
        let  	slice = self._EdgeStk.Slice();
        let  	mut it = beg.0;
        let  	endVal = end.0;
        while it < endVal {
            let  	c1 = slice[ it as usize][ 0];
            let  	highIndex = [c1, U32::_X];
            let  	uIt = slice[ it as usize .. endVal as usize].partition_point( |x| x <= &highIndex) as u32 + it;

            while it < uIt {
                let  	val = slice[ it as usize];
                let  	isLast = ( it + 1) == uIt;
                edgeCall( U32( it), val[ 0], val[ 1], isLast);
                it += 1;
            }
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	DumpDot( &self, ostr: &mut String)
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

//---------------------------------------------------------------------------------------------------------------------------------
