//-- fenst/provider.rs -------------------------------------------------------------------------------------------------------------
use	crate::fenst::xplr::BranchXplr;
use	crate::fenst::fsxplr::FsBranch;
use	crate::silo::Buff;
use	std::collections::HashMap;
use	std::sync::{ Arc, RwLock };

// ---------------------------------------------------------------------------------------------------------------------------------

pub trait XplrProvider: Send + Sync
{
    fn	Scheme( &self) -> &str;
    fn	OpenRoot( &self, uri: &str) -> Result< Box< dyn BranchXplr>, String>;
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
