//-- fenst/shardxplr.rs ------------------------------------------------------------------------------------------------------------
use	crate::fenst::xplr::{ Xplr, LeafXplr, BranchXplr };
use	crate::fenst::provider::XplrProvider;
use	crate::silo::{ Buff, U32 };

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct ShardLeaf
{
    name:   String,
    path:   String,
    token:  String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl ShardLeaf
{
    pub fn	New( name: String, path: String, token: String) -> Self
    {
        Self {
            name,
            path,
            token,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for ShardLeaf
{
    fn	Name( &self) -> &str
    {
        &self.name
    }

    fn	Path( &self) -> &str
    {
        &self.path
    }

    fn	IsLeaf( &self) -> bool
    {
        true
    }

    fn	AsLeaf( &self) -> Option< &dyn LeafXplr>
    {
        Some( self)
    }

    fn	AsBranch( &self) -> Option< &dyn BranchXplr>
    {
        None
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl LeafXplr for ShardLeaf
{
    fn	Size( &self) -> u64
    {
        self.token.len() as u64
    }

    fn	Extension( &self) -> &str
    {
        "ast"
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct ShardBranch
{
    name:     String,
    path:     String,
    children: Buff< Box< dyn Xplr>>,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl ShardBranch
{
    pub fn	New( name: String, path: String, children: Buff< Box< dyn Xplr>>) -> Self
    {
        Self {
            name,
            path,
            children,
        }
    }

    pub fn	FromDemo() -> Self
    {
        let  	leaf1 = Box::new( ShardLeaf::New( "identifier".to_string(), "ast://demo/id".to_string(), "Token(Ident)".to_string())) as Box< dyn Xplr>;
        let  	leaf2 = Box::new( ShardLeaf::New( "literal".to_string(), "ast://demo/lit".to_string(), "Token(Number)".to_string())) as Box< dyn Xplr>;
        let  	leaf3 = Box::new( ShardLeaf::New( "operator".to_string(), "ast://demo/op".to_string(), "Token(Plus)".to_string())) as Box< dyn Xplr>;

        Self::New( "ast_root".to_string(), "ast://demo".to_string(), Buff![ leaf1, leaf2, leaf3 ])
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for ShardBranch
{
    fn	Name( &self) -> &str
    {
        &self.name
    }

    fn	Path( &self) -> &str
    {
        &self.path
    }

    fn	IsLeaf( &self) -> bool
    {
        false
    }

    fn	AsLeaf( &self) -> Option< &dyn LeafXplr>
    {
        None
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
        let  	mut res: Buff< Box< dyn Xplr>> = Buff::NewEmpty();
        for child in &self.children {
            if child.IsLeaf() {
                if let  	Some( leaf) = child.AsLeaf() {
                    res.Push( Box::new( ShardLeaf::New( child.Name().to_string(), child.Path().to_string(), leaf.Size().to_string())));
                }
            } else {
                res.Push( Box::new( ShardBranch::New( child.Name().to_string(), child.Path().to_string(), Buff::NewEmpty())));
            }
        }
        Ok( res)
    }

    fn	ChildCount( &self) -> Result< U32, String>
    {
        Ok( self.children.Size())
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
