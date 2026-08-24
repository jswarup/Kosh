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
    pub and_gate: AndGate,
    pub xor_gate: XorGate,
}

impl HalfAdder
{
    pub fn	new( ctxt: &mut SimContext, name: &str, a: TriggerId, b: TriggerId, sum: TriggerId, carry: TriggerId) -> Self
{
        let  	and_gate = AndGate::new( ctxt, &format!( "{name}.and"), a, b, carry);
        let  	xor_gate = XorGate::new( ctxt, &format!( "{name}.xor"), a, b, sum);

        Self {
            _Name: name.to_string(),
            _A: a,
            _B: b,
            _Sum: sum,
            _Carry: carry,
            and_gate,
            xor_gate,
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
    h1: HalfAdder,
    h2: HalfAdder,
    or_gate: OrGate,
}

impl FullAdder
{
    pub fn	new( ctxt: &mut SimContext, name: &str, a: TriggerId, b: TriggerId, c_in: TriggerId, sum: TriggerId, carry: TriggerId) -> Self
{
        let  	h1_sum = ctxt.add_trigger( &format!( "{name}.h1_sum"), Reg::FALSE);
        let  	h1_carry = ctxt.add_trigger( &format!( "{name}.h1_carry"), Reg::FALSE);
        let  	h1 = HalfAdder::new( ctxt, &format!( "{name}.h1"), a, b, h1_sum, h1_carry);

        let  	h2_carry = ctxt.add_trigger( &format!( "{name}.h2_carry"), Reg::FALSE);
        let  	h2 = HalfAdder::new( ctxt, &format!( "{name}.h2"), h1_sum, c_in, sum, h2_carry);

        let  	or_gate = OrGate::new( ctxt, &format!( "{name}.or"), h1_carry, h2_carry, carry);

        Self {
            _Name: name.to_string(),
            _A: a,
            _B: b,
            _CIn: c_in,
            _Sum: sum,
            _Carry: carry,
            h1,
            h2,
            or_gate,
        }
    }
}

/// Ripple Carry Adder ( `Fr_Adder< N>`)
///
/// Composed of N full adders ( actually 1 HalfAdder and N-1 FullAdders to mimic Artesa exactly).
#[derive( Clone, Debug)]
pub struct RippleCarryAdder< const N: usize>
{
    _Name: String,
    _A: [TriggerId; N],
    _B: [TriggerId; N],
    _Sum: [TriggerId; N],
    _Carry: TriggerId,
    first_adder: HalfAdder,
    chain: Vec< FullAdder>,
}

impl< const N: usize> RippleCarryAdder< N>
{
    pub fn	new( ctxt: &mut SimContext, name: &str) -> Self
{
        assert!( N > 0, "RippleCarryAdder must have at least 1 bit");
        let  	mut a = [U32( 0); N];
        let  	mut b = [U32( 0); N];
        let  	mut sum = [U32( 0); N];
        for i in 0..N {
            a[i] = ctxt.add_trigger( &format!( "{name}._A[{i}]"), Reg::FALSE);
            b[i] = ctxt.add_trigger( &format!( "{name}._B[{i}]"), Reg::FALSE);
            sum[i] = ctxt.add_trigger( &format!( "{name}._Sum[{i}]"), Reg::FALSE);
        }
        let  	carry = ctxt.add_trigger( &format!( "{name}.carry_out"), Reg::FALSE);
        Self::with_triggers( ctxt, name, a, b, sum, carry)
    }

    pub fn	a( &self) -> &[TriggerId; N] { &self._A }
    pub fn	b( &self) -> &[TriggerId; N] { &self._B }
    pub fn	sum( &self) -> &[TriggerId; N] { &self._Sum }
    pub fn	carry( &self) -> TriggerId { self._Carry }

    pub fn	with_triggers( ctxt: &mut SimContext, name: &str, a: [TriggerId; N], b: [TriggerId; N], sum: [TriggerId; N], carry: TriggerId) -> Self
{
        assert!( N > 0, "RippleCarryAdder must have at least 1 bit");

        let  	mut chain = Vec::new();

        // 1st bit is a HalfAdder ( no carry in)
        let  	mut prev_carry = if N > 1 {
            ctxt.add_trigger( &format!( "{name}.carry_0"), Reg::FALSE)
        } else {
            carry
        };

        let  	first_adder = HalfAdder::new( ctxt, &format!( "{name}.bit0"), a[0], b[0], sum[0], prev_carry);

        // Remaining bits are FullAdders
        for i in 1..N {
            let  	next_carry = if i == N - 1 {
                carry
            } else {
                ctxt.add_trigger( &format!( "{name}.carry_{i}"), Reg::FALSE)
            };

            let  	fa = FullAdder::new( ctxt, &format!( "{name}.bit{i}"), a[i], b[i], prev_carry, sum[i], next_carry);
            chain.push( fa);
            prev_carry = next_carry;
        }

        Self {
            _Name: name.to_string(),
            _A: a,
            _B: b,
            _Sum: sum,
            _Carry: carry,
            first_adder,
            chain,
        }
    }

    pub fn	set_a( &self, ctxt: &mut SimContext, val: U32)
{
        for i in 0..N {
            let  	bit = if ( val & ( 1 << i)) != 0 { Reg::TRUE } else { Reg::FALSE };
            ctxt.set_value( self._A[i], bit);
        }
    }

    pub fn	set_b( &self, ctxt: &mut SimContext, val: U32)
{
        for i in 0..N {
            let  	bit = if ( val & ( 1 << i)) != 0 { Reg::TRUE } else { Reg::FALSE };
            ctxt.set_value( self._B[i], bit);
        }
    }

    pub fn	get_sum( &self, ctxt: &SimContext) -> U32
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
