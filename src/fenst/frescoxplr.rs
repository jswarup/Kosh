//-- fenst/frescoxplr.rs -----------------------------------------------------------------------------------------------------------
use	crate::fenst::provider::{ VirtualBranch, VirtualLeaf, XplrProvider };
use	crate::fenst::xplr::{ BranchXplr, Xplr };
use	crate::fresco::ExprRepos;
use	crate::silo::Buff;

// ---------------------------------------------------------------------------------------------------------------------------------

pub type FrescoLeaf = VirtualLeaf;
pub type FrescoBranch = VirtualBranch;

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FrescoProvider;

// ---------------------------------------------------------------------------------------------------------------------------------

impl FrescoProvider
{
    pub fn	New() -> Self
    {
        Self
    }

    pub fn	DemoRoot() -> VirtualBranch
    {
        let  	mut repos = ExprRepos::New();
        let  	varA = repos.VarCreate( "x".to_string(), false);
        let  	varB = repos.VarCreate( "y".to_string(), false);
        let  	realC = repos.RealCreate( 42.0);

        let  	leaf1 = Box::new( VirtualLeaf::New( "var_x".to_string(), "expr://demo/x".to_string(), "expr".to_string(), format!( "Var({})", varA).len() as u64)) as Box< dyn Xplr>;
        let  	leaf2 = Box::new( VirtualLeaf::New( "var_y".to_string(), "expr://demo/y".to_string(), "expr".to_string(), format!( "Var({})", varB).len() as u64)) as Box< dyn Xplr>;
        let  	leaf3 = Box::new( VirtualLeaf::New( "const_42".to_string(), "expr://demo/42".to_string(), "expr".to_string(), format!( "Real({})", realC).len() as u64)) as Box< dyn Xplr>;

        VirtualBranch::New( "demo".to_string(), "expr://demo".to_string(), Buff![ leaf1, leaf2, leaf3 ])
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
        Ok( Box::new( Self::DemoRoot()))
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
