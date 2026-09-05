//-- adder.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::{
    flux::{ FieldExp, IFluxExportSource, FieldImp, IFluxImportSource },
    rube::{
        engine::SimEngine,
        gates::{ AndGate, OrGate, XorGate },
        layout::Layout,
        module::{ HierModule, HierarchyError, IModule, KernelKind, ModuleId, PortSpec, SealedModule },
        port::{ PortDesc, PortId, PortType },
        reg::Reg,
    },
    silo::{ Buff, Stash, U32, USeg },
};

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
            KernelKind::Custom( "BusAdder32_Kernel"),
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

    pub fn	Hierarchical( name: &str) -> Result< SealedModule, HierarchyError>
    {
        let  	mut inPorts = Stash::New();
        inPorts.Push( PortSpec::Input( "a", PortType::U32Val));
        inPorts.Push( PortSpec::Input( "b", PortType::U32Val));

        let  	mut outPorts = Stash::New();
        outPorts.Push( PortSpec::Output( "sum", PortType::U32Val));
        outPorts.Push( PortSpec::Output( "carry", PortType::Bool));

        let  	mut module = HierModule::New( name, inPorts.IntoBuff(), outPorts.IntoBuff());
        module._Kernel = KernelKind::Custom( "BusAdder32_Kernel");
        return module.Seal();
    }

    #[inline]
    pub const fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

pub struct AdderPipeline;

//---------------------------------------------------------------------------------------------------------------------------------

impl AdderPipeline
{
    pub fn	Hierarchical( name: &str) -> Result< SealedModule, HierarchyError>
    {
        let  	mut inPorts = Stash::New();
        inPorts.Push( PortSpec::Input( "a", PortType::U32Val));
        inPorts.Push( PortSpec::Input( "b", PortType::U32Val));
        inPorts.Push( PortSpec::Input( "c", PortType::U32Val));

        let  	mut outPorts = Stash::New();
        outPorts.Push( PortSpec::Output( "sum", PortType::U32Val));
        outPorts.Push( PortSpec::Output( "carry", PortType::Bool));

        let  	mut module = HierModule::New( name, inPorts.IntoBuff(), outPorts.IntoBuff());

        let  	adder1 = module.AddSealedSubModule( "Adder1", BusAdder32::Hierarchical( "Adder1")?)?;
        let  	adder2 = module.AddSealedSubModule( "Adder2", BusAdder32::Hierarchical( "Adder2")?)?;

        // Internal chaining: adder1.sum (0) -> adder2.a (0)
        module.ConnectSubModules( adder1, 0, adder2, 0)?;

        // Boundary bindings:
        // this.a (0) -> adder1.a (0)
        module.BindInPort( 0, adder1, 0)?;
        // this.b (1) -> adder1.b (1)
        module.BindInPort( 1, adder1, 1)?;
        // this.c (2) -> adder2.b (1)
        module.BindInPort( 2, adder2, 1)?;

        // adder2.sum (0) -> this.sum (0)
        module.BindOutPort( 0, adder2, 0)?;
        // adder2.carry (1) -> this.carry (1)
        module.BindOutPort( 1, adder2, 1)?;

        return module.Seal();
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
