//-- fenst/provider.rs -------------------------------------------------------------------------------------------------------------
use	crate::fenst::xplr::{ Xplr, LeafXplr, BranchXplr };
use	crate::fenst::fsxplr::FsBranch;
use	crate::silo::{ Buff, U32 };
use	std::collections::HashMap;
use	std::sync::{ Arc, RwLock };

// ---------------------------------------------------------------------------------------------------------------------------------

pub trait XplrProvider: Send + Sync
{
    fn	Scheme( &self) -> &str;
    fn	OpenRoot( &self, uri: &str) -> Result< Box< dyn BranchXplr>, String>;
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Generic in-memory virtual leaf node.
pub struct VirtualLeaf
{
    pub _Name:      String,
    pub _Path:      String,
    pub _Extension: String,
    pub _Size:      u64,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl VirtualLeaf
{
    pub fn	New( name: String, path: String, extension: String, size: u64) -> Self
    {
        Self {
            _Name:      name,
            _Path:      path,
            _Extension: extension,
            _Size:      size,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for VirtualLeaf
{
    fn	Name( &self) -> &str
    {
        &self._Name
    }

    fn	Path( &self) -> &str
    {
        &self._Path
    }

    fn	AsLeaf( &self) -> Option< &dyn LeafXplr>
    {
        Some( self)
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl LeafXplr for VirtualLeaf
{
    fn	Size( &self) -> u64
    {
        self._Size
    }

    fn	Extension( &self) -> &str
    {
        &self._Extension
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

/// Generic in-memory virtual branch node holding child Xplr nodes.
pub struct VirtualBranch
{
    pub _Name:     String,
    pub _Path:     String,
    pub _Children: Buff< Box< dyn Xplr>>,
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl VirtualBranch
{
    pub fn	New( name: String, path: String, children: Buff< Box< dyn Xplr>>) -> Self
    {
        Self {
            _Name:     name,
            _Path:     path,
            _Children: children,
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl Xplr for VirtualBranch
{
    fn	Name( &self) -> &str
    {
        &self._Name
    }

    fn	Path( &self) -> &str
    {
        &self._Path
    }

    fn	AsBranch( &self) -> Option< &dyn BranchXplr>
    {
        Some( self)
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl BranchXplr for VirtualBranch
{
    fn	Children( &self) -> Result< Buff< Box< dyn Xplr>>, String>
    {
        let  	children = Buff::Create( self._Children.Size(), |i| {
            let  	child = &self._Children[i.AsUsize()];
            if let Some( leaf) = child.AsLeaf() {
                let  	leafBox: Box< dyn Xplr> = Box::new( VirtualLeaf::New(
                    child.Name().to_string(),
                    child.Path().to_string(),
                    leaf.Extension().to_string(),
                    leaf.Size(),
                ));
                leafBox
            } else {
                let  	branchBox: Box< dyn Xplr> = Box::new( VirtualBranch::New(
                    child.Name().to_string(),
                    child.Path().to_string(),
                    Buff::New(),
                ));
                branchBox
            }
        });
        Ok( children)
    }

    fn	ChildCount( &self) -> Result< U32, String>
    {
        Ok( self._Children.Size())
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct FsProvider;

// ---------------------------------------------------------------------------------------------------------------------------------

impl FsProvider
{
    pub fn	New() -> Self
    {
        Self
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

impl XplrProvider for FsProvider
{
    fn	Scheme( &self) -> &str
    {
        "file"
    }

    fn	OpenRoot( &self, uri: &str) -> Result< Box< dyn BranchXplr>, String>
    {
        let  	path = if uri.starts_with( "file://") {
            &uri[7..]
        } else {
            uri
        };

        Ok( Box::new( FsBranch::New( path.to_string())))
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub struct XplrRegistry
{
    _Providers: HashMap< String, Box< dyn XplrProvider>>,
}

use	crate::fenst::frescoxplr::FrescoProvider;
use	crate::fenst::shardxplr::ShardProvider;

// ---------------------------------------------------------------------------------------------------------------------------------

impl XplrRegistry
{
    pub fn	New() -> Self
    {
        let  	mut registry = Self {
            _Providers: HashMap::new(),
        };
        registry.Register( Box::new( FsProvider::New()));
        registry.Register( Box::new( FrescoProvider::New()));
        registry.Register( Box::new( ShardProvider::New()));
        registry
    }

    pub fn	Register( &mut self, provider: Box< dyn XplrProvider>)
    {
        let  	scheme = provider.Scheme().to_string();
        self._Providers.insert( scheme, provider);
    }

    pub fn	GetProvider( &self, scheme: &str) -> Option< &dyn XplrProvider>
    {
        self._Providers.get( scheme).map( |p| p.as_ref())
    }

    pub fn	Schemes( &self) -> Buff< String>
    {
        let  	mut keys: Buff< String> = self._Providers.keys().cloned().collect();
        keys.sort();
        keys
    }

    pub fn	OpenRoot( &self, uri: &str) -> Result< ( String, Box< dyn BranchXplr>), String>
    {
        let  	scheme = if let  	Some( idx) = uri.find( "://") {
            &uri[..idx]
        } else {
            "file"
        };

        if let  	Some( provider) = self.GetProvider( scheme) {
            let  	root = provider.OpenRoot( uri)?;
            Ok( ( scheme.to_string(), root))
        } else {
            Err( format!( "No provider registered for scheme: {}", scheme))
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------------------------

pub type SharedXplrRegistry = Arc< RwLock< XplrRegistry>>;

pub fn	CreateDefaultRegistry() -> SharedXplrRegistry
{
    Arc::new( RwLock::new( XplrRegistry::New()))
}

// ---------------------------------------------------------------------------------------------------------------------------------
