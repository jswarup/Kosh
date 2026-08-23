//-- fenst/shardxplr.rs ------------------------------------------------------------------------------------------------------------
use	crate::fenst::xplr::{ Xplr, LeafXplr, BranchXplr };
use	crate::fenst::provider::{ VirtualBranch, VirtualLeaf, XplrProvider };
use	crate::shard::{ Real, Int };
use	crate::silo::{ Buff, U32 };
use	crate::ShardTree;

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct ShardLeaf
{
    _Leaf: VirtualLeaf,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl ShardLeaf
{
    pub fn	New( name: String, path: String, grammarType: String) -> Self
    {
        let  	len = grammarType.len() as u64;
        Self {
            _Leaf: VirtualLeaf::New( name, path, "ast".to_string(), len),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for ShardLeaf
{
    fn	Name( &self) -> &str
    {
        self._Leaf.Name()
    }

    fn	Path( &self) -> &str
    {
        self._Leaf.Path()
    }

    fn	AsLeaf( &self) -> Option< &dyn LeafXplr>
    {
        Some( self)
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl LeafXplr for ShardLeaf
{
    fn	Size( &self) -> u64
    {
        self._Leaf.Size()
    }

    fn	Extension( &self) -> &str
    {
        self._Leaf.Extension()
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct ShardBranch
{
    _Branch: VirtualBranch,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl ShardBranch
{
    pub fn	New( name: String, path: String, children: Buff< Box< dyn Xplr>>) -> Self
    {
        Self {
            _Branch: VirtualBranch::New( name, path, children),
        }
    }

    pub fn	FromDemo() -> Self
    {
        let  	_intGrammar = ShardTree!( Int );
        let  	_realGrammar = ShardTree!( Real );

        let  	leaf1 = Box::new( ShardLeaf::New( "node_int".to_string(), "ast://demo/int".to_string(), "Int".to_string())) as Box< dyn Xplr>;
        let  	leaf2 = Box::new( ShardLeaf::New( "node_real".to_string(), "ast://demo/real".to_string(), "Real".to_string())) as Box< dyn Xplr>;

        Self::New( "demo".to_string(), "ast://demo".to_string(), Buff![ leaf1, leaf2 ])
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for ShardBranch
{
    fn	Name( &self) -> &str
    {
        self._Branch.Name()
    }

    fn	Path( &self) -> &str
    {
        self._Branch.Path()
    }

    fn	AsBranch( &self) -> Option< &dyn BranchXplr>
    {
        Some( self)
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl BranchXplr for ShardBranch
{
    fn	Children( &self) -> Result< Buff< Box< dyn Xplr>>, String>
    {
        self._Branch.Children()
    }

    fn	ChildCount( &self) -> Result< U32, String>
    {
        self._Branch.ChildCount()
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct ShardProvider;

// ---------------------------------------------------------------------------------------------------------------------------------

impl ShardProvider
{
    pub fn	New() -> Self
    {
        Self
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl XplrProvider for ShardProvider
{
    fn	Scheme( &self) -> &str
    {
        "ast"
    }

    fn	OpenRoot( &self, _uri: &str) -> Result< Box< dyn BranchXplr>, String>
    {
        Ok( Box::new( ShardBranch::FromDemo()))
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
