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
    },
    silo::{ Buff, Stash, U32, USeg },
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
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	xorGate = XorGate::New( layout, &format!( "{name}.Xor"), parent);
        let  	andGate = AndGate::New( layout, &format!( "{name}.And"), parent);

        return Self {
            _Xor: xorGate,
            _And: andGate,
        };
    }

    #[inline]
    pub const fn	In1( &self) -> PortId
    {
        return self._Xor.In1();
    }

    #[inline]
    pub const fn	In2( &self) -> PortId
    {
        return self._Xor.In2();
    }

    #[inline]
    pub const fn	Sum( &self) -> PortId
    {
        return self._Xor.Out();
    }

    #[inline]
    pub const fn	Carry( &self) -> PortId
    {
        return self._And.Out();
    }

    #[inline]
    pub fn	SetA( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self._Xor.In1(), val);
        engine.SetPortBool( self._And.In1(), val);
    }

    #[inline]
    pub fn	SetB( &self, engine: &mut SimEngine, val: Reg)
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
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	ha1 = HalfAdder::New( layout, &format!( "{name}.HA1"), parent);
        let  	ha2 = HalfAdder::New( layout, &format!( "{name}.HA2"), parent);
        let  	orGate = OrGate::New( layout, &format!( "{name}.Or"), parent);

        layout.Connect( ha1.Sum(), ha2._Xor.In1());
        layout.Connect( ha1.Sum(), ha2._And.In1());
        layout.Connect( ha1.Carry(), orGate.In1());
        layout.Connect( ha2.Carry(), orGate.In2());

        return Self {
            _HA1: ha1,
            _HA2: ha2,
            _Or: orGate,
        };
    }

    #[inline]
    pub const fn	Sum( &self) -> PortId
    {
        return self._HA2.Sum();
    }

    #[inline]
    pub const fn	Carry( &self) -> PortId
    {
        return self._Or.Out();
    }

    #[inline]
    pub fn	SetA( &self, engine: &mut SimEngine, val: Reg)
    {
        self._HA1.SetA( engine, val);
    }

    #[inline]
    pub fn	SetB( &self, engine: &mut SimEngine, val: Reg)
    {
        self._HA1.SetB( engine, val);
    }

    #[inline]
    pub fn	SetCIn( &self, engine: &mut SimEngine, val: Reg)
    {
        self._HA2.SetB( engine, val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// N-Bit Ripple Carry Adder
#[derive( Clone, Debug)]
pub struct Adder< const N: usize>
{
    pub _Bits: Buff< FullAdder>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< const N: usize> Adder< N>
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	mut bits: Stash< FullAdder> = Stash::WithCapacity( U32( N as u32));
        USeg::New( U32::_0, U32( N as u32)).Traverse( |i| {
            let  	bit = FullAdder::New( layout, &format!( "{name}.Bit{}", i.0), parent);
            if i > U32::_0 {
                let  	prevCarry = bits.Slice()[i.AsUsize() - 1].Carry();
                layout.Connect( prevCarry, bit._HA2._Xor.In2());
                layout.Connect( prevCarry, bit._HA2._And.In2());
            }
            bits.Push( bit);
        });

        return Self { _Bits: bits.IntoBuff() };
    }

    #[inline]
    pub fn	SetA( &self, engine: &mut SimEngine, val: U32)
    {
        let  	v = val.0 as usize;
        for i in 0..N {
            let  	bitVal = ( ( v >> i) & 1) != 0;
            self._Bits[i].SetA( engine, Reg::FromBool( bitVal));
        }
    }

    #[inline]
    pub fn	SetB( &self, engine: &mut SimEngine, val: U32)
    {
        let  	v = val.0 as usize;
        for i in 0..N {
            let  	bitVal = ( ( v >> i) & 1) != 0;
            self._Bits[i].SetB( engine, Reg::FromBool( bitVal));
        }
    }

    #[inline]
    pub fn	SetCarryIn( &self, engine: &mut SimEngine, val: Reg)
    {
        if !self._Bits.is_empty() {
            self._Bits[0].SetCIn( engine, val);
        }
    }

    #[inline]
    pub fn	GetSum( &self, engine: &SimEngine) -> usize
    {
        let  	mut sum = 0;
        for i in 0..N {
            if let Some( bit) = engine.GetPortBool( self._Bits[i].Sum()) {
                if bit.IsTrue() {
                    sum |= 1 << i;
                }
            }
        }
        return sum;
    }

    #[inline]
    pub fn	Carry( &self) -> PortId
    {
        return self._Bits[N - 1].Carry();
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 32-Bit High-Performance Bus Adder using custom word-level kernel
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
    pub fn	New( layout: &mut Layout, name: &str, parent: Option<crate::rube::module::ModuleId>) -> Self
    {
        let  	adderKernel = Arc::new( |inVals: &[Reg], outVals: &mut [Reg]| {
            let  	aVal = inVals[0].Val();
            let  	bVal = inVals[1].Val();
            let  	sum = aVal.wrapping_add( bVal) & 0xFFFF_FFFF;
            let  	carry = ( aVal + bVal) > 0xFFFF_FFFF;

            outVals[0] = Reg::FromU32( U32( sum as u32));
            outVals[1] = Reg::FromBool( carry);
        });

        let  	modId = layout.AddModule(
            name,
            parent,
            &[ PortDesc::U32( "a"), PortDesc::U32( "b") ],
            &[ PortDesc::U32( "sum"), PortDesc::Bool( "carry") ],
            KernelKind::Custom( adderKernel),
        );

        return Self {
            _A: layout.InPort( modId, 0).unwrap(),
            _B: layout.InPort( modId, 1).unwrap(),
            _Sum: layout.OutPort( modId, 0).unwrap(),
            _Carry: layout.OutPort( modId, 1).unwrap(),
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
