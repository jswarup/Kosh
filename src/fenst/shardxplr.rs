//-- fenst/shardxplr.rs ------------------------------------------------------------------------------------------------------------
use	crate::fenst::provider::{ VirtualBranch, VirtualLeaf, XplrProvider };
use	crate::fenst::xplr::{ BranchXplr, Xplr };
use	crate::shard::{ Int, Real };
use	crate::silo::Buff;
use	crate::ShardTree;

// ---------------------------------------------------------------------------------------------------------------------------------

pub type ShardLeaf = VirtualLeaf;
pub type ShardBranch = VirtualBranch;

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct ShardProvider;

// ---------------------------------------------------------------------------------------------------------------------------------

impl ShardProvider
{
    pub fn	New() -> Self
    {
        Self
    }

    pub fn	DemoRoot() -> VirtualBranch
    {
        let  	_intGrammar = ShardTree!( Int );
        let  	_realGrammar = ShardTree!( Real );

        let  	leaf1 = Box::new( VirtualLeaf::New( "node_int".to_string(), "ast://demo/int".to_string(), "ast".to_string(), "Int".len() as u64)) as Box< dyn Xplr>;
        let  	leaf2 = Box::new( VirtualLeaf::New( "node_real".to_string(), "ast://demo/real".to_string(), "ast".to_string(), "Real".len() as u64)) as Box< dyn Xplr>;

        VirtualBranch::New( "demo".to_string(), "ast://demo".to_string(), Buff![ leaf1, leaf2 ])
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
        Ok( Box::new( Self::DemoRoot()))
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------
