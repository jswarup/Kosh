use	crate::silo::uint::*;
/// Arithmetic components ( Adders)
///
/// Direct Rust equivalent of `Fr_HalfAdder`, `Fr_FullAdder`, `Fr_Adder`

use	crate::rube::gates::{AndGate, OrGate, XorGate};
use	crate::rube::reg::Reg;
use	crate::rube::trigger::TriggerId;
use	crate::rube::sim_context::SimContext;

/// Half Adder ( `Fr_HalfAdder`)
///
/// Ports:
/// - `a`: Input A
/// - `b`: Input B
/// - `sum`: Output Sum ( A ^ B)
/// - `carry`: Output Carry ( A & B)
#[derive( Clone, Debug)]
pub struct HalfAdder
{
    pub _Name: String,
    pub _A: TriggerId,
    pub _B: TriggerId,
    pub _Sum: TriggerId,
    pub _Carry: TriggerId,
    pub _AndGate: AndGate,
    pub _XorGate: XorGate,
}

impl HalfAdder
{
    pub fn	New( ctxt: &mut SimContext, name: &str, a: TriggerId, b: TriggerId, sum: TriggerId, carry: TriggerId) -> Self
{
        let  	andGate = AndGate::new( ctxt, &format!( "{name}.and"), a, b, carry);
        let  	xorGate = XorGate::new( ctxt, &format!( "{name}.xor"), a, b, sum);

        Self {
            _Name: name.to_string(),
            _A: a,
            _B: b,
            _Sum: sum,
            _Carry: carry,
            _AndGate: andGate,
            _XorGate: xorGate,
        }
    }
}

/// Full Adder ( `Fr_FullAdder`)
///
/// Ports:
/// - `a`: Input A
/// - `b`: Input B
/// - `c_in`: Input Carry
/// - `sum`: Output Sum
/// - `carry`: Output Carry
#[derive( Clone, Debug)]
pub struct FullAdder
{
    _Name: String,
    _A: TriggerId,
    _B: TriggerId,
    _CIn: TriggerId,
    _Sum: TriggerId,
    _Carry: TriggerId,
    _H1: HalfAdder,
    _H2: HalfAdder,
    _OrGate: OrGate,
}

impl FullAdder
{
    pub fn	New( ctxt: &mut SimContext, name: &str, a: TriggerId, b: TriggerId, cIn: TriggerId, sum: TriggerId, carry: TriggerId) -> Self
{
        let  	h1Sum = ctxt.add_trigger( &format!( "{name}.h1Sum"), Reg::FALSE);
        let  	h1Carry = ctxt.add_trigger( &format!( "{name}.h1Carry"), Reg::FALSE);
        let  	h1 = HalfAdder::New( ctxt, &format!( "{name}.h1"), a, b, h1Sum, h1Carry);

        let  	h2Carry = ctxt.add_trigger( &format!( "{name}.h2Carry"), Reg::FALSE);
        let  	h2 = HalfAdder::New( ctxt, &format!( "{name}.h2"), h1Sum, c_in, sum, h2Carry);

        let  	orGate = OrGate::new( ctxt, &format!( "{name}.or"), h1Carry, h2Carry, carry);

        Self {
            _Name: name.to_string(),
            _A: a,
            _B: b,
            _CIn: cIn,
            _Sum: sum,
            _Carry: carry,
            _H1: h1,
            _H2: h2,
            _OrGate: orGate,
        }
    }
}

/// Ripple Carry Adder ( `Fr_Adder< N>`)
///
/// Composed of N full adders ( actually 1 HalfAdder and N-1 FullAdders to mimic Artesa exactly).
#[derive( Clone, Debug)]
pub struct Adder< const N: usize>
{
    _Name: String,
    _A: [TriggerId; N],
    _B: [TriggerId; N],
    _Sum: [TriggerId; N],
    _Carry: TriggerId,
    _FirstAdder: HalfAdder,
    _Chain: Vec< FullAdder>,
}

impl< const N: usize> Adder< N>
{
    pub fn	New( ctxt: &mut SimContext, name: &str) -> Self
{
        assert!( N > 0, "Adder must have at least 1 bit");
        let  	mut a = [U32( 0); N];
        let  	mut b = [U32( 0); N];
        let  	mut sum = [U32( 0); N];
        for i in 0..N {
            a[i] = ctxt.add_trigger( &format!( "{name}._A[{i}]"), Reg::FALSE);
            b[i] = ctxt.add_trigger( &format!( "{name}._B[{i}]"), Reg::FALSE);
            sum[i] = ctxt.add_trigger( &format!( "{name}._Sum[{i}]"), Reg::FALSE);
        }
        let  	carry = ctxt.add_trigger( &format!( "{name}.carry_out"), Reg::FALSE);
        Self::WithTriggers( ctxt, name, a, b, sum, carry)
    }

    pub fn	A( &self) -> &[TriggerId; N] { &self._A }
    pub fn	B( &self) -> &[TriggerId; N] { &self._B }
    pub fn	Sum( &self) -> &[TriggerId; N] { &self._Sum }
    pub fn	Carry( &self) -> TriggerId { self._Carry }

    pub fn	WithTriggers( ctxt: &mut SimContext, name: &str, a: [TriggerId; N], b: [TriggerId; N], sum: [TriggerId; N], carry: TriggerId) -> Self
{
        assert!( N > 0, "Adder must have at least 1 bit");

        let  	mut chain = Vec::new();

        // 1st bit is a HalfAdder ( no carry in)
        let  	mut prevCarry = if N > 1 {
            ctxt.add_trigger( &format!( "{name}.carry_0"), Reg::FALSE)
        } else {
            carry
        };

        let  	firstAdder = HalfAdder::New( ctxt, &format!( "{name}.bit0"), a[0], b[0], sum[0], prevCarry);

        // Remaining bits are FullAdders
        for i in 1..N {
            let  	nextCarry = if i == N - 1 {
                carry
            } else {
                ctxt.add_trigger( &format!( "{name}.carry_{i}"), Reg::FALSE)
            };

            let  	fa = FullAdder::New( ctxt, &format!( "{name}.bit{i}"), a[i], b[i], prevCarry, sum[i], nextCarry);
            chain.push( fa);
            prevCarry = nextCarry;
        }

        Self {
            _Name: name.to_string(),
            _A: a,
            _B: b,
            _Sum: sum,
            _Carry: carry,
            _FirstAdder: firstAdder,
            _Chain: chain,
        }
    }

    pub fn	SetA( &self, ctxt: &mut SimContext, val: U32)
{
        for i in 0..N {
            let  	bit = if ( val & ( 1 << i)) != 0 { Reg::TRUE } else { Reg::FALSE };
            ctxt.set_value( self._A[i], bit);
        }
    }

    pub fn	SetB( &self, ctxt: &mut SimContext, val: U32)
{
        for i in 0..N {
            let  	bit = if ( val & ( 1 << i)) != 0 { Reg::TRUE } else { Reg::FALSE };
            ctxt.set_value( self._B[i], bit);
        }
    }

    pub fn	GetSum( &self, ctxt: &SimContext) -> U32
{
        let  	mut val = 0;
        for i in 0..N {
            let  	bit = ctxt.get_value( self._Sum[i]);
            assert!( bit.is_true() || bit.is_false(), "Output bit is not a valid boolean state: {:?}", bit);
            if bit.is_true() {
                val |= 1 << i;
            }
        }
        crate::silo::uint::U32(val)
    }
}
