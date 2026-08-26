//-- latches.rs ---------------------------------------------------------------------------------------------------------------------
/// Latches and Flip-Flops
///
/// Direct Rust equivalent of `Fr_RSLatch`, `Fr_CRSLatch`, `Fr_RSFlipFlop`, `Fr_DLatch`.

use	crate::rube::{
    gates::{ NandGate, NotGate },
    reg::Reg,
    sim_context::SimContext,
    trigger::TriggerId,
};

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IRSLatch
{
    fn	S( &self) -> TriggerId;
    fn	R( &self) -> TriggerId;
    fn	Q( &self) -> TriggerId;
    fn	Q1( &self) -> TriggerId;
    fn	Initialize( &self, ctxt: &mut SimContext);
    fn	ReadQ( &self, ctxt: &SimContext) -> Reg;
    fn	ReadQ1( &self, ctxt: &SimContext) -> Reg;
    fn	Apply( &self, ctxt: &mut SimContext, sVal: Reg, rVal: Reg) -> usize;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// RS Latch with cross-coupled NAND gates ( `Fr_RSLatch`)
///
/// Ports:
/// - `s`: Active-low Set input
/// - `r`: Active-low Reset input
/// - `q`: Output Q
/// - `q1`: Inverted Output Q1 ( not-Q)
///
/// Truth table ( active-low NAND RS Latch):
/// - S=1, R=1: Latch / Hold previous state
/// - S=0, R=1: Set ( Q=1, Q1=0)
/// - S=1, R=0: Reset ( Q=0, Q1=1)
/// - S=0, R=0: Metastable / Both high ( Q=1, Q1=1)
#[derive( Clone, Debug)]
pub struct RSLatch
{
    _Name: String,
    _S: TriggerId,
    _R: TriggerId,
    _Q: TriggerId,
    _Q1: TriggerId,
    _Nand1: NandGate,
    _Nand2: NandGate,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl RSLatch
{
    /// Constructs and wires the RS Latch within the given simulation context
    pub fn	New( ctxt: &mut SimContext, name: &str) -> Self
    {
        let  	s = ctxt.AddTrigger( &format!( "{name}._S"), Reg::TRUE);
        let  	r = ctxt.AddTrigger( &format!( "{name}._R"), Reg::TRUE);
        let  	q = ctxt.AddTrigger( &format!( "{name}._Q"), Reg::FALSE);
        let  	q1 = ctxt.AddTrigger( &format!( "{name}._Q1"), Reg::TRUE);

        let  	nand1 = NandGate::New( ctxt, &format!( "{name}.nand1"), s, q1, q);
        let  	nand2 = NandGate::New( ctxt, &format!( "{name}.nand2"), r, q, q1);

        let  	latch = Self {
            _Name: name.to_string(),
            _S: s,
            _R: r,
            _Q: q,
            _Q1: q1,
            _Nand1: nand1,
            _Nand2: nand2,
        };

        latch.Initialize( ctxt);
        return latch;
    }

    /// Construct with user-provided external signals
    pub fn	WithSignals( ctxt: &mut SimContext, name: &str, s: TriggerId, r: TriggerId, q: TriggerId, q1: TriggerId) -> Self
    {
        let  	nand1 = NandGate::New( ctxt, &format!( "{name}.nand1"), s, q1, q);
        let  	nand2 = NandGate::New( ctxt, &format!( "{name}.nand2"), r, q, q1);

        return Self {
            _Name: name.to_string(),
            _S: s,
            _R: r,
            _Q: q,
            _Q1: q1,
            _Nand1: nand1,
            _Nand2: nand2,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IRSLatch for RSLatch
{
    #[inline]
    fn	S( &self) -> TriggerId
    {
        return self._S;
    }

    #[inline]
    fn	R( &self) -> TriggerId
    {
        return self._R;
    }

    #[inline]
    fn	Q( &self) -> TriggerId
    {
        return self._Q;
    }

    #[inline]
    fn	Q1( &self) -> TriggerId
    {
        return self._Q1;
    }

    /// Initialize signal values: Q=0, Q1=1, S=1, R=1
    fn	Initialize( &self, ctxt: &mut SimContext)
    {
        ctxt.InitValue( self._Q, Reg::FALSE);
        ctxt.InitValue( self._Q1, Reg::TRUE);
        ctxt.InitValue( self._S, Reg::TRUE);
        ctxt.InitValue( self._R, Reg::TRUE);
    }

    /// Read Q output
    #[inline]
    fn	ReadQ( &self, ctxt: &SimContext) -> Reg
    {
        return ctxt.GetValue( self._Q);
    }

    /// Read Q1 output
    #[inline]
    fn	ReadQ1( &self, ctxt: &SimContext) -> Reg
    {
        return ctxt.GetValue( self._Q1);
    }

    /// Apply Set and Reset inputs and step/drive simulation to steady state
    fn	Apply( &self, ctxt: &mut SimContext, sVal: Reg, rVal: Reg) -> usize
    {
        ctxt.SetValue( self._S, sVal);
        ctxt.SetValue( self._R, rVal);
        return ctxt.Drive();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait ICRSLatch
{
    fn	Clk( &self) -> TriggerId;
    fn	S( &self) -> TriggerId;
    fn	R( &self) -> TriggerId;
    fn	Q( &self) -> TriggerId;
    fn	Q1( &self) -> TriggerId;
    fn	Initialize( &self, ctxt: &mut SimContext);
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Clocked RS Latch ( `Fr_CRSLatch`)
#[derive( Clone, Debug)]
pub struct CRSLatch
{
    _Name: String,
    _Clk: TriggerId,
    _S: TriggerId,
    _R: TriggerId,
    _Q: TriggerId,
    _Q1: TriggerId,
    _Nand1: NandGate,
    _Nand2: NandGate,
    _RSLatch: RSLatch,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl CRSLatch
{
    pub fn	New( ctxt: &mut SimContext, name: &str) -> Self
    {
        let  	clk = ctxt.AddTrigger( &format!( "{name}._Clk"), Reg::FALSE);
        let  	s = ctxt.AddTrigger( &format!( "{name}._S"), Reg::FALSE);
        let  	r = ctxt.AddTrigger( &format!( "{name}._R"), Reg::FALSE);
        let  	q = ctxt.AddTrigger( &format!( "{name}._Q"), Reg::FALSE);
        let  	q1 = ctxt.AddTrigger( &format!( "{name}._Q1"), Reg::TRUE);

        let  	sInt = ctxt.AddTrigger( &format!( "{name}.sInt"), Reg::TRUE);
        let  	rInt = ctxt.AddTrigger( &format!( "{name}.rInt"), Reg::TRUE);

        let  	nand1 = NandGate::New( ctxt, &format!( "{name}.nand1"), s, clk, sInt);
        let  	nand2 = NandGate::New( ctxt, &format!( "{name}.nand2"), r, clk, rInt);
        let  	rsLatch = RSLatch::WithSignals( ctxt, &format!( "{name}.rsLatch"), sInt, rInt, q, q1);

        let  	latch = Self {
            _Name: name.to_string(),
            _Clk: clk,
            _S: s,
            _R: r,
            _Q: q,
            _Q1: q1,
            _Nand1: nand1,
            _Nand2: nand2,
            _RSLatch: rsLatch,
        };

        latch.Initialize( ctxt);
        return latch;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl ICRSLatch for CRSLatch
{
    #[inline]
    fn	Clk( &self) -> TriggerId
    {
        return self._Clk;
    }

    #[inline]
    fn	S( &self) -> TriggerId
    {
        return self._S;
    }

    #[inline]
    fn	R( &self) -> TriggerId
    {
        return self._R;
    }

    #[inline]
    fn	Q( &self) -> TriggerId
    {
        return self._Q;
    }

    #[inline]
    fn	Q1( &self) -> TriggerId
    {
        return self._Q1;
    }

    fn	Initialize( &self, ctxt: &mut SimContext)
    {
        ctxt.InitValue( self._Q, Reg::FALSE);
        ctxt.InitValue( self._Q1, Reg::TRUE);
        ctxt.InitValue( self._S, Reg::FALSE);
        ctxt.InitValue( self._R, Reg::FALSE);
        ctxt.InitValue( self._Clk, Reg::FALSE);
        self._RSLatch.Initialize( ctxt);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IRSFlipFlop
{
    fn	Clk( &self) -> TriggerId;
    fn	S( &self) -> TriggerId;
    fn	R( &self) -> TriggerId;
    fn	Q( &self) -> TriggerId;
    fn	Q1( &self) -> TriggerId;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Master-Slave RS Flip-Flop ( `Fr_RSFlipFlop`)
#[derive( Clone, Debug)]
pub struct RSFlipFlop
{
    _Name: String,
    _Clk: TriggerId,
    _S: TriggerId,
    _R: TriggerId,
    _Q: TriggerId,
    _Q1: TriggerId,
    _NotGate: NotGate,
    _Latch1: CRSLatch,
    _Latch2: CRSLatch,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl RSFlipFlop
{
    pub fn	New( ctxt: &mut SimContext, name: &str) -> Self
    {
        let  	clk = ctxt.AddTrigger( &format!( "{name}._Clk"), Reg::FALSE);
        let  	clkInv = ctxt.AddTrigger( &format!( "{name}.clkInv"), Reg::TRUE);
        let  	s = ctxt.AddTrigger( &format!( "{name}._S"), Reg::FALSE);
        let  	r = ctxt.AddTrigger( &format!( "{name}._R"), Reg::FALSE);
        let  	q = ctxt.AddTrigger( &format!( "{name}._Q"), Reg::FALSE);
        let  	q1 = ctxt.AddTrigger( &format!( "{name}._Q1"), Reg::TRUE);

        let  	notGate = NotGate::New( ctxt, &format!( "{name}.not"), clk, clkInv);
        let  	latch1 = CRSLatch::New( ctxt, &format!( "{name}.latch1"));
        let  	latch2 = CRSLatch::New( ctxt, &format!( "{name}.latch2"));

        return Self {
            _Name: name.to_string(),
            _Clk: clk,
            _S: s,
            _R: r,
            _Q: q,
            _Q1: q1,
            _NotGate: notGate,
            _Latch1: latch1,
            _Latch2: latch2,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IRSFlipFlop for RSFlipFlop
{
    #[inline]
    fn	Clk( &self) -> TriggerId
    {
        return self._Clk;
    }

    #[inline]
    fn	S( &self) -> TriggerId
    {
        return self._S;
    }

    #[inline]
    fn	R( &self) -> TriggerId
    {
        return self._R;
    }

    #[inline]
    fn	Q( &self) -> TriggerId
    {
        return self._Q;
    }

    #[inline]
    fn	Q1( &self) -> TriggerId
    {
        return self._Q1;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub trait IDLatch
{
    fn	D( &self) -> TriggerId;
    fn	E( &self) -> TriggerId;
    fn	Q( &self) -> TriggerId;
    fn	Q1( &self) -> TriggerId;
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Transparent D Latch ( `Fr_DLatch`)
#[derive( Clone, Debug)]
pub struct DLatch
{
    _Name: String,
    _D: TriggerId,
    _E: TriggerId,
    _Q: TriggerId,
    _Q1: TriggerId,
    _Nand1: NandGate,
    _Nand2: NandGate,
    _RSLatch: RSLatch,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl DLatch
{
    pub fn	New( ctxt: &mut SimContext, name: &str) -> Self
    {
        let  	d = ctxt.AddTrigger( &format!( "{name}._D"), Reg::FALSE);
        let  	e = ctxt.AddTrigger( &format!( "{name}._E"), Reg::FALSE);
        let  	q = ctxt.AddTrigger( &format!( "{name}._Q"), Reg::FALSE);
        let  	q1 = ctxt.AddTrigger( &format!( "{name}._Q1"), Reg::TRUE);

        let  	n1Out = ctxt.AddTrigger( &format!( "{name}.n1Out"), Reg::TRUE);
        let  	n2Out = ctxt.AddTrigger( &format!( "{name}.n2Out"), Reg::TRUE);

        let  	nand1 = NandGate::New( ctxt, &format!( "{name}.nand1"), d, e, n1Out);
        let  	nand2 = NandGate::New( ctxt, &format!( "{name}.nand2"), e, n1Out, n2Out);
        let  	rsLatch = RSLatch::WithSignals( ctxt, &format!( "{name}.rsLatch"), n1Out, n2Out, q, q1);

        return Self {
            _Name: name.to_string(),
            _D: d,
            _E: e,
            _Q: q,
            _Q1: q1,
            _Nand1: nand1,
            _Nand2: nand2,
            _RSLatch: rsLatch,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IDLatch for DLatch
{
    #[inline]
    fn	D( &self) -> TriggerId
    {
        return self._D;
    }

    #[inline]
    fn	E( &self) -> TriggerId
    {
        return self._E;
    }

    #[inline]
    fn	Q( &self) -> TriggerId
    {
        return self._Q;
    }

    #[inline]
    fn	Q1( &self) -> TriggerId
    {
        return self._Q1;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
