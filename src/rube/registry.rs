//-- registry.rs ---------------------------------------------------------------------------------------------------------------------

use	std::{ collections::HashMap, sync::Arc };
use	crate::{ rube::reg::Reg, silo::U32 };

//---------------------------------------------------------------------------------------------------------------------------------

pub type CustomKernelFn = Arc< dyn Fn( &[Reg], &mut [Reg]) + Send + Sync>;

//---------------------------------------------------------------------------------------------------------------------------------

pub struct KernelRegistry
{
    pub _Map: HashMap< &'static str, CustomKernelFn>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for KernelRegistry
{
    fn	default() -> Self
    {
        Self::Default()
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl KernelRegistry
{
    pub fn	New() -> Self
    {
        return Self {
            _Map: HashMap::new(),
        };
    }

    pub fn	Default() -> Self
    {
        let  	mut registry = Self::New();

        registry._Map.insert( "BusAdder32_Kernel", Arc::new( |inVals: &[Reg], outVals: &mut [Reg]| {
            let  	aVal = inVals[0].Val();
            let  	bVal = inVals[1].Val();
            let  	sum = aVal.wrapping_add( bVal) & 0xFFFF_FFFF;
            let  	carry = ( aVal + bVal) > 0xFFFF_FFFF;

            outVals[0] = Reg::FromU32( U32( sum as u32));
            outVals[1] = Reg::FromBool( carry);
        }));

        return registry;
    }

    pub fn	FindOrInternStaticName( name: &str) -> &'static str
    {
        const KNOWN: &[&str] = &[
            "BusAdder32_Kernel",
        ];
        let  	mut found = None;
        KNOWN.iter().for_each( |&k| {
            if k == name {
                found = Some( k);
            }
        });
        if let Some( s) = found {
            return s;
        }

        use std::sync::Mutex;
        static INTERNED: Mutex< Option< std::collections::HashSet< &'static str>>> = Mutex::new( None);
        let  	mut guard = INTERNED.lock().unwrap();
        let  	set = guard.get_or_insert_with( std::collections::HashSet::new);
        if let Some( &existing) = set.get( name) {
            return existing;
        }
        let  	leaked: &'static str = Box::leak( name.to_string().into_boxed_str());
        set.insert( leaked);
        return leaked;
    }
}
