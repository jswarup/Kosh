//-- gates.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::rube::{
    layout::Layout,
    module::KernelKind,
    port::{ PortDesc, PortId },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[inline]
fn	CreateGate2( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>, kind: KernelKind) -> ( PortId, PortId, PortId)
{
    let  	m = layout.AddModule(
        name,
        parent,
        &[ PortDesc::Bool( "in1"), PortDesc::Bool( "in2") ],
        &[ PortDesc::Bool( "out") ],
        kind,
    );
    return (
        layout.InPort( m, 0).unwrap(),
        layout.InPort( m, 1).unwrap(),
        layout.OutPort( m, 0).unwrap(),
    );
}

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
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	( in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Nand);
        return Self { _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub const fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub const fn	Out( &self) -> PortId
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
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	( in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::And);
        return Self { _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub const fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub const fn	Out( &self) -> PortId
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
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	( in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Or);
        return Self { _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub const fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub const fn	Out( &self) -> PortId
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
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	( in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Xor);
        return Self { _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub const fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub const fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input NOR Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NorGate
{
    pub _In1: PortId,
    pub _In2: PortId,
    pub _Out: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl NorGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	( in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Nor);
        return Self { _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub const fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub const fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input XNOR Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct XnorGate
{
    pub _In1: PortId,
    pub _In2: PortId,
    pub _Out: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl XnorGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	( in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Xnor);
        return Self { _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	In1( &self) -> PortId
    {
        return self._In1;
    }

    #[inline]
    pub const fn	In2( &self) -> PortId
    {
        return self._In2;
    }

    #[inline]
    pub const fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Input Inverter / NOT Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NotGate
{
    pub _In: PortId,
    pub _Out: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl NotGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	m = layout.AddModule(
            name,
            parent,
            &[ PortDesc::Bool( "in") ],
            &[ PortDesc::Bool( "out") ],
            KernelKind::Not,
        );
        return Self {
            _In: layout.InPort( m, 0).unwrap(),
            _Out: layout.OutPort( m, 0).unwrap(),
        };
    }

    #[inline]
    pub const fn	In( &self) -> PortId
    {
        return self._In;
    }

    #[inline]
    pub const fn	Out( &self) -> PortId
    {
        return self._Out;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
