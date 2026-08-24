/// Latches and Flip-Flops
///
/// Direct Rust equivalent of `Fr_RSLatch`, `Fr_CRSLatch`, `Fr_RSFlipFlop`, `Fr_DLatch`.

use	crate::rube::gates::{NandGate, NotGate};
use	crate::rube::reg::Reg;
use	crate::rube::trigger::TriggerId;
use	crate::rube::sim_context::SimContext;

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
    nand1: NandGate,
    nand2: NandGate,
}

impl RSLatch
{
    /// Constructs and wires the RS Latch within the given simulation context
    pub fn	new( ctxt: &mut SimContext, name: &str) -> Self
{
        let  	s = ctxt.add_trigger( &format!( "{name}._S"), Reg::TRUE);
        let  	r = ctxt.add_trigger( &format!( "{name}._R"), Reg::TRUE);
        let  	q = ctxt.add_trigger( &format!( "{name}._Q"), Reg::FALSE);
        let  	q1 = ctxt.add_trigger( &format!( "{name}._Q1"), Reg::TRUE);

        let  	nand1 = NandGate::new( ctxt, &format!( "{name}.nand1"), s, q1, q);
        let  	nand2 = NandGate::new( ctxt, &format!( "{name}.nand2"), r, q, q1);

        let  	latch = Self {
            _Name: name.to_string(),
            _S: s,
            _R: r,
            _Q: q,
            _Q1: q1,
            nand1,
            nand2,
        };

        latch.initialize( ctxt);
        latch
    }

    pub fn	s( &self) -> TriggerId { self._S }
    pub fn	r( &self) -> TriggerId { self._R }
    pub fn	q( &self) -> TriggerId { self._Q }
    pub fn	q1( &self) -> TriggerId { self._Q1 }

    /// Construct with user-provided external signals
    pub fn	with_signals( ctxt: &mut SimContext, name: &str, s: TriggerId, r: TriggerId, q: TriggerId, q1: TriggerId) -> Self
{
        let  	nand1 = NandGate::new( ctxt, &format!( "{name}.nand1"), s, q1, q);
        let  	nand2 = NandGate::new( ctxt, &format!( "{name}.nand2"), r, q, q1);

        Self {
            _Name: name.to_string(),
            _S: s,
            _R: r,
            _Q: q,
            _Q1: q1,
            nand1,
            nand2,
        }
    }

    /// Initialize signal values: Q=0, Q1=1, S=1, R=1
    pub fn	initialize( &self, ctxt: &mut SimContext)
{
        ctxt.init_value( self._Q, Reg::FALSE);
        ctxt.init_value( self._Q1, Reg::TRUE);
        ctxt.init_value( self._S, Reg::TRUE);
        ctxt.init_value( self._R, Reg::TRUE);
    }

    /// Read Q output
    #[inline]
    pub fn	read_q( &self, ctxt: &SimContext) -> Reg
{
        ctxt.get_value( self._Q)
    }

    /// Read Q1 output
    #[inline]
    pub fn	read_q1( &self, ctxt: &SimContext) -> Reg
{
        ctxt.get_value( self._Q1)
    }

    /// Apply Set and Reset inputs and step/drive simulation to steady state
    pub fn	apply( &self, ctxt: &mut SimContext, s_val: Reg, r_val: Reg) -> usize
{
        ctxt.set_value( self._S, s_val);
        ctxt.set_value( self._R, r_val);
        ctxt.drive()
    }
}

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
    nand1: NandGate,
    nand2: NandGate,
    rs_latch: RSLatch,
}

impl CRSLatch
{
    pub fn	new( ctxt: &mut SimContext, name: &str) -> Self
{
        let  	clk = ctxt.add_trigger( &format!( "{name}._Clk"), Reg::FALSE);
        let  	s = ctxt.add_trigger( &format!( "{name}._S"), Reg::FALSE);
        let  	r = ctxt.add_trigger( &format!( "{name}._R"), Reg::FALSE);
        let  	q = ctxt.add_trigger( &format!( "{name}._Q"), Reg::FALSE);
        let  	q1 = ctxt.add_trigger( &format!( "{name}._Q1"), Reg::TRUE);

        let  	s_int = ctxt.add_trigger( &format!( "{name}.s_int"), Reg::TRUE);
        let  	r_int = ctxt.add_trigger( &format!( "{name}.r_int"), Reg::TRUE);

        let  	nand1 = NandGate::new( ctxt, &format!( "{name}.nand1"), s, clk, s_int);
        let  	nand2 = NandGate::new( ctxt, &format!( "{name}.nand2"), r, clk, r_int);
        let  	rs_latch = RSLatch::with_signals( ctxt, &format!( "{name}.rsLatch"), s_int, r_int, q, q1);

        let  	latch = Self {
            _Name: name.to_string(),
            _Clk: clk,
            _S: s,
            _R: r,
            _Q: q,
            _Q1: q1,
            nand1,
            nand2,
            rs_latch,
        };

        latch.initialize( ctxt);
        latch
    }

