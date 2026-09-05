//-- adder.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::{ FieldExp, IFluxExportSource, FieldImp, IFluxImportSource },
    rube::{
        engine::SimEngine,
        gates::{ AndGate, OrGate, XorGate },
        layout::Layout,
        module::{ IModule, KernelKind, ModuleId },
        port::{ PortDesc, PortId },
        reg::Reg,
        kernel::{ IKernel, KernelSignature, KernelError },
    },
    silo::{ Buff, Stash, U32, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub struct BusAdder32Kernel;

impl IKernel for BusAdder32Kernel
{
    fn	Name( &self) -> &'static str { "BusAdder32_Kernel" }
    fn	Version( &self) -> &'static str { "1.0.0" }
    fn	Signature( &self) -> &'static KernelSignature
    {
        static SIG: KernelSignature = KernelSignature {
            _InputPorts: 2,
            _OutputPorts: 2,
            _Parameters: &[],
        };
        &SIG
    }

    fn	Execute( &self, inputs: &[Reg], outputs: &mut [Reg]) -> Result< (), KernelError>
    {
        let  	aVal = inputs[0].Val();
        let  	bVal = inputs[1].Val();
        let  	sum = aVal.wrapping_add( bVal) & 0xFFFF_FFFF;
        let  	carry = ( aVal + bVal) > 0xFFFF_FFFF;

        outputs[0] = Reg::FromU32( U32( sum as u32));
        outputs[1] = Reg::FromBool( carry);
        Ok( ())
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Bit Half Adder ( XOR + AND)
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct HalfAdder
{
    pub _Id:     ModuleId,
    pub _Xor:    XorGate,
    pub _And:    AndGate,
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
            _Xor:    xorGate,
            _And:    andGate,
            _In1:    in1,
            _In2:    in2,
            _Sum:    sum,
            _Carry:  carry,
        };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for HalfAdder
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 1-Bit Full Adder
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct FullAdder
{
    pub _Id:     ModuleId,
    pub _HA1:    HalfAdder,
    pub _HA2:    HalfAdder,
    pub _Or:     OrGate,
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
            _HA1:    ha1,
            _HA2:    ha2,
            _Or:     orGate,
            _In1:    in1,
            _In2:    in2,
            _CIn:    cin,
            _Sum:    sum,
            _Carry:  carry,
        };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
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

impl IModule for FullAdder
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// N-Bit Ripple Carry Adder
#[derive( Clone, Debug)]
pub struct Adder< const N: usize>
{
    pub _Id:     ModuleId,
    pub _Bits:   Buff< FullAdder>,
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
        let  	mut inDescs = Stash::WithCapacity( 2 * N + 1);
        USeg::New( 0, N).Traverse( |i| {
            inDescs.Push( PortDesc::Bool( format!( "a{i}")));
        });
        USeg::New( 0, N).Traverse( |i| {
            inDescs.Push( PortDesc::Bool( format!( "b{i}")));
        });
        inDescs.Push( PortDesc::Bool( "cin"));

        let  	mut outDescs = Stash::WithCapacity( N + 1);
        USeg::New( 0, N).Traverse( |i| {
            outDescs.Push( PortDesc::Bool( format!( "sum{i}")));
        });
        outDescs.Push( PortDesc::Bool( "carry"));

        let  	modId = layout.AddModule( name, parent, inDescs.Slice(), outDescs.Slice(), KernelKind::None);

        let  	mut aPorts = Stash::WithCapacity( N);
        let  	mut bPorts = Stash::WithCapacity( N);
        USeg::New( 0, N).Traverse( |i| {
            aPorts.Push( layout.InPort( modId, i).unwrap());
            bPorts.Push( layout.InPort( modId, i + N).unwrap());
        });
        let  	cinPort = layout.InPort( modId, 2 * N).unwrap();

        let  	mut sumPorts = Stash::WithCapacity( N);
        USeg::New( 0, N).Traverse( |i| {
            sumPorts.Push( layout.OutPort( modId, i).unwrap());
        });
        let  	carryPort = layout.OutPort( modId, N).unwrap();

        let  	mut fullAdders: Stash< FullAdder> = Stash::WithCapacity( N);
        USeg::New( 0, N).Traverse( |i| {
            let  	bit = FullAdder::New( layout, &format!( "{name}.Bit{i}"), Some( modId));

            layout.Connect( aPorts[i], bit.In1());
            layout.Connect( bPorts[i], bit.In2());

            if i == 0 {
                layout.Connect( cinPort, bit.CIn());
            } else {
                let  	prevCarry = fullAdders[i - 1].Carry();
                layout.Connect( prevCarry, bit.CIn());
            }

            layout.Connect( bit.Sum(), sumPorts[i]);
            fullAdders.Push( bit);
        });

        layout.Connect( fullAdders[N - 1].Carry(), carryPort);
        layout.SealModule( modId);

        return Self {
            _Id:     modId,
            _Bits:   fullAdders.IntoBuff(),
            _A:      aPorts.IntoBuff(),
            _B:      bPorts.IntoBuff(),
            _CIn:    cinPort,
            _Sum:    sumPorts.IntoBuff(),
            _Carry:  carryPort,
        };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }

    #[inline]
    pub fn	SetA< V: Into< U32>>( &self, engine: &mut SimEngine, val: V)
    {
        let  	v = val.into().AsUsize();
        USeg::New( 0, N).Traverse( |i| {
            let  	bitVal = ( ( v >> i.0) & 1) != 0;
            engine.SetPortBool( self._A[i], Reg::FromBool( bitVal));
        });
    }

    #[inline]
    pub fn	SetB< V: Into< U32>>( &self, engine: &mut SimEngine, val: V)
    {
        let  	v = val.into().AsUsize();
        USeg::New( 0, N).Traverse( |i| {
            let  	bitVal = ( ( v >> i.0) & 1) != 0;
            engine.SetPortBool( self._B[i], Reg::FromBool( bitVal));
        });
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
        USeg::New( 0, N).Traverse( |i| {
            if let Some( bit) = engine.GetPortBool( self._Sum[i]) {
                if bit.IsTrue() {
                    sum |= 1 << i.0;
                }
            }
        });
        return sum;
    }

    #[inline]
    pub fn	Carry( &self) -> PortId
    {
        return self._Carry;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl< const N: usize> IModule for Adder< N>
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

/// 32-Bit High-Performance Bus Adder using custom word-level kernel
#[derive( Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct BusAdder32
{
    pub _Id:     ModuleId,
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
        let  	modId = layout.AddModule(
            name,
            parent,
            &[ PortDesc::U32( "a"), PortDesc::U32( "b") ],
            &[ PortDesc::U32( "sum"), PortDesc::Bool( "carry") ],
            KernelKind::Trait( std::sync::Arc::new( BusAdder32Kernel)),
        );

        let  	a = layout.InPort( modId, 0).unwrap();
        let  	b = layout.InPort( modId, 1).unwrap();
        let  	sum = layout.OutPort( modId, 0).unwrap();
        let  	carry = layout.OutPort( modId, 1).unwrap();

        layout.SealModule( modId);

        return Self {
            _Id:     modId,
            _A:      a,
            _B:      b,
            _Sum:    sum,
            _Carry:  carry,
        };
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }

    #[inline]
    pub fn	A( &self) -> PortId { self._A }

    #[inline]
    pub fn	B( &self) -> PortId { self._B }

    #[inline]
    pub fn	Sum( &self) -> PortId { self._Sum }

    #[inline]
    pub fn	Carry( &self) -> PortId { self._Carry }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct AdderPipeline
{
    pub _Id:      ModuleId,
    pub _A:       PortId,
    pub _B:       PortId,
    pub _C:       PortId,
    pub _Sum:     PortId,
    pub _Carry:   PortId,
    pub _Adder1:  BusAdder32,
    pub _Adder2:  BusAdder32,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl AdderPipeline
{
    pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
    {
        let  	modId = layout.AddModule(
            name,
            parent,
            &[ PortDesc::U32( "a"), PortDesc::U32( "b"), PortDesc::U32( "c") ],
            &[ PortDesc::U32( "sum"), PortDesc::Bool( "carry") ],
            KernelKind::None,
        );

        let  	a = layout.InPort( modId, 0).unwrap();
        let  	b = layout.InPort( modId, 1).unwrap();
        let  	c = layout.InPort( modId, 2).unwrap();
        let  	sum = layout.OutPort( modId, 0).unwrap();
        let  	carry = layout.OutPort( modId, 1).unwrap();

        let  	adder1 = BusAdder32::New( layout, &format!( "{name}.Adder1"), Some( modId));
        let  	adder2 = BusAdder32::New( layout, &format!( "{name}.Adder2"), Some( modId));

        // Submodule chaining: Adder1.Sum -> Adder2.A
        layout.Connect( adder1.Sum(), adder2.A());

        // Boundary pass-down bindings
        layout.Connect( a, adder1.A());
        layout.Connect( b, adder1.B());
        layout.Connect( c, adder2.B());

        // Boundary pass-up bindings
        layout.Connect( adder2.Sum(), sum);
        layout.Connect( adder2.Carry(), carry);

        layout.SealModule( modId);

        return Self {
            _Id:      modId,
            _A:       a,
            _B:       b,
            _C:       c,
            _Sum:     sum,
            _Carry:   carry,
            _Adder1:  adder1,
            _Adder2:  adder2,
        };
    }

    #[inline]
    pub fn	A( &self) -> PortId { self._A }

    #[inline]
    pub fn	B( &self) -> PortId { self._B }

    #[inline]
    pub fn	C( &self) -> PortId { self._C }

    #[inline]
    pub fn	Sum( &self) -> PortId { self._Sum }

    #[inline]
    pub fn	Carry( &self) -> PortId { self._Carry }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IModule for AdderPipeline
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IModule for BusAdder32
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

crate::ImplFluxSource!( BusAdder32, _Id, _A, _B, _Sum, _Carry);
crate::ImplFluxSource!( HalfAdder, _Id, _Xor, _And, _In1, _In2, _Sum, _Carry);
crate::ImplFluxSource!( FullAdder, _Id, _HA1, _HA2, _Or, _In1, _In2, _CIn, _Sum, _Carry);

//---------------------------------------------------------------------------------------------------------------------------------

crate::DefineModuleInterface!(
    BusAdder32,
    "bus_adder_32",
    "1.0.0",
    "32-bit high-performance bus adder with carry-out",
    inports: [ ("a", 32), ("b", 32) ],
    outports: [ ("sum", 32), ("carry", 1) ]
);

crate::DefineModuleInterface!(
    HalfAdder,
    "half_adder",
    "1.0.0",
    "1-bit half adder",
    inports: [ ("a", 1), ("b", 1) ],
    outports: [ ("sum", 1), ("carry", 1) ]
);

crate::DefineModuleInterface!(
    FullAdder,
    "full_adder",
    "1.0.0",
    "1-bit full adder",
    inports: [ ("a", 1), ("b", 1), ("cin", 1) ],
    outports: [ ("sum", 1), ("carry", 1) ]
);

crate::DefineModuleInterface!(
    AdderPipeline,
    "adder_pipeline",
    "1.0.0",
    "2-stage 32-bit adder pipeline",
    inports: [ ("a", 32), ("b", 32), ("c", 32) ],
    outports: [ ("sum", 32), ("carry", 1) ]
);

//---------------------------------------------------------------------------------------------------------------------------------

impl< const N: usize> IFluxExportSource for Adder< N>
{
    fn	FetchFieldExp< 'a>( &'a self, field: &mut FieldExp< 'a>)
    {
        self._Id.FetchFieldExp( field);
    }
}

impl< const N: usize> IFluxImportSource for Adder< N>
{
    fn	FetchFieldImp< 'a>( &'a mut self, field: &mut FieldImp< 'a>)
    {
        self._Id.FetchFieldImp( field);
    }
}
