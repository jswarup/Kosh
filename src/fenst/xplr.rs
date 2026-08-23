//-- fenst/xplr.rs -----------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, U32 };
use	serde::{ Serialize, Deserialize };

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde( rename_all = "snake_case")]
pub struct XplrNodeDto
{
    pub _Id:          String,
    pub _Name:        String,
    pub _IsLeaf:      bool,
    pub _Provider:    String,
    pub _Size:        u64,
    pub _Extension:   String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde( rename_all = "snake_case")]
pub struct StreamChunkDto
{
    pub _Path:        String,
    pub _Offset:      u64,
    pub _Length:      usize,
    pub _TotalSize:   u64,
    pub _IsEof:       bool,
    pub _Content:     String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub trait Xplr
{
    fn	Name( &self) -> &str;
    fn	Path( &self) -> &str;

    fn	IsLeaf( &self) -> bool
    {
        self.AsLeaf().is_some()
    }

    fn	AsLeaf( &self) -> Option< &dyn LeafXplr>
    {
        None
    }

    fn	AsBranch( &self) -> Option< &dyn BranchXplr>
    {
        None
    }

    fn	ToDto( &self, providerScheme: &str) -> XplrNodeDto
    {
        let  	isLeaf = self.IsLeaf();
        let  	size = self.AsLeaf().map( |l| l.Size()).unwrap_or( 0);
        let  	extension = self.AsLeaf().map( |l| l.Extension().to_string()).unwrap_or_default();

        XplrNodeDto {
            _Id:          self.Path().to_string(),
            _Name:        self.Name().to_string(),
            _IsLeaf:      isLeaf,
            _Provider:    providerScheme.to_string(),
            _Size:        size,
            _Extension:   extension,
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
    fn	Children( &self) -> Result< Buff< Box< dyn Xplr>>, String>;
    fn	ChildCount( &self) -> Result< U32, String>;
}

// ---------------------------------------------------------------------------------------------------------------------------------
