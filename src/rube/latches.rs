//-- latches.rs ---------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::{ FieldExp, IFluxExportSource },
    rube::{
    engine::SimEngine,
    gates::{ NandGate, NotGate },
    layout::Layout,
    module::{ IModule, KernelKind, ModuleId },
    port::{ PortDesc, PortId },
    reg::Reg,
} };

//---------------------------------------------------------------------------------------------------------------------------------

/// Asynchronous RS Latch ( Cross-coupled NANDs)
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RSLatch
{
    pub _Id:     ModuleId,
    pub _Nand1:  NandGate,
    pub _Nand2:  NandGate,
    pub _S:      PortId,
    pub _R:      PortId,
    pub _Q:      PortId,
    pub _Q1:     PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl RSLatch
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	modId = layout.AddModule(
            name,
            parent,
            &[ PortDesc::Bool( "S"), PortDesc::Bool( "R") ],
            &[ PortDesc::Bool( "Q"), PortDesc::Bool( "Q1") ],
            KernelKind::None,
        );

        let  	s = layout.InPort( modId, 0).unwrap();
        let  	r = layout.InPort( modId, 1).unwrap();
        let  	q = layout.OutPort( modId, 0).unwrap();
        let  	q1 = layout.OutPort( modId, 1).unwrap();

        let  	nand1 = NandGate::New( layout, &format!( "{name}.Nand1"), Some( modId));
        let  	nand2 = NandGate::New( layout, &format!( "{name}.Nand2"), Some( modId));

        layout.Connect( s, nand1.In1());
        layout.Connect( r, nand2.In1());

        layout.Connect( nand1.Out(), nand2.In2());
        layout.Connect( nand2.Out(), nand1.In2());

        layout.Connect( nand1.Out(), q);
        layout.Connect( nand2.Out(), q1);

        layout.SealModule( modId);

        return Self {
            _Id:     modId,
            _Nand1:  nand1,
            _Nand2:  nand2,
            _S:      s,
            _R:      r,
            _Q:      q,
            _Q1:     q1,
        };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }

    #[inline]
    pub const fn	S( &self) -> PortId
    {
        return self._S;
    }

    #[inline]
    pub const fn	R( &self) -> PortId
    {
        return self._R;
    }

    #[inline]
    pub const fn	Q( &self) -> PortId
    {
        return self._Q;
    }

    #[inline]
    pub const fn	Q1( &self) -> PortId
    {
        return self._Q1;
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

impl IModule for RSLatch
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Clocked RS Latch
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct CRSLatch
{
    pub _Id:     ModuleId,
    pub _GateS:  NandGate,
    pub _GateR:  NandGate,
    pub _RS:     RSLatch,
    pub _Clk1:   PortId,
    pub _Clk2:   PortId,
    pub _S:      PortId,
    pub _R:      PortId,
    pub _Q:      PortId,
    pub _Q1:     PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CRSLatch
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	modId = layout.AddModule(
            name,
            parent,
            &[ PortDesc::Bool( "Clk1"), PortDesc::Bool( "Clk2"), PortDesc::Bool( "S"), PortDesc::Bool( "R") ],
            &[ PortDesc::Bool( "Q"), PortDesc::Bool( "Q1") ],
            KernelKind::None,
        );

        let  	clk1 = layout.InPort( modId, 0).unwrap();
        let  	clk2 = layout.InPort( modId, 1).unwrap();
        let  	s = layout.InPort( modId, 2).unwrap();
        let  	r = layout.InPort( modId, 3).unwrap();
        let  	q = layout.OutPort( modId, 0).unwrap();
        let  	q1 = layout.OutPort( modId, 1).unwrap();

        let  	gateS = NandGate::New( layout, &format!( "{name}.GateS"), Some( modId));
        let  	gateR = NandGate::New( layout, &format!( "{name}.GateR"), Some( modId));
        let  	rs = RSLatch::New( layout, &format!( "{name}.RS"), Some( modId));

        layout.Connect( s, gateS.In1());
        layout.Connect( clk1, gateS.In2());
        layout.Connect( clk2, gateR.In1());
        layout.Connect( r, gateR.In2());

        layout.Connect( gateS.Out(), rs.S());
        layout.Connect( gateR.Out(), rs.R());

        layout.Connect( rs.Q(), q);
        layout.Connect( rs.Q1(), q1);

        layout.SealModule( modId);

        return Self {
            _Id:     modId,
            _GateS:  gateS,
            _GateR:  gateR,
            _RS:     rs,
            _Clk1:   clk1,
            _Clk2:   clk2,
            _S:      s,
            _R:      r,
            _Q:      q,
            _Q1:     q1,
        };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }

    #[inline]
    pub const fn	Clk1( &self) -> PortId
    {
        return self._Clk1;
    }

    #[inline]
    pub const fn	Clk2( &self) -> PortId
    {
        return self._Clk2;
    }

    #[inline]
    pub const fn	S( &self) -> PortId
    {
        return self._S;
    }

    #[inline]
    pub const fn	R( &self) -> PortId
    {
        return self._R;
    }

    #[inline]
    pub const fn	Q( &self) -> PortId
    {
        return self._Q;
    }

    #[inline]
    pub const fn	Q1( &self) -> PortId
    {
        return self._Q1;
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

impl IModule for CRSLatch
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Transparent D-Latch
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct DLatch
{
    pub _Id:    ModuleId,
    pub _Not:   NotGate,
    pub _CRS:   CRSLatch,
    pub _D:     PortId,
    pub _DInv:  PortId,
    pub _E1:    PortId,
    pub _E2:    PortId,
    pub _Q:     PortId,
    pub _Q1:    PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl DLatch
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	modId = layout.AddModule(
            name,
            parent,
            &[ PortDesc::Bool( "D"), PortDesc::Bool( "DInv"), PortDesc::Bool( "E1"), PortDesc::Bool( "E2") ],
            &[ PortDesc::Bool( "Q"), PortDesc::Bool( "Q1") ],
            KernelKind::None,
        );

        let  	d = layout.InPort( modId, 0).unwrap();
        let  	dInv = layout.InPort( modId, 1).unwrap();
        let  	e1 = layout.InPort( modId, 2).unwrap();
        let  	e2 = layout.InPort( modId, 3).unwrap();
        let  	q = layout.OutPort( modId, 0).unwrap();
        let  	q1 = layout.OutPort( modId, 1).unwrap();

        let  	crs = CRSLatch::New( layout, &format!( "{name}.CRS"), Some( modId));
        let  	inv = NotGate::New( layout, &format!( "{name}.Inv"), Some( modId));

        layout.Connect( d, crs.S());
        layout.Connect( dInv, inv.In());
        layout.Connect( e1, crs.Clk1());
        layout.Connect( e2, crs.Clk2());

        layout.Connect( inv.Out(), crs.R());

        layout.Connect( crs.Q(), q);
        layout.Connect( crs.Q1(), q1);

        layout.SealModule( modId);

        return Self {
            _Id:    modId,
            _Not:   inv,
            _CRS:   crs,
            _D:     d,
            _DInv:  dInv,
            _E1:    e1,
            _E2:    e2,
            _Q:     q,
            _Q1:    q1,
        };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }

    #[inline]
    pub const fn	D( &self) -> PortId
    {
        return self._D;
    }

    #[inline]
    pub const fn	DInv( &self) -> PortId
    {
        return self._DInv;
    }

    #[inline]
    pub const fn	E1( &self) -> PortId
    {
        return self._E1;
    }

    #[inline]
    pub const fn	E2( &self) -> PortId
    {
        return self._E2;
    }

    #[inline]
    pub const fn	Q( &self) -> PortId
    {
        return self._Q;
    }

    #[inline]
    pub const fn	Q1( &self) -> PortId
    {
        return self._Q1;
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

//---------------------------------------------------------------------------------------------------------------------------------

impl IModule for DLatch
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------


//---------------------------------------------------------------------------------------------------------------------------------


impl IFluxExportSource for DLatch
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        let  	mut step = 0;
        let  	generator = move |key: &mut String, val: &mut FieldExp< 'a>| -> bool {
            match step {
                0 => {
                    *key = "Type".to_string();
                    *val = FieldExp::Str( "DLatch");
                    step += 1;
                    true
                }
                1 => {
                    *key = "_Id".to_string();
                    *val = FieldExp::FluxSource( &self._Id);
                    step += 1;
                    true
                }
                2 => {
                    *key = "_D".to_string();
                    *val = FieldExp::FluxSource( &self._D);
                    step += 1;
                    true
                }
                3 => {
                    *key = "_E1".to_string();
                    *val = FieldExp::FluxSource( &self._E1);
                    step += 1;
                    true
                }
                4 => {
                    *key = "_Q".to_string();
                    *val = FieldExp::FluxSource( &self._Q);
                    step += 1;
                    true
                }
                5 => {
                    *key = "_Q1".to_string();
                    *val = FieldExp::FluxSource( &self._Q1);
                    step += 1;
                    true
                }
                _ => false,
            }
        };
        *field = FieldExp::Obj( Box::new( generator));
    }
}

