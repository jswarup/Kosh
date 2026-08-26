//-- modlayout.rs -------------------------------------------------------------------------------------------------------------------
use	crate::silo::{ EdgeConnect, Stash, U32 };

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
        return Self {
            _ModuleId: moduleId,
            _Name: name.into(),
        };
    }

    #[inline]
    pub fn	ModuleId( &self) -> U32
    {
        return self._ModuleId;
    }

    #[inline]
    pub fn	Name( &self) -> &str
    {
        return &self._Name;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct ModLayout
{
    pub _ModConn: EdgeConnect,
    pub _Names: Stash< String>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for ModLayout
{
    fn	default() -> Self
    {
        return Self::New();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ModLayout
{
    pub fn	New() -> Self
    {
        return Self {
            _ModConn: EdgeConnect::New(),
            _Names: Stash::New(),
        };
    }

    #[inline]
    pub fn	ModConn( &self) -> &EdgeConnect
    {
        return &self._ModConn;
    }

    #[inline]
    pub fn	ModConnMut( &mut self) -> &mut EdgeConnect
    {
        return &mut self._ModConn;
    }

    pub fn	Register( &mut self, name: String) -> U32
    {
        let  	sz = self._Names.Size();
        self._Names.Push( name);
        return sz;
    }

    #[inline]
    pub fn	NameAt( &self, index: U32) -> &str
    {
        return &self._Names.Slice()[index.AsUsize()];
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
