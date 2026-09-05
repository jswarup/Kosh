//-- fifo.rs ------------------------------------------------------------------------------------------------------------------------

use	std::sync::{ Arc, Mutex };
use	std::collections::VecDeque;

use	crate::{
    rube::{
        layout::Layout,
        module::{ IModule, KernelKind, ModuleId },
        port::{ PortDesc, PortId, PortType },
        reg::Reg,
    },
    silo::U32,
};

//---------------------------------------------------------------------------------------------------------------------------------

/// A synchronous FIFO (First-In, First-Out) memory queue with configurable depth and width.
#[derive( Clone, Debug)]
pub struct Fifo
{
    pub _Id:       ModuleId,
    pub _Clk:      PortId,
    pub _Reset:    PortId,
    pub _Push:     PortId,
    pub _Pop:      PortId,
    pub _DataIn:   PortId,
    pub _DataOut:  PortId,
    pub _Empty:    PortId,
    pub _Full:     PortId,
}

//---------------------------------------------------------------------------------------------------------------------------------

struct FifoState
{
    _Queue: VecDeque< u64>,
    _Depth: usize,
    _WidthMask: u64,
    _LastClk: bool,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Fifo
{
    pub fn	New( layout: &mut Layout, name: &str, depth: usize, width: u32, parent: Option< ModuleId>) -> Self
    {
        let  	inPorts = [
            PortDesc::Bool( "Clk"),
            PortDesc::Bool( "Reset"),
            PortDesc::Bool( "Push"),
            PortDesc::Bool( "Pop"),
            PortDesc::New( "DataIn", PortType::Custom( U32( width))),
        ];

        let  	outPorts = [
            PortDesc::New( "DataOut", PortType::Custom( U32( width))),
            PortDesc::Bool( "Empty"),
            PortDesc::Bool( "Full"),
        ];

        let  	widthMask = if width >= 64 { 0xFFFF_FFFF_FFFF_FFFF } else { ( 1u64 << width) - 1 };
        let  	state = Arc::new( Mutex::new( FifoState {
            _Queue: VecDeque::with_capacity( depth),
            _Depth: depth,
            _WidthMask: widthMask,
            _LastClk: false,
        }));

        let  	kernel = Arc::new( move |inVals: &[Reg], outVals: &mut [Reg]| {
            let  	mut s = state.lock().unwrap();
            let  	clk = inVals[0].IsTrue();
            let  	reset = inVals[1].IsTrue();

            let  	clkRose = clk && !s._LastClk;
            s._LastClk = clk;

            if reset {
                s._Queue.clear();
            } else if clkRose {
                let  	push = inVals[2].IsTrue();
                let  	pop = inVals[3].IsTrue();
                let  	dataIn = inVals[4].Val() & s._WidthMask;

                if push && s._Queue.len() < s._Depth {
                    s._Queue.push_back( dataIn);
                }
                if pop && !s._Queue.is_empty() {
                    s._Queue.pop_front();
                }
            } else {
                return; // No reset and no clock edge -> do nothing, spurious trigger on inputs
            }

            let  	empty = s._Queue.is_empty();
            let  	full = s._Queue.len() == s._Depth;
            let  	dataOut = s._Queue.front().copied().unwrap_or( 0);

            outVals[0] = Reg::Known( dataOut);
            outVals[1] = Reg::FromBool( empty);
            outVals[2] = Reg::FromBool( full);
        });

        let  	modId = layout.AddModule( name, parent, &inPorts, &outPorts, KernelKind::Behavioral( kernel));
        layout.SealModule( modId);

        return Self {
            _Id:      modId,
            _Clk:     layout.InPort( modId, 0).unwrap(),
            _Reset:   layout.InPort( modId, 1).unwrap(),
            _Push:    layout.InPort( modId, 2).unwrap(),
            _Pop:     layout.InPort( modId, 3).unwrap(),
            _DataIn:  layout.InPort( modId, 4).unwrap(),
            _DataOut: layout.OutPort( modId, 0).unwrap(),
            _Empty:   layout.OutPort( modId, 1).unwrap(),
            _Full:    layout.OutPort( modId, 2).unwrap(),
        };
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[inline]
    pub const fn	Id( &self) -> ModuleId { self._Id }

    #[inline]
    pub const fn	Clk( &self) -> PortId { self._Clk }

    #[inline]
    pub const fn	Reset( &self) -> PortId { self._Reset }

    #[inline]
    pub const fn	Push( &self) -> PortId { self._Push }

    #[inline]
    pub const fn	Pop( &self) -> PortId { self._Pop }

    #[inline]
    pub const fn	DataIn( &self) -> PortId { self._DataIn }

    #[inline]
    pub const fn	DataOut( &self) -> PortId { self._DataOut }

    #[inline]
    pub const fn	Empty( &self) -> PortId { self._Empty }

    #[inline]
    pub const fn	Full( &self) -> PortId { self._Full }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl IModule for Fifo
{
    #[inline]
    fn	Id( &self) -> ModuleId
    {
        return self._Id;
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

impl Default for Fifo
{
    fn default() -> Self
    {
        return Self {
            _Id:      ModuleId::default(),
            _Clk:     PortId::default(),
            _Reset:   PortId::default(),
            _Push:    PortId::default(),
            _Pop:     PortId::default(),
            _DataIn:  PortId::default(),
            _DataOut: PortId::default(),
            _Empty:   PortId::default(),
            _Full:    PortId::default(),
        };
    }
}

crate::ImplFluxSource!( Fifo, _Id, _Clk, _Reset, _Push, _Pop, _DataIn, _DataOut, _Empty, _Full);

//---------------------------------------------------------------------------------------------------------------------------------

crate::DefineModuleInterface!(
    Fifo,
    "fifo",
    "1.0.0",
    "Synchronous FIFO (First-In, First-Out) memory queue",
    inports: [ ("Clk", 1), ("Reset", 1), ("Push", 1), ("Pop", 1), ("DataIn", 32) ],
    outports: [ ("DataOut", 32), ("Empty", 1), ("Full", 1) ]
);
