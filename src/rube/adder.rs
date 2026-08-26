//-- adder.rs -----------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::{
    rube::{
        engine::SimEngine,
        gates::{ AndGate, OrGate, XorGate },
        layout::Layout,
        module::KernelKind,
        port::{ PortDesc, PortId },
        reg::Reg,
        regval::RegVal,
    },
    silo::U32,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Bit Half Adder ( XOR + AND)
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HalfAdder
{
    pub _Xor: XorGate,
    pub _And: AndGate,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl HalfAdder
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	xorGate = XorGate::New( layout, &format!( "{name}.Xor"));
        let  	andGate = AndGate::New( layout, &format!( "{name}.And"));

        return Self {
            _Xor: xorGate,
            _And: andGate,
        };
    }

    #[inline]
    pub fn	Sum( &self) -> PortId
    {
        return self._Xor.Out();
    }

    #[inline]
    pub fn	Carry( &self) -> PortId
    {
        return self._And.Out();
    }

    pub fn	SetA( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        engine.SetPortBool( self._Xor.In1(), val);
        engine.SetPortBool( self._And.In1(), val);
    }

    pub fn	SetB( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        engine.SetPortBool( self._Xor.In2(), val);
        engine.SetPortBool( self._And.In2(), val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Bit Full Adder
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FullAdder
{
    pub _HA1: HalfAdder,
    pub _HA2: HalfAdder,
    pub _Or: OrGate,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl FullAdder
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	ha1 = HalfAdder::New( layout, &format!( "{name}.HA1"));
        let  	ha2 = HalfAdder::New( layout, &format!( "{name}.HA2"));
        let  	orGate = OrGate::New( layout, &format!( "{name}.Or"));

        let _ = layout.Connect( ha1.Sum(), ha2._Xor.In1());
        let _ = layout.Connect( ha1.Sum(), ha2._And.In1());
        let _ = layout.Connect( ha1.Carry(), orGate.In1());
        let _ = layout.Connect( ha2.Carry(), orGate.In2());

        return Self {
            _HA1: ha1,
            _HA2: ha2,
            _Or: orGate,
        };
    }

    #[inline]
    pub fn	Sum( &self) -> PortId
    {
        return self._HA2.Sum();
    }

    #[inline]
    pub fn	Carry( &self) -> PortId
    {
        return self._Or.Out();
    }

    pub fn	SetA( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        self._HA1.SetA( engine, val);
    }

    pub fn	SetB( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        self._HA1.SetB( engine, val);
    }

    pub fn	SetCIn( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        self._HA2.SetB( engine, val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// N-Bit Ripple Carry Adder
#[derive( Clone, Debug)]
pub struct Adder< const N: usize>
{
    pub _Bits: Vec< FullAdder>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< const N: usize> Adder< N>
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	mut bits: Vec< FullAdder> = Vec::with_capacity( N);
        for i in 0..N {
            let  	fa = FullAdder::New( layout, &format!( "{name}.Bit{i}"));
            if i > 0 {
                let _ = layout.Connect( bits[i - 1].Carry(), fa._HA2._Xor.In2());
                let _ = layout.Connect( bits[i - 1].Carry(), fa._HA2._And.In2());
            }
            bits.push( fa);
        }
        return Self { _Bits: bits };
    }

    #[inline]
    pub fn	Sum( &self, bit: usize) -> PortId
    {
        return self._Bits[bit].Sum();
    }

    #[inline]
    pub fn	Carry( &self) -> PortId
    {
        return self._Bits[N - 1].Carry();
    }

    pub fn	SetA( &self, engine: &mut SimEngine, val: U32)
    {
        let  	v = u32::from( val);
        for i in 0..N {
            let  	bit = ( ( v >> i) & 1) != 0;
            self._Bits[i].SetA( engine, Reg::FromBool( bit));
        }
    }

    pub fn	SetB( &self, engine: &mut SimEngine, val: U32)
    {
        let  	v = u32::from( val);
        for i in 0..N {
            let  	bit = ( ( v >> i) & 1) != 0;
            self._Bits[i].SetB( engine, Reg::FromBool( bit));
        }
    }

    pub fn	SetCIn( &self, engine: &mut SimEngine, val: Reg< bool>)
    {
        if !self._Bits.is_empty() {
            self._Bits[0].SetCIn( engine, val);
        }
    }

    pub fn	GetSum( &self, engine: &SimEngine) -> u32
    {
        let  	mut res = 0u32;
        for i in 0..N {
            if let Some( bit) = engine.GetPortBool( self.Sum( i)) {
                if bit.IsTrue() {
                    res |= 1 << i;
                }
            }
        }
        return res;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Fast 32-bit Bus Adder / ALU Module
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BusAdder32
{
    pub _A: PortId,
    pub _B: PortId,
    pub _Sum: PortId,
    pub _Carry: PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BusAdder32
{
    pub fn	New( layout: &mut Layout, name: &str) -> Self
    {
        let  	kernel = Arc::new( |inVals: &[RegVal], outVals: &mut [RegVal]| {
            let  	a = inVals[0]._Val;
            let  	b = inVals[1]._Val;
            let  	sum = a.wrapping_add( b) & 0xFFFF_FFFF;
            let  	carry = ( a + b) > 0xFFFF_FFFF;
            outVals[0] = RegVal::FromU32( U32( sum as u32));
            outVals[1] = RegVal::FromBool( carry);
        });

        let  	m = layout.AddModule(
            name,
            &[ PortDesc::U32( "a"), PortDesc::U32( "b") ],
            &[ PortDesc::U32( "sum"), PortDesc::Bool( "carry") ],
            KernelKind::Custom( kernel),
        );

        return Self {
            _A: layout.InPort( m, 0).unwrap(),
            _B: layout.InPort( m, 1).unwrap(),
            _Sum: layout.OutPort( m, 0).unwrap(),
            _Carry: layout.OutPort( m, 1).unwrap(),
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
