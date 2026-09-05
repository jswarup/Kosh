use	crate::rube::{
    interface::{ DataType, ModuleInterface },
    layout::Layout,
    port::PortDir,
};

//---------------------------------------------------------------------------------------------------------------------------------

#[derive( Clone, Debug)]
pub struct PortIntrospection< 'a>
{
    pub _Name:          &'a str,
    pub _Width:         usize,
    pub _DataType:      DataType,
    pub _Direction:     PortDir,
    pub _Documentation: Option< &'static str>,
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IModuleIntrospection
{
    fn	Interface( &self) -> &'static ModuleInterface;
    fn	HierarchyPath( &self, layout: &Layout) -> String;
    fn	ListInports( &self) -> Vec< PortIntrospection< '_>>;
    fn	ListOutports( &self) -> Vec< PortIntrospection< '_>>;
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< T: crate::rube::interface::IModuleInterface + crate::rube::module::IModule> IModuleIntrospection for T
{
    #[inline]
    fn	Interface( &self) -> &'static ModuleInterface
    {
        return <Self as crate::rube::interface::IModuleInterface>::Interface();
    }

    fn	HierarchyPath( &self, layout: &Layout) -> String
    {
        let  	mut path = String::new();
        let  	mut current = Some( self.Id());
        let  	mut names = Vec::new();

        while let Some( modId) = current {
            let  	module = &layout.Modules()[modId.0.AsUsize()];
            names.push( module._Name.as_str());
            current = module._Parent;
        }

        names.reverse();
        path.push_str( &names.join( "."));
        return path;
    }

    fn	ListInports( &self) -> Vec< PortIntrospection< '_>>
    {
        let  	interface = self.Interface();
        return interface._InPorts.iter().map( |p| PortIntrospection {
            _Name:          p._Name,
            _Width:         p._Width,
            _DataType:      p._DataType,
            _Direction:     p._Direction,
            _Documentation: p._Documentation,
        }).collect();
    }

    fn	ListOutports( &self) -> Vec< PortIntrospection< '_>>
    {
        let  	interface = self.Interface();
        return interface._OutPorts.iter().map( |p| PortIntrospection {
            _Name:          p._Name,
            _Width:         p._Width,
            _DataType:      p._DataType,
            _Direction:     p._Direction,
            _Documentation: p._Documentation,
        }).collect();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests
{
    use	super::*;
    use	crate::rube::adder::BusAdder32;

    #[test]
    fn	TestIntrospectionPaths()
    {
        let  	mut layout = Layout::New();
        let  	adder = BusAdder32::New( &mut layout, "MyAdder", None);
        
        let  	path = adder.HierarchyPath( &layout);
        assert_eq!( path, "MyAdder");

        let  	inports = adder.ListInports();
        assert_eq!( inports.len(), 2);
        assert_eq!( inports[0]._Name, "a");

        let  	outports = adder.ListOutports();
        assert_eq!( outports.len(), 2);
        assert_eq!( outports[0]._Name, "sum");
    }
}
