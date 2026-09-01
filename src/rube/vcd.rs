//-- vcd.rs -----------------------------------------------------------------------------------------------------------------------

use	crate::{
    rube::{
        engine::SimEngine,
        layout::Layout,
        reg::Reg,
        trigger::ITriggerWad,
    },
    silo::{ IAccess, U32, USeg },
};

//---------------------------------------------------------------------------------------------------------------------------------

pub struct VcdWriter
{
    _TrigToIdStr: Vec< String>,
    _TrigBits: Vec< u32>,
}

//---------------------------------------------------------------------------------------------------------------------------------

impl VcdWriter
{
    pub fn	New( layout: &Layout, engine: &SimEngine) -> Self
    {
        let  	trigCount = engine.Triggers().Size().0 as usize;
        let  	mut trigToIdStr = vec![String::new(); trigCount];
        let  	mut trigBits = vec![0; trigCount];

        USeg::New( U32::_0, engine.Triggers().Size()).Traverse( |i| {
            trigToIdStr[i.AsUsize()] = Self::GenerateIdStr( i.0);
        });

        // Resolve bit width for each trigger by finding the first port mapped to it
        layout.Ports().iter().enumerate().for_each( |( pIdx, port)| {
            if let Some( trigId) = engine.GetPortTrigger( crate::rube::port::PortId( U32( pIdx as u32))) {
                trigBits[trigId.0 as usize] = port._Type.Bits();
            }
        });

        return Self {
            _TrigToIdStr: trigToIdStr,
            _TrigBits: trigBits,
        };
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	WriteHeader( &self, layout: &Layout, engine: &SimEngine, out: &mut String)
    {
        out.push_str( "$version\n   Kosh Rube Engine\n$end\n");
        out.push_str( "$timescale 1ns $end\n");

        layout.Modules().iter().for_each( |module| {
            out.push_str( &format!( "$scope module {} $end\n", module._Name));
            
            module._InPorts.Arr().Traverse( |&portId| {
                if let Some( port) = layout.Port( portId) {
                    if let Some( trigId) = engine.GetPortTrigger( portId) {
                        let  	vcdId = &self._TrigToIdStr[trigId.0 as usize];
                        let  	bits = port._Type.Bits();
                        out.push_str( &format!( "$var wire {} {} {} $end\n", bits, vcdId, port._Name));
                    }
                }
            });

            module._OutPorts.Arr().Traverse( |&portId| {
                if let Some( port) = layout.Port( portId) {
                    if let Some( trigId) = engine.GetPortTrigger( portId) {
                        let  	vcdId = &self._TrigToIdStr[trigId.0 as usize];
                        let  	bits = port._Type.Bits();
                        out.push_str( &format!( "$var wire {} {} {} $end\n", bits, vcdId, port._Name));
                    }
                }
            });

            out.push_str( "$upscope $end\n");
        });

        out.push_str( "$enddefinitions $end\n");
        out.push_str( "$dumpvars\n");

        // Dump initial values
        USeg::New( U32::_0, engine.Triggers().Size()).Traverse( |i| {
            let  	val = engine.GetTrigger( i);
            let  	bits = self._TrigBits[i.AsUsize()];
            let  	vcdId = &self._TrigToIdStr[i.AsUsize()];
            Self::FormatVal( val, bits, vcdId, out);
        });

        out.push_str( "$end\n");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub fn	DumpCycle( &self, engine: &SimEngine, out: &mut String)
    {
        out.push_str( &format!( "#{}\n", engine.CycleCount()));
        
        USeg::New( U32::_0, engine.Triggers().Size()).Traverse( |i| {
            if engine.Triggers().IsEdge( i) {
                let  	val = engine.GetTrigger( i);
                let  	bits = self._TrigBits[i.AsUsize()];
                let  	vcdId = &self._TrigToIdStr[i.AsUsize()];
                Self::FormatVal( val, bits, vcdId, out);
            }
        });
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	GenerateIdStr( mut val: u32) -> String
    {
        let  	mut res = String::new();
        loop {
            let  	rem = ( val % 94) as u8;
            res.push( ( rem + 33) as char);
            val /= 94;
            if val == 0 {
                break;
            }
        }
        return res;
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	FormatVal( val: Reg, bits: u32, id: &str, out: &mut String)
    {
        if bits == 1 {
            if val.IsX() {
                out.push( 'x');
            } else {
                out.push( if val.IsTrue() { '1' } else { '0' });
            }
            out.push_str( id);
            out.push( '\n');
        } else {
            out.push( 'b');
            let  	mut started = false;
            // Iterate from MSB to LSB
            for i in ( 0..bits).rev() {
                let  	mask = 1u64 << i;
                if ( val._X & mask) != 0 {
                    out.push( 'x');
                    started = true;
                } else if ( val._Val & mask) != 0 {
                    out.push( '1');
                    started = true;
                } else {
                    if started || i == 0 {
                        out.push( '0');
                    }
                }
            }
            out.push( ' ');
            out.push_str( id);
            out.push( '\n');
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
