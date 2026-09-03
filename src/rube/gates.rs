//-- gates.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::rube::{
    layout::Layout,
    module::{ IModule, KernelKind, ModuleId },
    port::{ PortDesc, PortId },
};

//---------------------------------------------------------------------------------------------------------------------------------

#[inline]
fn	CreateGate2( layout: &mut Layout, name: &str, parent: Option< ModuleId>, kind: KernelKind) -> ( ModuleId, PortId, PortId, PortId)
{
    let  	m = layout.AddModule(
        name,
        parent,
        &[ PortDesc::Bool( "in1"), PortDesc::Bool( "in2") ],
        &[ PortDesc::Bool( "out") ],
        kind,
    );
    let  	in1 = layout.InPort( m, 0).unwrap();
    let  	in2 = layout.InPort( m, 1).unwrap();
    let  	out = layout.OutPort( m, 0).unwrap();
    layout.SealModule( m);
    return ( m, in1, in2, out);
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input NAND Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NandGate
{
    pub _Id:   ModuleId,
    pub _In1:  PortId,
    pub _In2:  PortId,
    pub _Out:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl NandGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	( m, in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Nand);
        return Self { _Id: m, _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for NandGate
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input AND Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct AndGate
{
    pub _Id:   ModuleId,
    pub _In1:  PortId,
    pub _In2:  PortId,
    pub _Out:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AndGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	( m, in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::And);
        return Self { _Id: m, _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for AndGate
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input OR Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct OrGate
{
    pub _Id:   ModuleId,
    pub _In1:  PortId,
    pub _In2:  PortId,
    pub _Out:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl OrGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	( m, in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Or);
        return Self { _Id: m, _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for OrGate
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input XOR Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct XorGate
{
    pub _Id:   ModuleId,
    pub _In1:  PortId,
    pub _In2:  PortId,
    pub _Out:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl XorGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	( m, in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Xor);
        return Self { _Id: m, _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for XorGate
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input NOR Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NorGate
{
    pub _Id:   ModuleId,
    pub _In1:  PortId,
    pub _In2:  PortId,
    pub _Out:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl NorGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	( m, in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Nor);
        return Self { _Id: m, _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for NorGate
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 2-Input XNOR Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct XnorGate
{
    pub _Id:   ModuleId,
    pub _In1:  PortId,
    pub _In2:  PortId,
    pub _Out:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl XnorGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	( m, in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Xnor);
        return Self { _Id: m, _In1: in1, _In2: in2, _Out: out };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for XnorGate
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Input Inverter / NOT Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct NotGate
{
    pub _Id:   ModuleId,
    pub _In:   PortId,
    pub _Out:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl NotGate
{
    #[inline]
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	m = layout.AddModule(
            name,
            parent,
            &[ PortDesc::Bool( "in") ],
            &[ PortDesc::Bool( "out") ],
            KernelKind::Not,
        );
        let  	inPort = layout.InPort( m, 0).unwrap();
        let  	outPort = layout.OutPort( m, 0).unwrap();
        layout.SealModule( m);
        return Self {
            _Id:  m,
            _In:  inPort,
            _Out: outPort,
        };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for NotGate
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
