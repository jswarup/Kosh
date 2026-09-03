//-- adder.rs -----------------------------------------------------------------------------------------------------------------------

use	std::sync::Arc;
use	crate::{
    rube::{
        engine::SimEngine,
        gates::{ AndGate, OrGate, XorGate },
        layout::Layout,
        module::{ KernelKind, ModuleId },
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
    pub _Id:     ModuleId,
    pub _In1:    PortId,
    pub _In2:    PortId,
    pub _Sum:    PortId,
    pub _Carry:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl HalfAdder
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	modId = layout.AddModule(
            name,
            parent,
            &[ PortDesc::Bool( "a"), PortDesc::Bool( "b") ],
            &[ PortDesc::Bool( "sum"), PortDesc::Bool( "carry") ],
            KernelKind::None,
        );

        let  	in1 = layout.InPort( modId, 0).unwrap();
        let  	in2 = layout.InPort( modId, 1).unwrap();
        let  	sum = layout.OutPort( modId, 0).unwrap();
        let  	carry = layout.OutPort( modId, 1).unwrap();

        let  	xorGate = XorGate::New( layout, &format!( "{name}.Xor"), Some( modId));
        let  	andGate = AndGate::New( layout, &format!( "{name}.And"), Some( modId));

        layout.Connect( in1, xorGate.In1());
        layout.Connect( in2, xorGate.In2());
        layout.Connect( in1, andGate.In1());
        layout.Connect( in2, andGate.In2());

        layout.Connect( xorGate.Out(), sum);
        layout.Connect( andGate.Out(), carry);

        layout.SealModule( modId);

        return Self {
            _Id:     modId,
            _In1:    in1,
            _In2:    in2,
            _Sum:    sum,
            _Carry:  carry,
        };
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
    pub const fn	Sum( &self) -> PortId
    {
        return self._Sum;
    }

    #[inline]
    pub const fn	Carry( &self) -> PortId
    {
        return self._Carry;
    }

    #[inline]
    pub fn	SetA( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self._In1, val);
    }

    #[inline]
    pub fn	SetB( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self._In2, val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Bit Full Adder
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FullAdder
{
    pub _Id:     ModuleId,
    pub _In1:    PortId,
    pub _In2:    PortId,
    pub _CIn:    PortId,
    pub _Sum:    PortId,
    pub _Carry:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl FullAdder
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	modId = layout.AddModule(
            name,
            parent,
            &[ PortDesc::Bool( "a"), PortDesc::Bool( "b"), PortDesc::Bool( "cin") ],
            &[ PortDesc::Bool( "sum"), PortDesc::Bool( "carry") ],
            KernelKind::None,
        );

        let  	in1 = layout.InPort( modId, 0).unwrap();
        let  	in2 = layout.InPort( modId, 1).unwrap();
        let  	cin = layout.InPort( modId, 2).unwrap();
        let  	sum = layout.OutPort( modId, 0).unwrap();
        let  	carry = layout.OutPort( modId, 1).unwrap();

        let  	ha1 = HalfAdder::New( layout, &format!( "{name}.HA1"), Some( modId));
        let  	ha2 = HalfAdder::New( layout, &format!( "{name}.HA2"), Some( modId));
        let  	orGate = OrGate::New( layout, &format!( "{name}.Or"), Some( modId));

        // Pass-down
        layout.Connect( in1, ha1.In1());
        layout.Connect( in2, ha1.In2());
        layout.Connect( cin, ha2.In2());

        // Sibling-to-sibling
        layout.Connect( ha1.Sum(), ha2.In1());
        layout.Connect( ha1.Carry(), orGate.In1());
        layout.Connect( ha2.Carry(), orGate.In2());

        // Pass-up
        layout.Connect( ha2.Sum(), sum);
        layout.Connect( orGate.Out(), carry);

        layout.SealModule( modId);

        return Self {
            _Id:     modId,
            _In1:    in1,
            _In2:    in2,
            _CIn:    cin,
            _Sum:    sum,
            _Carry:  carry,
        };
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
    pub const fn	CIn( &self) -> PortId
    {
        return self._CIn;
    }

    #[inline]
    pub const fn	Sum( &self) -> PortId
    {
        return self._Sum;
    }

    #[inline]
    pub const fn	Carry( &self) -> PortId
    {
        return self._Carry;
    }

    #[inline]
    pub fn	SetA( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self._In1, val);
    }

    #[inline]
    pub fn	SetB( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self._In2, val);
    }

    #[inline]
    pub fn	SetCIn( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self._CIn, val);
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// N-Bit Ripple Carry Adder
#[derive( Clone, Debug)]
pub struct Adder< const N: usize>
{
    pub _Id:     ModuleId,
    pub _A:      Buff< PortId>,
    pub _B:      Buff< PortId>,
    pub _CIn:    PortId,
    pub _Sum:    Buff< PortId>,
    pub _Carry:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< const N: usize> Adder< N>
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	mut inDescs = Stash::WithCapacity( U32( ( 2 * N + 1) as u32));
        for i in 0..N {
            inDescs.Push( PortDesc::Bool( format!( "a{i}")));
        }
        for i in 0..N {
            inDescs.Push( PortDesc::Bool( format!( "b{i}")));
        }
        inDescs.Push( PortDesc::Bool( "cin"));

        let  	mut outDescs = Stash::WithCapacity( U32( ( N + 1) as u32));
        for i in 0..N {
            outDescs.Push( PortDesc::Bool( format!( "sum{i}")));
        }
        outDescs.Push( PortDesc::Bool( "carry"));

        let  	modId = layout.AddModule( name, parent, inDescs.Slice(), outDescs.Slice(), KernelKind::None);

        let  	mut aPorts = Stash::WithCapacity( U32( N as u32));
        let  	mut bPorts = Stash::WithCapacity( U32( N as u32));
        for i in 0..N {
            aPorts.Push( layout.InPort( modId, i as u32).unwrap());
            bPorts.Push( layout.InPort( modId, ( N + i) as u32).unwrap());
        }
        let  	cinPort = layout.InPort( modId, ( 2 * N) as u32).unwrap();

        let  	mut sumPorts = Stash::WithCapacity( U32( N as u32));
        for i in 0..N {
            sumPorts.Push( layout.OutPort( modId, i as u32).unwrap());
        }
        let  	carryPort = layout.OutPort( modId, N as u32).unwrap();

        let  	mut fullAdders: Stash< FullAdder> = Stash::WithCapacity( U32( N as u32));
        USeg::New( U32::_0, U32( N as u32)).Traverse( |i| {
            let  	idx = i.AsUsize();
            let  	bit = FullAdder::New( layout, &format!( "{name}.Bit{}", i.0), Some( modId));

            layout.Connect( aPorts[idx], bit.In1());
            layout.Connect( bPorts[idx], bit.In2());

            if i == U32::_0 {
                layout.Connect( cinPort, bit.CIn());
            } else {
                let  	prevCarry = fullAdders[idx - 1].Carry();
                layout.Connect( prevCarry, bit.CIn());
            }

            layout.Connect( bit.Sum(), sumPorts[idx]);
            fullAdders.Push( bit);
        });

        layout.Connect( fullAdders[N - 1].Carry(), carryPort);
        layout.SealModule( modId);

        return Self {
            _Id:     modId,
            _A:      aPorts.IntoBuff(),
            _B:      bPorts.IntoBuff(),
            _CIn:    cinPort,
            _Sum:    sumPorts.IntoBuff(),
            _Carry:  carryPort,
        };
    }

    #[inline]
    pub fn	SetA( &self, engine: &mut SimEngine, val: U32)
    {
        let  	v = val.0 as usize;
        for i in 0..N {
            let  	bitVal = ( ( v >> i) & 1) != 0;
            engine.SetPortBool( self._A[i], Reg::FromBool( bitVal));
        }
    }

    #[inline]
    pub fn	SetB( &self, engine: &mut SimEngine, val: U32)
    {
        let  	v = val.0 as usize;
        for i in 0..N {
            let  	bitVal = ( ( v >> i) & 1) != 0;
            engine.SetPortBool( self._B[i], Reg::FromBool( bitVal));
        }
    }

    #[inline]
    pub fn	SetCarryIn( &self, engine: &mut SimEngine, val: Reg)
    {
        engine.SetPortBool( self._CIn, val);
    }

    #[inline]
    pub fn	GetSum( &self, engine: &SimEngine) -> usize
    {
        let  	mut sum = 0;
        for i in 0..N {
            if let Some( bit) = engine.GetPortBool( self._Sum[i]) {
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
        return self._Carry;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 32-Bit High-Performance Bus Adder using custom word-level kernel
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BusAdder32
{
    pub _A:      PortId,
    pub _B:      PortId,
    pub _Sum:    PortId,
    pub _Carry:  PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl BusAdder32
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
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

        let  	a = layout.InPort( modId, 0).unwrap();
        let  	b = layout.InPort( modId, 1).unwrap();
        let  	sum = layout.OutPort( modId, 0).unwrap();
        let  	carry = layout.OutPort( modId, 1).unwrap();

        layout.SealModule( modId);

        return Self {
            _A:      a,
            _B:      b,
            _Sum:    sum,
            _Carry:  carry,
        };
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
