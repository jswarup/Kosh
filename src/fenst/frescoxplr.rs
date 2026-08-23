//-- fenst/frescoxplr.rs -----------------------------------------------------------------------------------------------------------
use	crate::fenst::xplr::{ Xplr, LeafXplr, BranchXplr };
use	crate::fenst::provider::{ VirtualBranch, VirtualLeaf, XplrProvider };
use	crate::fresco::ExprRepos;
use	crate::silo::{ Buff, U32 };

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FrescoLeaf
{
    _Leaf: VirtualLeaf,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl FrescoLeaf
{
    pub fn	New( name: String, path: String, value: String) -> Self
    {
        let  	len = value.len() as u64;
        Self {
            _Leaf: VirtualLeaf::New( name, path, "expr".to_string(), len),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for FrescoLeaf
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

impl LeafXplr for FrescoLeaf
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

pub struct FrescoBranch
{
    _Branch: VirtualBranch,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl FrescoBranch
{
    pub fn	New( name: String, path: String, entries: Buff< Box< dyn Xplr>>) -> Self
    {
        Self {
            _Branch: VirtualBranch::New( name, path, entries),
        }
    }

    pub fn	FromDemo() -> Self
    {
        let  	mut repos = ExprRepos::New();
        let  	varA = repos.VarCreate( "x".to_string(), false);
        let  	varB = repos.VarCreate( "y".to_string(), false);
        let  	realC = repos.RealCreate( 42.0);

        let  	leaf1 = Box::new( FrescoLeaf::New( "var_x".to_string(), "expr://demo/x".to_string(), format!( "Var({})", varA))) as Box< dyn Xplr>;
        let  	leaf2 = Box::new( FrescoLeaf::New( "var_y".to_string(), "expr://demo/y".to_string(), format!( "Var({})", varB))) as Box< dyn Xplr>;
        let  	leaf3 = Box::new( FrescoLeaf::New( "const_42".to_string(), "expr://demo/42".to_string(), format!( "Real({})", realC))) as Box< dyn Xplr>;

        Self::New( "demo".to_string(), "expr://demo".to_string(), Buff![ leaf1, leaf2, leaf3 ])
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for FrescoBranch
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

impl BranchXplr for FrescoBranch
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

pub struct FrescoProvider;

// ---------------------------------------------------------------------------------------------------------------------------------

impl FrescoProvider
{
    pub fn	New() -> Self
    {
        Self
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl XplrProvider for FrescoProvider
{
    fn	Scheme( &self) -> &str
    {
        "expr"
    }

    fn	OpenRoot( &self, _uri: &str) -> Result< Box< dyn BranchXplr>, String>
    {
        Ok( Box::new( FrescoBranch::FromDemo()))
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
