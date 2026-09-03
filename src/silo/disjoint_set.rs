//-- disjoint_set.rs ----------------------------------------------------------------------------------------------------------------

use	crate::silo::{ Stash, U8, U32, USeg };

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone)]
pub struct DisjointSet
{
    pub _Parent:  Stash< U32>,
    pub _Rank:    Stash< U8>,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IDisjointSet
{
    fn	Size( &self) -> U32;
    fn	Find( &mut self, elem: U32) -> U32;
    fn	Union( &mut self, a: U32, b: U32) -> U32;
    fn	Same( &mut self, a: U32, b: U32) -> bool;
    fn	Grow( &mut self, count: U32);
    fn	Clear( &mut self);
}

//---------------------------------------------------------------------------------------------------------------------------------

impl DisjointSet
{
    pub fn	New() -> Self
    {
        return Self {
            _Parent:  Stash::New(),
            _Rank:    Stash::New(),
        };
    }

    pub fn	WithCapacity( capacity: U32) -> Self
    {
        return Self {
            _Parent:  Stash::WithCapacity( capacity),
            _Rank:    Stash::WithCapacity( capacity),
        };
    }

    #[inline]
    pub fn	FindConst( &self, elem: U32) -> U32
    {
        assert!( elem < self.Size(), "DisjointSet element out of bounds");
        let  	mut curr = elem;
        while self._Parent[curr] != curr {
            curr = self._Parent[curr];
        }
        return curr;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for DisjointSet
{
    fn	default() -> Self
    {
        return Self::New();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IDisjointSet for DisjointSet
{
    #[inline]
    fn	Size( &self) -> U32
    {
        return self._Parent.Size();
    }

    fn	Find( &mut self, elem: U32) -> U32
    {
        assert!( elem < self.Size(), "DisjointSet element out of bounds");
        let  	mut curr = elem;
        while self._Parent[curr] != curr {
            curr = self._Parent[curr];
        }
        let  	root = curr;

        let  	mut node = elem;
        while self._Parent[node] != node {
            let  	next = self._Parent[node];
            self._Parent[node] = root;
            node = next;
        }
        return root;
    }

    fn	Union( &mut self, a: U32, b: U32) -> U32
    {
        let  	rootA = self.Find( a);
        let  	rootB = self.Find( b);
        if rootA == rootB {
            return rootA;
        }

        let  	rankA = self._Rank[rootA];
        let  	rankB = self._Rank[rootB];

        if rankA < rankB {
            self._Parent[rootA] = rootB;
            return rootB;
        } else if rankA > rankB {
            self._Parent[rootB] = rootA;
            return rootA;
        } else {
            self._Parent[rootB] = rootA;
            self._Rank[rootA] += U8::_1;
            return rootA;
        }
    }

    #[inline]
    fn	Same( &mut self, a: U32, b: U32) -> bool
    {
        return self.Find( a) == self.Find( b);
    }

    fn	Grow( &mut self, count: U32)
    {
        let  	start = self.Size();
        USeg::New( U32::_0, count).Traverse( |i| {
            let  	idx = start + i;
            self._Parent.Push( idx);
            self._Rank.Push( U8::_0);
        });
    }

    fn	Clear( &mut self)
    {
        self._Parent.Clear();
        self._Rank.Clear();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
