//-- gates.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::{
    rube::{
        layout::Layout,
        module::{ IModule, KernelKind, KernelOp, ModuleId },
        port::{ PortDesc, PortId },
    },
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

macro_rules! DefineGate2
{
    ( $gate:ident, $op:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive( Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
        pub struct $gate
        {
            pub _Id:   ModuleId,
            pub _In1:  PortId,
            pub _In2:  PortId,
            pub _Out:  PortId,
        }

        impl $gate
        {
            #[inline]
            pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
            {
                let  	( m, in1, in2, out) = CreateGate2( layout, name, parent, KernelKind::Fast( KernelOp::$op));
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

        impl IModule for $gate
        {
            #[inline]
            fn	Id( &self) -> ModuleId
            {
                return self._Id;
            }
        }

        crate::ImplFluxSource!( $gate, _Id, _In1, _In2, _Out);
    };
}

DefineGate2!( NandGate, Nand, "2-Input NAND Gate");
DefineGate2!( AndGate, And, "2-Input AND Gate");
DefineGate2!( OrGate, Or, "2-Input OR Gate");
DefineGate2!( XorGate, Xor, "2-Input XOR Gate");
DefineGate2!( NorGate, Nor, "2-Input NOR Gate");
DefineGate2!( XnorGate, Xnor, "2-Input XNOR Gate");

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Input Inverter / NOT Gate
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
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
            KernelKind::Fast( KernelOp::Not),
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

crate::ImplFluxSource!( NotGate, _Id, _In, _Out);
