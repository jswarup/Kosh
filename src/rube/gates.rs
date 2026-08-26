//-- gates.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::rube::{
    layout::Layout,
    module::KernelKind,
    port::PortId,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input NAND Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NandGate
{
    pub _In1: PortId,
    pub _In2: PortId,
    pub _Out: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl NandGate
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	m = layout.AddModuleSimple( name, &[ "in1", "in2" ], &[ "out" ], KernelKind::Nand);
        return Self {
            _In1: layout.InPort( m, 0).unwrap(),
            _In2: layout.InPort( m, 1).unwrap(),
            _Out: layout.OutPort( m, 0).unwrap(),
        };
    }

    #[inline]
    pub fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input AND Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AndGate
{
    pub _In1: PortId,
    pub _In2: PortId,
    pub _Out: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AndGate
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	m = layout.AddModuleSimple( name, &[ "in1", "in2" ], &[ "out" ], KernelKind::And);
        return Self {
            _In1: layout.InPort( m, 0).unwrap(),
            _In2: layout.InPort( m, 1).unwrap(),
            _Out: layout.OutPort( m, 0).unwrap(),
        };
    }

    #[inline]
    pub fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input OR Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OrGate
{
    pub _In1: PortId,
    pub _In2: PortId,
    pub _Out: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl OrGate
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	m = layout.AddModuleSimple( name, &[ "in1", "in2" ], &[ "out" ], KernelKind::Or);
        return Self {
            _In1: layout.InPort( m, 0).unwrap(),
            _In2: layout.InPort( m, 1).unwrap(),
            _Out: layout.OutPort( m, 0).unwrap(),
        };
    }

    #[inline]
    pub fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Input NOT ( Inverter) Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NotGate
{
    pub _In: PortId,
    pub _Out: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl NotGate
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	m = layout.AddModuleSimple( name, &[ "in" ], &[ "out" ], KernelKind::Not);
        return Self {
            _In: layout.InPort( m, 0).unwrap(),
            _Out: layout.OutPort( m, 0).unwrap(),
        };
    }

    #[inline]
    pub fn	In( &self) -> PortId
    {
        return self._In;
    }

    #[inline]
    pub fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input XOR Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct XorGate
{
    pub _In1: PortId,
    pub _In2: PortId,
    pub _Out: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl XorGate
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	m = layout.AddModuleSimple( name, &[ "in1", "in2" ], &[ "out" ], KernelKind::Xor);
        return Self {
            _In1: layout.InPort( m, 0).unwrap(),
            _In2: layout.InPort( m, 1).unwrap(),
            _Out: layout.OutPort( m, 0).unwrap(),
        };
    }

    #[inline]
    pub fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
