//-- modlayout.rs -------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ EdgeConnect, Stash, U32 };

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IModule
{
    fn	ModuleId( &self) -> U32;
    fn	Name( &self) -> &str;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct Module
{
    pub _ModuleId: U32,
    pub _Name: String,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Module
{
    pub fn	New( moduleId: U32, name: impl Into< String>) -> Self
    {
        Self {
            _ModuleId: moduleId,
            _Name: name.into(),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IModule for Module
{
    fn	ModuleId( &self) -> U32
    {
        return self._ModuleId;
    }

    fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IModLayout
{
    fn	ModConn( &self) -> &EdgeConnect;
    fn	ModConnMut( &mut self) -> &mut EdgeConnect;
    fn	Register( &mut self, name: String) -> U32;
    fn	NameAt( &self, index: U32) -> &str;
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ModLayout
{
    pub _ModConn: EdgeConnect,
    pub _Names: Stash< String>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ModLayout
{
    pub fn	New() -> Self
    {
        Self {
            _ModConn: EdgeConnect::New(),
            _Names: Stash::New(),
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IModLayout for ModLayout
{
    fn	ModConn( &self) -> &EdgeConnect
    {
        return &self._ModConn;
    }

    fn	ModConnMut( &mut self) -> &mut EdgeConnect
    {
        return &mut self._ModConn;
    }

    fn	Register( &mut self, name: String) -> U32
    {
        let  	sz = self._Names.Size();
        self._Names.Push( name);
        return sz;
    }

    fn	NameAt( &self, index: U32) -> &str
    {
        return &self._Names.Slice()[index.AsUsize()];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
