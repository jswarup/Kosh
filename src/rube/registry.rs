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
}
