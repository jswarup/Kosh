//-- _tests.rs --------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests {
    use	crate::{
        rube::{
            adder::{ Adder, BusAdder32 },
            gates::{ AndGate, NandGate, NotGate, OrGate, XorGate },
            latches::{ CRSLatch, DLatch, RSLatch },
            layout::{ Layout, LayoutError },
            module::KernelKind,
            port::PortDesc,
            portlayout::{ PortLayout, TopologyPort },
            reg::Reg,
            regval::RegVal,
            trigger::TriggerWad,
        },
        silo::{ IEdgeBroadcast, IEdgeConnect, U32 },
    };

    #[test]
    fn	test_adder_basic()
    {
        let  	mut layout = Layout::New();
        const N: usize = 16;
        let  	adder = Adder::< N>::New( &mut layout, "Adder16");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        let  	testCases = [
            ( 0, 0, 0, false),
            ( 1, 1, 2, false),
            ( 5, 7, 12, false),
            ( 255, 255, 510, false),
            ( 0xFFFF, 1, 0, true), // 16-bit overflow
            ( 0x8000, 0x8000, 0, true),
            ( 12345, 54321, 66666, true), // 66666 is > 65535, so overflow
        ];

        for ( aVal, bVal, expectedSum, expectedCarry) in testCases {
            adder.SetA( &mut engine, U32( aVal));
            adder.SetB( &mut engine, U32( bVal));

            // Tick for full ripple carry propagation ( 3 gate delays per bit)
            for _ in 0..( N * 3) {
                engine.Tick();
            }

            let  	sumVal = adder.GetSum( &engine);
            let  	carryVal = engine.GetPortBool( adder.Carry()).unwrap().IsTrue();

            let  	expectedTruncated = expectedSum & 0xFFFF;
            let  	expectedOverflow = expectedSum > 0xFFFF;

            assert_eq!(
                sumVal, expectedTruncated,
                "Sum failed for {} + {}. Expected {}, got {}",
                aVal, bVal, expectedTruncated, sumVal
            );
            assert_eq!(
                carryVal, expectedCarry || expectedOverflow,
                "Carry failed for {} + {}. Expected {}, got {}",
                aVal, bVal, expectedCarry || expectedOverflow, carryVal
            );
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_clocked_rs_latch()
    {
        let  	mut layout = Layout::New();
        let  	crs = CRSLatch::New( &mut layout, "CRSLatch");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // When clk=0, S and R changes have no effect
        crs.SetClk( &mut engine, Reg::FALSE);
        crs.SetS( &mut engine, Reg::TRUE);
        crs.SetR( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::FALSE));

        // Pulse clock high while S=1, R=0 -> Q becomes 1
        crs.SetClk( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( crs.Q1()), Some( Reg::FALSE));

        // Clock back low -> retains Q=1
        crs.SetClk( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));

        // Set R=1, S=0 with clk=0 -> still Q=1
        crs.SetS( &mut engine, Reg::FALSE);
        crs.SetR( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));

        // Pulse clk=1 -> Q resets to 0
        crs.SetClk( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( crs.Q1()), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_d_latch()
    {
        let  	mut layout = Layout::New();
        let  	dLatch = DLatch::New( &mut layout, "DLatch");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // Enable = 0 ( latched): changing D has no effect
        dLatch.SetE( &mut engine, Reg::FALSE);
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::FALSE));

        // Enable = 1 ( transparent): D=1 passes to Q=1
        dLatch.SetE( &mut engine, Reg::TRUE);
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( dLatch.Q1()), Some( Reg::FALSE));

        // D=0 passes to Q=0
        dLatch.SetD( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( dLatch.Q1()), Some( Reg::TRUE));

        // Set D=1, then Enable=0 ( latch 1)
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::TRUE));

        dLatch.SetE( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }

        // Now change D=0 while E=0 -> Q remains 1
        dLatch.SetD( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::TRUE), "Q should stay latched at 1");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_nand_gate()
    {
        let  	mut layout = Layout::New();
        let  	gate = NandGate::New( &mut layout, "Nand");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        let  	truthTable = [
            ( false, false, Reg::TRUE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::FALSE),
        ];

        for ( a, b, expected) in truthTable {
            engine.SetPortBool( gate.In1(), Reg::FromBool( a));
            engine.SetPortBool( gate.In2(), Reg::FromBool( b));
            engine.Tick();
            assert_eq!( engine.GetPortBool( gate.Out()), Some( expected), "NAND failed for {a} and {b}");
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_not_gate()
    {
        let  	mut layout = Layout::New();
        let  	gate = NotGate::New( &mut layout, "Not");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        engine.SetPortBool( gate.In(), Reg::FALSE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( gate.Out()), Some( Reg::TRUE));

        engine.SetPortBool( gate.In(), Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( gate.Out()), Some( Reg::FALSE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_and_gate()
    {
        let  	mut layout = Layout::New();
        let  	gate = AndGate::New( &mut layout, "And");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        let  	truthTable = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::FALSE),
            ( true, false, Reg::FALSE),
            ( true, true, Reg::TRUE),
        ];

        for ( a, b, expected) in truthTable {
            engine.SetPortBool( gate.In1(), Reg::FromBool( a));
            engine.SetPortBool( gate.In2(), Reg::FromBool( b));
            engine.Tick();
            assert_eq!( engine.GetPortBool( gate.Out()), Some( expected), "AND failed for {a} and {b}");
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_or_gate()
    {
        let  	mut layout = Layout::New();
        let  	gate = OrGate::New( &mut layout, "Or");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        let  	truthTable = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::TRUE),
        ];

        for ( a, b, expected) in truthTable {
            engine.SetPortBool( gate.In1(), Reg::FromBool( a));
            engine.SetPortBool( gate.In2(), Reg::FromBool( b));
            engine.Tick();
            assert_eq!( engine.GetPortBool( gate.Out()), Some( expected), "OR failed for {a} and {b}");
        }
    }
    
    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_xor_gate()
    {
        let  	mut layout = Layout::New();
        let  	gate = XorGate::New( &mut layout, "Xor");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        let  	truthTable = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::FALSE),
        ];

        for ( a, b, expected) in truthTable {
            engine.SetPortBool( gate.In1(), Reg::FromBool( a));
            engine.SetPortBool( gate.In2(), Reg::FromBool( b));
            engine.Tick();
            assert_eq!( engine.GetPortBool( gate.Out()), Some( expected), "XOR failed for {a} and {b}");
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_reg_bitwise_operations()
    {
        // Basic NOT
        assert_eq!( !Reg::FALSE, Reg::TRUE);
        assert_eq!( !Reg::TRUE, Reg::FALSE);
        assert_eq!( !Reg::X, Reg::X);

        // Basic AND
        assert_eq!( Reg::TRUE & Reg::TRUE, Reg::TRUE);
        assert_eq!( Reg::TRUE & Reg::FALSE, Reg::FALSE);
        assert_eq!( Reg::FALSE & Reg::X, Reg::FALSE); // 0 & X = 0
        assert_eq!( Reg::TRUE & Reg::X, Reg::X);      // 1 & X = X

        // Basic OR
        assert_eq!( Reg::FALSE | Reg::FALSE, Reg::FALSE);
        assert_eq!( Reg::TRUE | Reg::FALSE, Reg::TRUE);
        assert_eq!( Reg::TRUE | Reg::X, Reg::TRUE);   // 1 | X = 1
        assert_eq!( Reg::FALSE | Reg::X, Reg::X);     // 0 | X = X
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_generic_reg()
    {
        let  	knownVal = Reg::< U32>::Known( U32( 42));
        assert!( knownVal.IsValid());
        assert!( !knownVal.IsX());
        assert_eq!( *knownVal.Val(), U32( 42));

        let  	unknownVal = Reg::< U32>::Unknown( U32( 0));
        assert!( !unknownVal.IsValid());
        assert!( unknownVal.IsX());

        let  	mut defaultReg = Reg::< U32>::default();
        assert!( defaultReg.IsX());
        assert!( !defaultReg.IsValid());

        defaultReg._Val = U32( 100);
        defaultReg._X = false;
        assert_eq!( *defaultReg.Val(), U32( 100));
        assert!( defaultReg.IsValid());

        defaultReg.ConvertX();
        assert!( defaultReg.IsX());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_generic_trigger_wad()
    {
        let  	mut triggers = TriggerWad::< U32>::New();
        let  	s0 = triggers.Add( "bus0", Reg::< U32>::Known( U32( 10)));
        let  	s1 = triggers.Add( "bus1", Reg::< U32>::Unknown( U32( 0)));

        assert_eq!( triggers.Len(), 2);
        assert!( !triggers.IsEmpty());
        assert_eq!( triggers.Name( s0), "bus0");
        assert_eq!( triggers.Name( s1), "bus1");

        assert_eq!( *triggers.Get( s0).Val(), U32( 10));
        assert!( triggers.Get( s0).IsValid());
        assert!( triggers.Get( s1).IsX());

        triggers.SetFutureValue( s0, Reg::< U32>::Known( U32( 20)));
        assert!( triggers.IsArmed( s0));

        let  	( past, cur) = triggers.Advance( s0);
        assert_eq!( *past.Val(), U32( 10));
        assert_eq!( *cur.Val(), U32( 20));
        assert!( triggers.IsEdge( s0));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_triggers_gate_methods()
    {
        let  	triggers = {
            let  	mut t = TriggerWad::New();
            t.Add( "f", Reg::FALSE);
            t.Add( "t", Reg::TRUE);
            t.Add( "x", Reg::X);
            t
        };
        let  	idF = U32( 0);
        let  	idT = U32( 1);
        let  	idX = U32( 2);

        // NAND
        assert_eq!( triggers.Nand( idT, idT), Reg::FALSE);
        assert_eq!( triggers.Nand( idT, idF), Reg::TRUE);
        assert_eq!( triggers.Nand( idF, idF), Reg::TRUE);
        assert_eq!( triggers.Nand( idF, idX), Reg::TRUE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_triggers_edges()
    {
        let  	mut triggers = TriggerWad::New();
        // Trigger 0: False -> False
        let  	s0 = triggers.Add( "s0", Reg::FALSE);
        triggers.SetFutureValue( s0, Reg::FALSE);
        triggers.Advance( s0);

        // Trigger 1: False -> True
        let  	s1 = triggers.Add( "s1", Reg::FALSE);
        triggers.SetFutureValue( s1, Reg::TRUE);
        triggers.Advance( s1);

        // Trigger 2: True -> False
        let  	s2 = triggers.Add( "s2", Reg::TRUE);
        triggers.SetFutureValue( s2, Reg::FALSE);
        triggers.Advance( s2);

        assert!( !triggers.IsEdge( s0));

        assert!( triggers.IsEdge( s1));
        assert!( triggers.IsPosedge( s1));
        assert!( !triggers.IsNegedge( s1));

        assert!( triggers.IsEdge( s2));
        assert!( !triggers.IsPosedge( s2));
        assert!( triggers.IsNegedge( s2));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rs_latch_initialization()
    {
        let  	mut layout = Layout::New();
        let  	latch = RSLatch::New( &mut layout, "LatchTest");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        engine.SetPortBool( latch.S(), Reg::TRUE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        let  	qTrig = engine.GetPortTrigger( latch.Q()).unwrap();
        let  	q1Trig = engine.GetPortTrigger( latch.Q1()).unwrap();
        engine.InitSignal( qTrig, RegVal::FALSE);
        engine.InitSignal( q1Trig, RegVal::TRUE);
        engine.Tick();

        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rs_latch_set_and_hold()
    {
        let  	mut layout = Layout::New();
        let  	latch = RSLatch::New( &mut layout, "LatchTest");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // S=0, R=1 -> Set state: Q=1, Q1=0
        engine.SetPortBool( latch.S(), Reg::FALSE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        engine.Tick();
        engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::TRUE), "Q should be 1 after Set");
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::FALSE), "Q1 should be 0 after Set");

        // S=1, R=1 -> Hold state: Q remains 1, Q1 remains 0
        engine.SetPortBool( latch.S(), Reg::TRUE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::TRUE), "Q must hold 1");
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::FALSE), "Q1 must hold 0");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rs_latch_reset_and_hold()
    {
        let  	mut layout = Layout::New();
        let  	latch = RSLatch::New( &mut layout, "LatchTest");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // First Set to 1
        engine.SetPortBool( latch.S(), Reg::FALSE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        engine.Tick();
        engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::TRUE));

        // S=1, R=0 -> Reset state: Q=0, Q1=1
        engine.SetPortBool( latch.S(), Reg::TRUE);
        engine.SetPortBool( latch.R(), Reg::FALSE);
        engine.Tick();
        engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::FALSE), "Q should be 0 after Reset");
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::TRUE), "Q1 should be 1 after Reset");

        // S=1, R=1 -> Hold state: Q remains 0, Q1 remains 1
        engine.SetPortBool( latch.S(), Reg::TRUE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::FALSE), "Q must hold 0");
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::TRUE), "Q1 must hold 1");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_port_layout()
    {
        let  	layout = PortLayout::New( U32( 10));
        assert_eq!( layout.PortCast().SzGroup(), U32( 0));
        assert_eq!( layout.PortConn().SzEdge(), U32( 0));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_port_basic()
    {
        let  	port = TopologyPort::New( "clk", U32( 42));
        assert_eq!( port.Name(), "clk");
        assert_eq!( port.Trigger(), U32( 42));
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    // Synchronous Discrete-Event Dataflow Framework Tests
    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_layout_declaration_and_wiring()
    {
        let  	mut layout = Layout::New();

        let  	modAnd = layout.AddModuleSimple( "And0", &[ "a", "b" ], &[ "out" ], KernelKind::And);
        let  	modNot = layout.AddModuleSimple( "Not0", &[ "in" ], &[ "out" ], KernelKind::Not);

        let  	andOut = layout.OutPort( modAnd, 0).unwrap();
        let  	notIn = layout.InPort( modNot, 0).unwrap();

        assert_eq!( layout.Modules().len(), 2);
        assert_eq!( layout.Ports().len(), 5); // 3 for AND + 2 for NOT

        // Connect AND output to NOT input ( fan-out of 1)
        assert!( layout.Connect( andOut, notIn).is_ok());
        assert_eq!( layout.Connections().len(), 1);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_layout_validation_duplicate_input()
    {
        let  	mut layout = Layout::New();

        let  	modAnd1 = layout.AddModuleSimple( "And1", &[ "a", "b" ], &[ "out" ], KernelKind::And);
        let  	modAnd2 = layout.AddModuleSimple( "And2", &[ "a", "b" ], &[ "out" ], KernelKind::And);
        let  	modNot = layout.AddModuleSimple( "Not", &[ "in" ], &[ "out" ], KernelKind::Not);

        let  	and1Out = layout.OutPort( modAnd1, 0).unwrap();
        let  	and2Out = layout.OutPort( modAnd2, 0).unwrap();
        let  	notIn = layout.InPort( modNot, 0).unwrap();

        // First driver succeeds
        assert!( layout.Connect( and1Out, notIn).is_ok());

        // Second driver on same input MUST fail ( 1-to-1 input assignment rule)
        let  	res = layout.Connect( and2Out, notIn);
        assert!( matches!( res, Err( LayoutError::DuplicateInputDriver { .. })));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_layout_type_mismatch_rejection()
    {
        let  	mut layout = Layout::New();

        let  	modU32 = layout.AddModule(
            "U32Producer",
            &[],
            &[ PortDesc::U32( "data") ],
            KernelKind::Custom( std::sync::Arc::new( |_, _| {})),
        );
        let  	modBool = layout.AddModule(
            "BoolConsumer",
            &[ PortDesc::Bool( "flag") ],
            &[],
            KernelKind::Custom( std::sync::Arc::new( |_, _| {})),
        );

        let  	u32Out = layout.OutPort( modU32, 0).unwrap();
        let  	boolIn = layout.InPort( modBool, 0).unwrap();

        // Connecting U32 to Bool MUST fail with TypeMismatch
        let  	res = layout.Connect( u32Out, boolIn);
        assert!( matches!( res, Err( LayoutError::TypeMismatch { .. })));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_compiled_and_gate_sim_engine()
    {
        let  	mut layout = Layout::New();
        let  	modAnd = layout.AddModuleSimple( "AndGate", &[ "a", "b" ], &[ "out" ], KernelKind::And);

        let  	portA = layout.InPort( modAnd, 0).unwrap();
        let  	portB = layout.InPort( modAnd, 1).unwrap();
        let  	portOut = layout.OutPort( modAnd, 0).unwrap();

        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // Cycle 0: Inputs FALSE, FALSE -> Out evaluates to FALSE
        engine.SetPortBool( portA, Reg::FALSE);
        engine.SetPortBool( portB, Reg::FALSE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( portOut), Some( Reg::FALSE));

        // Cycle 1: Inputs TRUE, FALSE -> Out evaluates to FALSE
        engine.SetPortBool( portA, Reg::TRUE);
        engine.SetPortBool( portB, Reg::FALSE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( portOut), Some( Reg::FALSE));

        // Cycle 2: Inputs TRUE, TRUE -> Out evaluates to TRUE
        engine.SetPortBool( portA, Reg::TRUE);
        engine.SetPortBool( portB, Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( portOut), Some( Reg::TRUE));
        assert_eq!( engine.CycleCount(), 3);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_compiled_half_adder_sim_engine()
    {
        let  	mut layout = Layout::New();

        let  	modXor = layout.AddModuleSimple( "Xor", &[ "a", "b" ], &[ "sum" ], KernelKind::Xor);
        let  	modAnd = layout.AddModuleSimple( "And", &[ "a", "b" ], &[ "carry" ], KernelKind::And);

        let  	xorA = layout.InPort( modXor, 0).unwrap();
        let  	xorB = layout.InPort( modXor, 1).unwrap();
        let  	sumOut = layout.OutPort( modXor, 0).unwrap();

        let  	andA = layout.InPort( modAnd, 0).unwrap();
        let  	andB = layout.InPort( modAnd, 1).unwrap();
        let  	carryOut = layout.OutPort( modAnd, 0).unwrap();

        let  	mut engine = layout.Compile().expect( "Compilation failed");

        let  	truthTable = [
            ( false, false, false, false), // 0+0 = sum 0, carry 0
            ( false, true, true, false),   // 0+1 = sum 1, carry 0
            ( true, false, true, false),   // 1+0 = sum 1, carry 0
            ( true, true, false, true),    // 1+1 = sum 0, carry 1
        ];

        for ( a, b, expSum, expCarry) in truthTable {
            engine.SetPortBool( xorA, Reg::FromBool( a));
            engine.SetPortBool( xorB, Reg::FromBool( b));
            engine.SetPortBool( andA, Reg::FromBool( a));
            engine.SetPortBool( andB, Reg::FromBool( b));

            engine.Tick();

            assert_eq!(
                engine.GetPortBool( sumOut),
                Some( Reg::FromBool( expSum)),
                "Half adder sum mismatch for A={a}, B={b}"
            );
            assert_eq!(
                engine.GetPortBool( carryOut),
                Some( Reg::FromBool( expCarry)),
                "Half adder carry mismatch for A={a}, B={b}"
            );
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_compiled_sr_latch_sequential_clocking()
    {
        let  	mut layout = Layout::New();

        // RS Latch built of two cross-coupled NAND gates:
        // nand1( S, Q1) -> Q
        // nand2( R, Q) -> Q1
        let  	nand1 = layout.AddModuleSimple( "Nand1", &[ "s", "q1" ], &[ "q" ], KernelKind::Nand);
        let  	nand2 = layout.AddModuleSimple( "Nand2", &[ "r", "q" ], &[ "q1" ], KernelKind::Nand);

        let  	sPort = layout.InPort( nand1, 0).unwrap();
        let  	q1InNand1 = layout.InPort( nand1, 1).unwrap();
        let  	qPort = layout.OutPort( nand1, 0).unwrap();

        let  	rPort = layout.InPort( nand2, 0).unwrap();
        let  	qInNand2 = layout.InPort( nand2, 1).unwrap();
        let  	q1Port = layout.OutPort( nand2, 0).unwrap();

        // Cross-couple wiring ( Output -> Input)
        assert!( layout.Connect( qPort, qInNand2).is_ok());
        assert!( layout.Connect( q1Port, q1InNand1).is_ok());

        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // Initialize stable state: S=1, R=1, Q=0, Q1=1
        engine.SetPortBool( sPort, Reg::TRUE);
        engine.SetPortBool( rPort, Reg::TRUE);
        let  	qTrig = engine.GetPortTrigger( qPort).unwrap();
        let  	q1Trig = engine.GetPortTrigger( q1Port).unwrap();
        engine.InitSignal( qTrig, RegVal::FALSE);
        engine.InitSignal( q1Trig, RegVal::TRUE);

        // Step 1: Hold ( S=1, R=1) -> remains Q=0, Q1=1
        engine.Tick();
        assert_eq!( engine.GetPortBool( qPort), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( q1Port), Some( Reg::TRUE));

        // Step 2: Set ( S=0, R=1) -> Q transitions to 1
        engine.SetPortBool( sPort, Reg::FALSE);
        engine.SetPortBool( rPort, Reg::TRUE);
        engine.Tick();
        engine.Tick(); // Settle cross-coupled feedback
        assert_eq!( engine.GetPortBool( qPort), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( q1Port), Some( Reg::FALSE));

        // Step 3: Hold ( S=1, R=1) -> Q holds 1
        engine.SetPortBool( sPort, Reg::TRUE);
        engine.SetPortBool( rPort, Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( qPort), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( q1Port), Some( Reg::FALSE));

        // Step 4: Reset ( S=1, R=0) -> Q transitions to 0
        engine.SetPortBool( sPort, Reg::TRUE);
        engine.SetPortBool( rPort, Reg::FALSE);
        engine.Tick();
        engine.Tick(); // Settle cross-coupled feedback
        assert_eq!( engine.GetPortBool( qPort), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( q1Port), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_compiled_heterogeneous_u32_bus_simulation()
    {
        let  	mut layout = Layout::New();

        // 32-bit ALU adder with Enable signal:
        // Inputs: [ a: U32, b: U32, enable: Bool ]
        // Outputs: [ sum: U32, overflow: Bool ]
        let  	aluKernel = std::sync::Arc::new( |inVals: &[RegVal], outVals: &mut [RegVal]| {
            let  	aVal = inVals[0].Val();
            let  	bVal = inVals[1].Val();
            let  	en = inVals[2].IsTrue();

            if en {
                let  	sum = aVal.wrapping_add( bVal) & 0xFFFF_FFFF;
                let  	overflow = ( aVal + bVal) > 0xFFFF_FFFF;
                outVals[0] = RegVal::FromU32( U32( sum as u32));
                outVals[1] = RegVal::FromBool( overflow);
            } else {
                outVals[0] = RegVal::FromU32( U32( 0));
                outVals[1] = RegVal::FALSE;
            }
        });

        let  	aluMod = layout.AddModule(
            "Alu32",
            &[ PortDesc::U32( "a"), PortDesc::U32( "b"), PortDesc::Bool( "en") ],
            &[ PortDesc::U32( "sum"), PortDesc::Bool( "overflow") ],
            KernelKind::Custom( aluKernel),
        );

        let  	portA = layout.InPort( aluMod, 0).unwrap();
        let  	portB = layout.InPort( aluMod, 1).unwrap();
        let  	portEn = layout.InPort( aluMod, 2).unwrap();
        let  	portSum = layout.OutPort( aluMod, 0).unwrap();
        let  	portOverflow = layout.OutPort( aluMod, 1).unwrap();

        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // Cycle 1: en=false -> sum=0, overflow=false
        engine.SetPortU32( portA, Reg::Known( U32( 100)));
        engine.SetPortU32( portB, Reg::Known( U32( 200)));
        engine.SetPortBool( portEn, Reg::FALSE);
        engine.Tick();
        assert_eq!( engine.GetPortU32( portSum), Some( Reg::Known( U32( 0))));
        assert_eq!( engine.GetPortBool( portOverflow), Some( Reg::FALSE));

        // Cycle 2: en=true -> sum=300, overflow=false
        engine.SetPortBool( portEn, Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortU32( portSum), Some( Reg::Known( U32( 300))));
        assert_eq!( engine.GetPortBool( portOverflow), Some( Reg::FALSE));

        // Cycle 3: 0xFFFF_FFFF + 1 -> sum=0, overflow=true
        engine.SetPortU32( portA, Reg::Known( U32( 0xFFFF_FFFF)));
        engine.SetPortU32( portB, Reg::Known( U32( 1)));
        engine.Tick();
        assert_eq!( engine.GetPortU32( portSum), Some( Reg::Known( U32( 0))));
        assert_eq!( engine.GetPortBool( portOverflow), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_bus_adder_32_simulation()
    {
        let  	mut layout = Layout::New();
        let  	busAdder = BusAdder32::New( &mut layout, "Adder32");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        engine.SetPortU32( busAdder._A, Reg::Known( U32( 12345)));
        engine.SetPortU32( busAdder._B, Reg::Known( U32( 54321)));
        engine.Tick();

        assert_eq!( engine.GetPortU32( busAdder._Sum), Some( Reg::Known( U32( 66666))));
        assert_eq!( engine.GetPortBool( busAdder._Carry), Some( Reg::FALSE));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------