//-- fenst/frescoxplr.rs -----------------------------------------------------------------------------------------------------------
use	crate::fenst::xplr::{ Xplr, LeafXplr, BranchXplr };
use	crate::fenst::provider::XplrProvider;
use	crate::fresco::ExprRepos;
use	crate::silo::U32;

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FrescoLeaf
{
    name:   String,
    path:   String,
    value:  String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl FrescoLeaf
{
    pub fn	New( name: String, path: String, value: String) -> Self
    {
        Self {
            name,
            path,
            value,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for FrescoLeaf
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

impl LeafXplr for FrescoLeaf
{
    fn	Size( &self) -> u64
    {
        self.value.len() as u64
    }

    fn	Extension( &self) -> &str
    {
        "expr"
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FrescoBranch
{
    name:     String,
    path:     String,
    entries:  Vec< Box< dyn Xplr>>,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl FrescoBranch
{
    pub fn	New( name: String, path: String, entries: Vec< Box< dyn Xplr>>) -> Self
    {
        Self {
            name,
            path,
            entries,
        }
    }

    pub fn	FromDemo() -> Self
    {
        let  	mut repos = ExprRepos::NewEmpty();
        let  	varA = repos.VarCreate( "x".to_string(), false);
        let  	varB = repos.VarCreate( "y".to_string(), false);
        let  	realC = repos.RealCreate( 42.0);

        let  	leaf1 = Box::new( FrescoLeaf::New( "var_x".to_string(), "expr://demo/x".to_string(), format!( "Var({})", varA))) as Box< dyn Xplr>;
        let  	leaf2 = Box::new( FrescoLeaf::New( "var_y".to_string(), "expr://demo/y".to_string(), format!( "Var({})", varB))) as Box< dyn Xplr>;
        let  	leaf3 = Box::new( FrescoLeaf::New( "const_42".to_string(), "expr://demo/42".to_string(), format!( "Real({})", realC))) as Box< dyn Xplr>;

        Self::New( "demo".to_string(), "expr://demo".to_string(), vec![ leaf1, leaf2, leaf3 ])
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for FrescoBranch
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

impl BranchXplr for FrescoBranch
{
    fn	Children( &self) -> Result< Vec< Box< dyn Xplr>>, String>
    {
        let  	mut children: Vec< Box< dyn Xplr>> = Vec::new();
        for entry in &self.entries {
            if entry.IsLeaf() {
                if let  	Some( leaf) = entry.AsLeaf() {
                    children.push( Box::new( FrescoLeaf::New( entry.Name().to_string(), entry.Path().to_string(), leaf.Size().to_string())));
                }
            } else {
                children.push( Box::new( FrescoBranch::New( entry.Name().to_string(), entry.Path().to_string(), Vec::new())));
            }
        }
        Ok( children)
    }

    fn	ChildCount( &self) -> Result< U32, String>
    {
        Ok( U32( self.entries.len() as u32))
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
