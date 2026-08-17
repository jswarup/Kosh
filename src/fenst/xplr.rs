//-- fenst/xplr.rs -----------------------------------------------------------------------------------------------------------------
use	crate::silo::{ Buff, U32 };
use	serde::{ Serialize, Deserialize };

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct XplrNodeDto
{
    #[serde( rename = "id")]
    pub _Id:          String,
    #[serde( rename = "name")]
    pub _Name:        String,
    #[serde( rename = "is_leaf")]
    pub _IsLeaf:      bool,
    #[serde( rename = "provider")]
    pub _Provider:    String,
    #[serde( rename = "size")]
    pub _Size:        u64,
    #[serde( rename = "extension")]
    pub _Extension:   String,
}

// ---------------------------------------------------------------------------------------------------------------------------------

#[derive( Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct StreamChunkDto
{
    #[serde( rename = "path")]
    pub _Path:        String,
    #[serde( rename = "offset")]
    pub _Offset:      u64,
    #[serde( rename = "length")]
    pub _Length:      usize,
    #[serde( rename = "total_size")]
    pub _TotalSize:   u64,
    #[serde( rename = "is_eof")]
    pub _IsEof:       bool,
    #[serde( rename = "content")]
    pub _Content:     String,
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
