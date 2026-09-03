//-- latches.rs ---------------------------------------------------------------------------------------------------------------------

use	crate::rube::{
    engine::SimEngine,
    gates::{ NandGate, NotGate },
    layout::Layout,
    port::PortId,
    reg::Reg,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// Asynchronous RS Latch ( Cross-coupled NANDs)
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RSLatch
{
    pub _Nand1: NandGate,
    pub _Nand2: NandGate,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl RSLatch
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	nand1 = NandGate::New( layout, &format!( "{name}.Nand1"), parent);
        let  	nand2 = NandGate::New( layout, &format!( "{name}.Nand2"), parent);

        layout.Connect( nand1.Out(), nand2.In2());
        layout.Connect( nand2.Out(), nand1.In2());

        return Self {
            _Nand1: nand1,
            _Nand2: nand2,
        };
    }

    #[inline]
    pub const fn	S( &self) -> PortId
    {
        return self._Nand1.In1();
    }

    #[inline]
    pub const fn	R( &self) -> PortId
    {
        return self._Nand2.In1();
    }

    #[inline]
    pub const fn	Q( &self) -> PortId
    {
        return self._Nand1.Out();
    }

    #[inline]
    pub const fn	Q1( &self) -> PortId
    {
        return self._Nand2.Out();
    }

    #[inline]
    pub fn	SetS( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self.S(), val);
    }

    #[inline]
    pub fn	SetR( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self.R(), val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Clocked RS Latch
#[derive( Clone, Debug)]
pub struct CRSLatch
{
    pub _GateS: NandGate,
    pub _GateR: NandGate,
    pub _RS: RSLatch,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CRSLatch
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	gateS = NandGate::New( layout, &format!( "{name}.GateS"), parent);
        let  	gateR = NandGate::New( layout, &format!( "{name}.GateR"), parent);
        let  	rs = RSLatch::New( layout, &format!( "{name}.RS"), parent);

        layout.Connect( gateS.Out(), rs.S());
        layout.Connect( gateR.Out(), rs.R());

        return Self {
            _GateS: gateS,
            _GateR: gateR,
            _RS: rs,
        };
    }

    #[inline]
    pub const fn	Clk1( &self) -> PortId
    {
        return self._GateS.In2();
    }

    #[inline]
    pub const fn	Clk2( &self) -> PortId
    {
        return self._GateR.In1();
    }

    #[inline]
    pub const fn	S( &self) -> PortId
    {
        return self._GateS.In1();
    }

    #[inline]
    pub const fn	R( &self) -> PortId
    {
        return self._GateR.In2();
    }

    #[inline]
    pub const fn	Q( &self) -> PortId
    {
        return self._RS.Q();
    }

    #[inline]
    pub const fn	Q1( &self) -> PortId
    {
        return self._RS.Q1();
    }

    #[inline]
    pub fn	SetS( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self.S(), val);
    }

    #[inline]
    pub fn	SetR( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self.R(), val);
    }

    #[inline]
    pub fn	SetClk( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self.Clk1(), val);
        engine.SetPortBool( self.Clk2(), val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Transparent D-Latch
#[derive( Clone, Debug)]
pub struct DLatch
{
    pub _CRS: CRSLatch,
    pub _Inv: NotGate,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl DLatch
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	crs = CRSLatch::New( layout, &format!( "{name}.CRS"), parent);
        let  	inv = NotGate::New( layout, &format!( "{name}.Inv"), parent);

        layout.Connect( inv.Out(), crs.R());

        return Self {
            _CRS: crs,
            _Inv: inv,
        };
    }

    #[inline]
    pub const fn	D( &self) -> PortId
    {
        return self._CRS.S();
    }

    #[inline]
    pub const fn	DInv( &self) -> PortId
    {
        return self._Inv.In();
    }

    #[inline]
    pub const fn	E1( &self) -> PortId
    {
        return self._CRS.Clk1();
    }

    #[inline]
    pub const fn	E2( &self) -> PortId
    {
        return self._CRS.Clk2();
    }

    #[inline]
    pub const fn	Q( &self) -> PortId
    {
        return self._CRS.Q();
    }

    #[inline]
    pub const fn	Q1( &self) -> PortId
    {
        return self._CRS.Q1();
    }

    #[inline]
    pub fn	SetD( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self.D(), val);
        engine.SetPortBool( self.DInv(), val);
    }

    #[inline]
    pub fn	SetE( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self.E1(), val);
        engine.SetPortBool( self.E2(), val);
    }
}


