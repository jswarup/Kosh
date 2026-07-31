//-- fenst/xplr.rs -----------------------------------------------------------------------------------------------------------------
use	crate::silo::U32;
use	serde::{ Serialize, Deserialize };

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct XplrNodeDto
{
    pub id:          String,
    pub name:        String,
    pub is_leaf:     bool,
    pub provider:    String,
    pub size:        u64,
    pub extension:   String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StreamChunkDto
{
    pub path:        String,
    pub offset:      u64,
    pub length:      usize,
    pub total_size:  u64,
    pub is_eof:      bool,
    pub content:     String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub trait Xplr
{
    fn	Name( &self) -> &str;
    fn	Path( &self) -> &str;
    fn	IsLeaf( &self) -> bool;
    fn	AsLeaf( &self) -> Option< &dyn LeafXplr>;
    fn	AsBranch( &self) -> Option< &dyn BranchXplr>;

    fn	ToDto( &self, providerScheme: &str) -> XplrNodeDto
    {
        let  	isLeaf = self.IsLeaf();
        let  	size = self.AsLeaf().map( |l| l.Size()).unwrap_or( 0);
        let  	extension = self.AsLeaf().map( |l| l.Extension().to_string()).unwrap_or_default();

        XplrNodeDto {
            id:          self.Path().to_string(),
            name:        self.Name().to_string(),
            is_leaf:     isLeaf,
            provider:    providerScheme.to_string(),
            size,
            extension,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub trait LeafXplr: Xplr
{
    fn	Size( &self) -> u64;
    fn	Extension( &self) -> &str;
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub trait BranchXplr: Xplr
{
    fn	Children( &self) -> Result< Vec< Box< dyn Xplr>>, String>;
    fn	ChildCount( &self) -> Result< U32, String>;
}

// ---------------------------------------------------------------------------------------------------------------------------------
