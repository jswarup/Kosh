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
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	nand1 = NandGate::New( layout, &format!( "{name}.Nand1"));
        let  	nand2 = NandGate::New( layout, &format!( "{name}.Nand2"));

        let _ = layout.Connect( nand1.Out(), nand2.In2());
        let _ = layout.Connect( nand2.Out(), nand1.In2());

        return Self {
            _Nand1: nand1,
            _Nand2: nand2,
        };
    }

    #[inline]
    pub fn	S( &self) -> PortId
    {
        return self._Nand1.In1();
    }

    #[inline]
    pub fn	R( &self) -> PortId
    {
        return self._Nand2.In1();
    }

    #[inline]
    pub fn	Q( &self) -> PortId
    {
        return self._Nand1.Out();
    }

    #[inline]
    pub fn	Q1( &self) -> PortId
    {
        return self._Nand2.Out();
    }

    pub fn	SetS( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        engine.SetPortBool( self.S(), val);
    }

    pub fn	SetR( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        engine.SetPortBool( self.R(), val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Clocked RS Latch
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CRSLatch
{
    pub _GateS: NandGate,
    pub _GateR: NandGate,
    pub _RS: RSLatch,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CRSLatch
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	gateS = NandGate::New( layout, &format!( "{name}.GateS"));
        let  	gateR = NandGate::New( layout, &format!( "{name}.GateR"));
        let  	rs = RSLatch::New( layout, &format!( "{name}.RS"));

        let _ = layout.Connect( gateS.Out(), rs.S());
        let _ = layout.Connect( gateR.Out(), rs.R());

        return Self {
            _GateS: gateS,
            _GateR: gateR,
            _RS: rs,
        };
    }

    #[inline]
    pub fn	Clk1( &self) -> PortId
    {
        return self._GateS.In2();
    }

    #[inline]
    pub fn	Clk2( &self) -> PortId
    {
        return self._GateR.In1();
    }

    #[inline]
    pub fn	S( &self) -> PortId
    {
        return self._GateS.In1();
    }

    #[inline]
    pub fn	R( &self) -> PortId
    {
        return self._GateR.In2();
    }

    #[inline]
    pub fn	Q( &self) -> PortId
    {
        return self._RS.Q();
    }

    #[inline]
    pub fn	Q1( &self) -> PortId
    {
        return self._RS.Q1();
    }

    pub fn	SetClk( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        engine.SetPortBool( self._GateS.In2(), val);
        engine.SetPortBool( self._GateR.In1(), val);
    }

    pub fn	SetS( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        engine.SetPortBool( self.S(), val);
    }

    pub fn	SetR( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        engine.SetPortBool( self.R(), val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Transparent D Latch
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DLatch
{
    pub _Not: NotGate,
    pub _CRS: CRSLatch,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl DLatch
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	notGate = NotGate::New( layout, &format!( "{name}.Not"));
        let  	crs = CRSLatch::New( layout, &format!( "{name}.CRS"));

        let _ = layout.Connect( notGate.Out(), crs.R());

        return Self {
            _Not: notGate,
            _CRS: crs,
        };
    }

    #[inline]
    pub fn	D( &self) -> PortId
    {
        return self._CRS.S();
    }

    #[inline]
    pub fn	Q( &self) -> PortId
    {
        return self._CRS.Q();
    }

    #[inline]
    pub fn	Q1( &self) -> PortId
    {
        return self._CRS.Q1();
    }

    pub fn	SetE( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        self._CRS.SetClk( engine, val);
    }

    pub fn	SetD( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        engine.SetPortBool( self._CRS.S(), val);
        engine.SetPortBool( self._Not.In(), val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Master-Slave RS Flip-Flop
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RSFlipFlop
{
    pub _Master: CRSLatch,
    pub _Slave: CRSLatch,
    pub _Not: NotGate,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl RSFlipFlop
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	master = CRSLatch::New( layout, &format!( "{name}.Master"));
        let  	slave = CRSLatch::New( layout, &format!( "{name}.Slave"));
        let  	notGate = NotGate::New( layout, &format!( "{name}.Not"));

        let _ = layout.Connect( master.Q(), slave.S());
        let _ = layout.Connect( master.Q1(), slave.R());
        let _ = layout.Connect( notGate.Out(), slave.Clk1());
        let _ = layout.Connect( notGate.Out(), slave.Clk2());

        return Self {
            _Master: master,
            _Slave: slave,
            _Not: notGate,
        };
    }

    #[inline]
    pub fn	S( &self) -> PortId
    {
        return self._Master.S();
    }

    #[inline]
    pub fn	R( &self) -> PortId
    {
        return self._Master.R();
    }

    #[inline]
    pub fn	Q( &self) -> PortId
    {
        return self._Slave.Q();
    }

    #[inline]
    pub fn	Q1( &self) -> PortId
    {
        return self._Slave.Q1();
    }

    pub fn	SetClk( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        self._Master.SetClk( engine, val);
        engine.SetPortBool( self._Not.In(), val);
    }

    pub fn	SetS( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        self._Master.SetS( engine, val);
    }

    pub fn	SetR( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        self._Master.SetR( engine, val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