impl IFluxExportSource for CRSLatch
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        let  	mut step = 0;
        let  	generator = move |key: &mut String, val: &mut FieldExp< 'a>| -> bool {
            match step {
                0 => {
                    *key = "Type".to_string();
                    *val = FieldExp::Str( "CRSLatch");
                    step += 1;
                    true
                }
                1 => {
                    *key = "_Id".to_string();
                    *val = FieldExp::FluxSource( &self._Id);
                    step += 1;
                    true
                }
                2 => {
                    *key = "_S".to_string();
                    *val = FieldExp::FluxSource( &self._S);
                    step += 1;
                    true
                }
                3 => {
                    *key = "_R".to_string();
                    *val = FieldExp::FluxSource( &self._R);
                    step += 1;
                    true
                }
                4 => {
                    *key = "_Clk1".to_string();
                    *val = FieldExp::FluxSource( &self._Clk1);
                    step += 1;
                    true
                }
                5 => {
                    *key = "_Q".to_string();
                    *val = FieldExp::FluxSource( &self._Q);
                    step += 1;
                    true
                }
                6 => {
                    *key = "_Q1".to_string();
                    *val = FieldExp::FluxSource( &self._Q1);
                    step += 1;
                    true
                }
                _ => false,
            }
        };
        *field = FieldExp::Obj( Box::new( generator));
    }
}

impl IFluxExportSource for RSLatch
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        let  	mut step = 0;
        let  	generator = move |key: &mut String, val: &mut FieldExp< 'a>| -> bool {
            match step {
                0 => {
                    *key = "Type".to_string();
                    *val = FieldExp::Str( "RSLatch");
                    step += 1;
                    true
                }
                1 => {
                    *key = "_Id".to_string();
                    *val = FieldExp::FluxSource( &self._Id);
                    step += 1;
                    true
                }
                2 => {
                    *key = "_S".to_string();
                    *val = FieldExp::FluxSource( &self._S);
                    step += 1;
                    true
                }
                3 => {
                    *key = "_R".to_string();
                    *val = FieldExp::FluxSource( &self._R);
                    step += 1;
                    true
                }
                4 => {
                    *key = "_Q".to_string();
                    *val = FieldExp::FluxSource( &self._Q);
                    step += 1;
                    true
                }
                5 => {
                    *key = "_Q1".to_string();
                    *val = FieldExp::FluxSource( &self._Q1);
                    step += 1;
                    true
                }
                _ => false,
            }
        };
        *field = FieldExp::Obj( Box::new( generator));
    }
}