    pub fn	clk( &self) -> TriggerId { self._Clk }
    pub fn	s( &self) -> TriggerId { self._S }
    pub fn	r( &self) -> TriggerId { self._R }
    pub fn	q( &self) -> TriggerId { self._Q }
    pub fn	q1( &self) -> TriggerId { self._Q1 }

    pub fn	initialize( &self, ctxt: &mut SimContext)
{
        ctxt.init_value( self._Q, Reg::FALSE);
        ctxt.init_value( self._Q1, Reg::TRUE);
        ctxt.init_value( self._S, Reg::FALSE);
        ctxt.init_value( self._R, Reg::FALSE);
        ctxt.init_value( self._Clk, Reg::FALSE);
        self.rs_latch.initialize( ctxt);
    }
}

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
    not_gate: NotGate,
    latch1: CRSLatch,
    latch2: CRSLatch,
}

impl RSFlipFlop
{
    pub fn	new( ctxt: &mut SimContext, name: &str) -> Self
{
        let  	clk = ctxt.add_trigger( &format!( "{name}._Clk"), Reg::FALSE);
        let  	clk_inv = ctxt.add_trigger( &format!( "{name}.clk_inv"), Reg::TRUE);
        let  	s = ctxt.add_trigger( &format!( "{name}._S"), Reg::FALSE);
        let  	r = ctxt.add_trigger( &format!( "{name}._R"), Reg::FALSE);
        let  	q = ctxt.add_trigger( &format!( "{name}._Q"), Reg::FALSE);
        let  	q1 = ctxt.add_trigger( &format!( "{name}._Q1"), Reg::TRUE);

        let  	not_gate = NotGate::new( ctxt, &format!( "{name}.not"), clk, clk_inv);
        let  	latch1 = CRSLatch::new( ctxt, &format!( "{name}.latch1"));
        let  	latch2 = CRSLatch::new( ctxt, &format!( "{name}.latch2"));

        Self {
            _Name: name.to_string(),
            _Clk: clk,
            _S: s,
            _R: r,
            _Q: q,
            _Q1: q1,
            not_gate,
            latch1,
            latch2,
        }
    }

    pub fn	clk( &self) -> TriggerId { self._Clk }
    pub fn	s( &self) -> TriggerId { self._S }
    pub fn	r( &self) -> TriggerId { self._R }
    pub fn	q( &self) -> TriggerId { self._Q }
    pub fn	q1( &self) -> TriggerId { self._Q1 }
}

/// Transparent D Latch ( `Fr_DLatch`)
#[derive( Clone, Debug)]
pub struct DLatch
{
    _Name: String,
    _D: TriggerId,
    _E: TriggerId,
    _Q: TriggerId,
    _Q1: TriggerId,
    nand1: NandGate,
    nand2: NandGate,
    rs_latch: RSLatch,
}

impl DLatch
{
    pub fn	new( ctxt: &mut SimContext, name: &str) -> Self
{
        let  	d = ctxt.add_trigger( &format!( "{name}._D"), Reg::FALSE);
        let  	e = ctxt.add_trigger( &format!( "{name}._E"), Reg::FALSE);
        let  	q = ctxt.add_trigger( &format!( "{name}._Q"), Reg::FALSE);
        let  	q1 = ctxt.add_trigger( &format!( "{name}._Q1"), Reg::TRUE);

        let  	n1_out = ctxt.add_trigger( &format!( "{name}.n1_out"), Reg::TRUE);
        let  	n2_out = ctxt.add_trigger( &format!( "{name}.n2_out"), Reg::TRUE);

        let  	nand1 = NandGate::new( ctxt, &format!( "{name}.nand1"), d, e, n1_out);
        let  	nand2 = NandGate::new( ctxt, &format!( "{name}.nand2"), e, n1_out, n2_out);
        let  	rs_latch = RSLatch::with_signals( ctxt, &format!( "{name}.rsLatch"), n1_out, n2_out, q, q1);

        Self {
            _Name: name.to_string(),
            _D: d,
            _E: e,
            _Q: q,
            _Q1: q1,
            nand1,
            nand2,
            rs_latch,
        }
    }

    pub fn	d( &self) -> TriggerId { self._D }
    pub fn	e( &self) -> TriggerId { self._E }
    pub fn	q( &self) -> TriggerId { self._Q }
    pub fn	q1( &self) -> TriggerId { self._Q1 }
}
