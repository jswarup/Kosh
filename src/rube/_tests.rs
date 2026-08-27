//-- _tests.rs --------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests
{
    use	std::sync::Arc;
    use	crate::{
        rube::{
            adder::{ Adder, BusAdder32 },
            gates::{ AndGate, NandGate, NotGate, OrGate, XorGate },
            latches::{ CRSLatch, DLatch, RSLatch },
            layout::{ Layout, LayoutError },
            module::KernelKind,
            port::{ PortDesc, PortId, PortType, TopologyPort },
            reg::Reg,
            sim_context::{ ActionKind, SimContext, SimError },
            trigger::{ TriggerSense, TriggerWad },
        },
        silo::{ IEdgeConnect, U32 },
    };

    //-----------------------------------------------------------------------------------------------------------------------------

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
            ( 0xFFFF, 1, 0, true),
            ( 0x8000, 0x8000, 0, true),
            ( 12345, 54321, 66666, true),
        ];

        for ( aVal, bVal, expectedSum, expectedCarry) in testCases {
            adder.SetA( &mut engine, U32( aVal));
            adder.SetB( &mut engine, U32( bVal));
            for _ in 0..( N * 3) { engine.Tick(); }

            let  	sumVal = adder.GetSum( &engine);
            let  	carryVal = engine.GetPortBool( adder.Carry()).unwrap().IsTrue();
            let  	expTrunc = expectedSum & 0xFFFF;
            let  	expOverflow = expectedSum > 0xFFFF;

            assert_eq!( sumVal, expTrunc);
            assert_eq!( carryVal, expectedCarry || expOverflow);
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_clocked_rs_latch()
    {
        let  	mut layout = Layout::New();
        let  	crs = CRSLatch::New( &mut layout, "CRSLatch");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // clk=0, S=1, R=0 -> no effect
        crs.SetClk( &mut engine, Reg::FALSE);
        crs.SetS( &mut engine, Reg::TRUE);
        crs.SetR( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::FALSE));

        // Pulse clk=1 -> Q becomes 1, Q1 becomes 0
        crs.SetClk( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( crs.Q1()), Some( Reg::FALSE));

        // clk=0 -> holds Q=1
        crs.SetClk( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));

        // S=0, R=1, clk=0 -> still holds Q=1
        crs.SetS( &mut engine, Reg::FALSE);
        crs.SetR( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));

        // Pulse clk=1 -> Q resets to 0, Q1 to 1
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

        // Enable=0: latched
        dLatch.SetE( &mut engine, Reg::FALSE);
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::FALSE));

        // Enable=1: transparent ( D=1 -> Q=1)
        dLatch.SetE( &mut engine, Reg::TRUE);
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( dLatch.Q1()), Some( Reg::FALSE));

        // D=0 -> Q=0
        dLatch.SetD( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( dLatch.Q1()), Some( Reg::TRUE));

        // Set D=1, latch with E=0, change D=0 -> Q stays 1
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Tick(); }
        dLatch.SetE( &mut engine, Reg::FALSE);
        dLatch.SetD( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Tick(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    fn	testGate2< G>(
        make: impl FnOnce( &mut Layout, &str) -> G,
        in1: impl Fn( &G) -> PortId,
        in2: impl Fn( &G) -> PortId,
        out: impl Fn( &G) -> PortId,
        table: &[( bool, bool, Reg)],
    )
    {
        let  	mut layout = Layout::New();
        let  	gate = make( &mut layout, "Gate");
        let  	mut engine = layout.Compile().expect( "Compilation failed");
        for &( a, b, exp) in table {
            engine.SetPortBool( in1( &gate), Reg::FromBool( a));
            engine.SetPortBool( in2( &gate), Reg::FromBool( b));
            engine.Tick();
            assert_eq!( engine.GetPortBool( out( &gate)), Some( exp));
        }
    }

    #[test]
    fn	test_nand_gate()
    {
        testGate2(
            NandGate::New, |g| g.In1(), |g| g.In2(), |g| g.Out(),
            &[ ( false, false, Reg::TRUE), ( false, true, Reg::TRUE), ( true, false, Reg::TRUE), ( true, true, Reg::FALSE) ],
        );
    }

    #[test]
    fn	test_and_gate()
    {
        testGate2(
            AndGate::New, |g| g.In1(), |g| g.In2(), |g| g.Out(),
            &[ ( false, false, Reg::FALSE), ( false, true, Reg::FALSE), ( true, false, Reg::FALSE), ( true, true, Reg::TRUE) ],
        );
    }

    #[test]
    fn	test_or_gate()
    {
        testGate2(
            OrGate::New, |g| g.In1(), |g| g.In2(), |g| g.Out(),
            &[ ( false, false, Reg::FALSE), ( false, true, Reg::TRUE), ( true, false, Reg::TRUE), ( true, true, Reg::TRUE) ],
        );
    }

    #[test]
    fn	test_xor_gate()
    {
        testGate2(
            XorGate::New, |g| g.In1(), |g| g.In2(), |g| g.Out(),
            &[ ( false, false, Reg::FALSE), ( false, true, Reg::TRUE), ( true, false, Reg::TRUE), ( true, true, Reg::FALSE) ],
        );
    }

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
    fn	test_reg_bitwise_operations()
    {
        assert_eq!( ( !Reg::FALSE).AsBool(), Reg::TRUE);
        assert_eq!( ( !Reg::TRUE).AsBool(), Reg::FALSE);
        assert_eq!( ( !Reg::X).AsBool(), Reg::X);

        assert_eq!( Reg::TRUE & Reg::TRUE, Reg::TRUE);
        assert_eq!( Reg::TRUE & Reg::FALSE, Reg::FALSE);
        assert_eq!( Reg::FALSE & Reg::X, Reg::FALSE);
        assert_eq!( Reg::TRUE & Reg::X, Reg::X);

        assert_eq!( Reg::FALSE | Reg::FALSE, Reg::FALSE);
        assert_eq!( Reg::TRUE | Reg::FALSE, Reg::TRUE);
        assert_eq!( Reg::TRUE | Reg::X, Reg::TRUE);
        assert_eq!( Reg::FALSE | Reg::X, Reg::X);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_generic_reg()
    {
        let  	knownVal = Reg::Known( 42);
        assert!( knownVal.IsValid() && !knownVal.IsX());
        assert_eq!( knownVal.Val(), 42);
        assert_eq!( knownVal.GetU32(), U32( 42));

        let  	unknownVal = Reg::Unknown( 0xFF);
        assert!( !unknownVal.IsValid() && unknownVal.IsX());

        let  	defaultReg = Reg::default();
        assert!( !defaultReg.IsValid() && defaultReg.IsX());

        let  	mut reg100 = Reg::FromU32( U32( 100));
        assert_eq!( reg100.Val(), 100);
        assert!( reg100.IsValid());
        assert_eq!( reg100.GetU32(), U32( 100));

        reg100._X = 1;
        assert!( reg100.IsX());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_trigger_wad_basic()
    {
        let  	mut triggers = TriggerWad::New();
        let  	s0 = triggers.AddTyped( "bus0", PortType::U32Val, Reg::FromU32( U32( 10)));
        let  	s1 = triggers.AddTyped( "bus1", PortType::U32Val, Reg::X_U32);

        assert_eq!( triggers.Len(), 2);
        assert!( !triggers.IsEmpty());
        assert_eq!( triggers.Name( s0), "bus0");
        assert_eq!( triggers.Name( s1), "bus1");
        assert_eq!( triggers.PortType( s0), PortType::U32Val);

        assert_eq!( triggers.Get( s0).GetU32(), U32( 10));
        assert!( triggers.Get( s0).IsValid());
        assert!( triggers.Get( s1).IsX());

        triggers.SetFutureValue( s0, Reg::FromU32( U32( 20)));
        assert!( triggers.IsArmed( s0));

        let  	( past, cur) = triggers.Advance( s0);
        assert_eq!( past.GetU32(), U32( 10));
        assert_eq!( cur.GetU32(), U32( 20));
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
            t.Add( "x", Reg::X_BOOL);
            t
        };
        let  	idF = U32( 0);
        let  	idT = U32( 1);
        let  	idX = U32( 2);

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
        let  	s0 = triggers.Add( "s0", Reg::FALSE);
        triggers.SetFutureValue( s0, Reg::FALSE);
        triggers.Advance( s0);

        let  	s1 = triggers.Add( "s1", Reg::FALSE);
        triggers.SetFutureValue( s1, Reg::TRUE);
        triggers.Advance( s1);

        let  	s2 = triggers.Add( "s2", Reg::TRUE);
        triggers.SetFutureValue( s2, Reg::FALSE);
        triggers.Advance( s2);

        assert!( !triggers.IsEdge( s0));
        assert!( triggers.IsEdge( s1) && triggers.IsPosedge( s1) && !triggers.IsNegedge( s1));
        assert!( triggers.IsEdge( s2) && !triggers.IsPosedge( s2) && triggers.IsNegedge( s2));
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
        engine.InitTrigger( qTrig, Reg::FALSE);
        engine.InitTrigger( q1Trig, Reg::TRUE);
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

        // Set: S=0, R=1 -> Q=1, Q1=0
        engine.SetPortBool( latch.S(), Reg::FALSE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        engine.Tick(); engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::FALSE));

        // Hold: S=1, R=1 -> Q=1, Q1=0
        engine.SetPortBool( latch.S(), Reg::TRUE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::FALSE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rs_latch_reset_and_hold()
    {
        let  	mut layout = Layout::New();
        let  	latch = RSLatch::New( &mut layout, "LatchTest");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // Set to 1
        engine.SetPortBool( latch.S(), Reg::FALSE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        engine.Tick(); engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::TRUE));

        // Reset: S=1, R=0 -> Q=0, Q1=1
        engine.SetPortBool( latch.S(), Reg::TRUE);
        engine.SetPortBool( latch.R(), Reg::FALSE);
        engine.Tick(); engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::TRUE));

        // Hold: S=1, R=1 -> Q=0, Q1=1
        engine.SetPortBool( latch.S(), Reg::TRUE);
        engine.SetPortBool( latch.R(), Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( latch.Q()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( latch.Q1()), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_multibit_fast_module_u32()
    {
        let  	mut layout = Layout::New();
        let  	andMod = layout.AddModule(
            "And32",
            &[ PortDesc::U32( "a"), PortDesc::U32( "b") ],
            &[ PortDesc::U32( "out") ],
            KernelKind::And,
        );

        let  	portA = layout.InPort( andMod, 0).unwrap();
        let  	portB = layout.InPort( andMod, 1).unwrap();
        let  	portOut = layout.OutPort( andMod, 0).unwrap();
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        engine.SetPortU32( portA, Reg::Known( 0x1234_5678));
        engine.SetPortU32( portB, Reg::Known( 0x00FF_00FF));
        engine.Tick();

        assert_eq!( engine.GetPortU32( portOut), Some( Reg::Known( 0x0034_0078)));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_native_word_arithmetic()
    {
        let  	mut layout = Layout::New();
        let  	addMod = layout.AddModule(
            "Add32",
            &[ PortDesc::U32( "a"), PortDesc::U32( "b") ],
            &[ PortDesc::U32( "sum") ],
            KernelKind::Add,
        );
        let  	subMod = layout.AddModule(
            "Sub32",
            &[ PortDesc::U32( "a"), PortDesc::U32( "b") ],
            &[ PortDesc::U32( "diff") ],
            KernelKind::Sub,
        );

        let  	addA = layout.InPort( addMod, 0).unwrap();
        let  	addB = layout.InPort( addMod, 1).unwrap();
        let  	addOut = layout.OutPort( addMod, 0).unwrap();

        let  	subA = layout.InPort( subMod, 0).unwrap();
        let  	subB = layout.InPort( subMod, 1).unwrap();
        let  	subOut = layout.OutPort( subMod, 0).unwrap();

        let  	mut engine = layout.Compile().expect( "Compilation failed");

        engine.SetPortU32( addA, Reg::Known( 1000));
        engine.SetPortU32( addB, Reg::Known( 250));
        engine.SetPortU32( subA, Reg::Known( 1000));
        engine.SetPortU32( subB, Reg::Known( 250));
        engine.Tick();

        assert_eq!( engine.GetPortU32( addOut), Some( Reg::Known( 1250)));
        assert_eq!( engine.GetPortU32( subOut), Some( Reg::Known( 750)));
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

    #[test]
    fn	test_layout_declaration_and_wiring()
    {
        let  	mut layout = Layout::New();
        let  	modAnd = layout.AddModuleSimple( "And0", &[ "a", "b" ], &[ "out" ], KernelKind::And);
        let  	modNot = layout.AddModuleSimple( "Not0", &[ "in" ], &[ "out" ], KernelKind::Not);

        let  	andOut = layout.OutPort( modAnd, 0).unwrap();
        let  	notIn = layout.InPort( modNot, 0).unwrap();

        assert_eq!( layout.Modules().len(), 2);
        assert_eq!( layout.Ports().len(), 5);
        assert!( layout.Connect( andOut, notIn).is_ok());
        assert_eq!( layout.Connections().SzEdge(), U32( 1));

        let  	mut dot = String::new();
        layout.DumpDot( &mut dot);
        assert!( dot.contains( "digraph"));
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

        assert!( layout.Connect( and1Out, notIn).is_ok());
        assert!( matches!( layout.Connect( and2Out, notIn), Err( LayoutError::DuplicateInputDriver { .. })));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_layout_type_mismatch_rejection()
    {
        let  	mut layout = Layout::New();
        let  	modU32 = layout.AddModule( "U32Producer", &[], &[ PortDesc::U32( "data") ], KernelKind::Custom( Arc::new( |_, _| {})));
        let  	modBool = layout.AddModule( "BoolConsumer", &[ PortDesc::Bool( "flag") ], &[], KernelKind::Custom( Arc::new( |_, _| {})));

        let  	u32Out = layout.OutPort( modU32, 0).unwrap();
        let  	boolIn = layout.InPort( modBool, 0).unwrap();

        assert!( matches!( layout.Connect( u32Out, boolIn), Err( LayoutError::TypeMismatch { .. })));
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

        for ( a, b, exp) in [ ( Reg::FALSE, Reg::FALSE, Reg::FALSE), ( Reg::TRUE, Reg::FALSE, Reg::FALSE), ( Reg::TRUE, Reg::TRUE, Reg::TRUE) ] {
            engine.SetPortBool( portA, a);
            engine.SetPortBool( portB, b);
            engine.Tick();
            assert_eq!( engine.GetPortBool( portOut), Some( exp));
        }
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
            ( false, false, false, false),
            ( false, true, true, false),
            ( true, false, true, false),
            ( true, true, false, true),
        ];

        for ( a, b, expSum, expCarry) in truthTable {
            engine.SetPortBool( xorA, Reg::FromBool( a));
            engine.SetPortBool( xorB, Reg::FromBool( b));
            engine.SetPortBool( andA, Reg::FromBool( a));
            engine.SetPortBool( andB, Reg::FromBool( b));
            engine.Tick();
            assert_eq!( engine.GetPortBool( sumOut), Some( Reg::FromBool( expSum)));
            assert_eq!( engine.GetPortBool( carryOut), Some( Reg::FromBool( expCarry)));
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_compiled_sr_latch_sequential_clocking()
    {
        let  	mut layout = Layout::New();
        let  	nand1 = layout.AddModuleSimple( "Nand1", &[ "s", "q1" ], &[ "q" ], KernelKind::Nand);
        let  	nand2 = layout.AddModuleSimple( "Nand2", &[ "r", "q" ], &[ "q1" ], KernelKind::Nand);

        let  	sPort = layout.InPort( nand1, 0).unwrap();
        let  	q1InNand1 = layout.InPort( nand1, 1).unwrap();
        let  	qPort = layout.OutPort( nand1, 0).unwrap();

        let  	rPort = layout.InPort( nand2, 0).unwrap();
        let  	qInNand2 = layout.InPort( nand2, 1).unwrap();
        let  	q1Port = layout.OutPort( nand2, 0).unwrap();

        assert!( layout.Connect( qPort, qInNand2).is_ok());
        assert!( layout.Connect( q1Port, q1InNand1).is_ok());

        let  	mut engine = layout.Compile().expect( "Compilation failed");

        // S=1, R=1, Init Q=0, Q1=1
        engine.SetPortBool( sPort, Reg::TRUE);
        engine.SetPortBool( rPort, Reg::TRUE);
        let  	qTrig = engine.GetPortTrigger( qPort).unwrap();
        let  	q1Trig = engine.GetPortTrigger( q1Port).unwrap();
        engine.InitTrigger( qTrig, Reg::FALSE);
        engine.InitTrigger( q1Trig, Reg::TRUE);

        // Hold -> Q=0, Q1=1
        engine.Tick();
        assert_eq!( engine.GetPortBool( qPort), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( q1Port), Some( Reg::TRUE));

        // Set ( S=0, R=1) -> Q=1, Q1=0
        engine.SetPortBool( sPort, Reg::FALSE);
        engine.SetPortBool( rPort, Reg::TRUE);
        engine.Tick(); engine.Tick();
        assert_eq!( engine.GetPortBool( qPort), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( q1Port), Some( Reg::FALSE));

        // Hold ( S=1, R=1) -> Q=1, Q1=0
        engine.SetPortBool( sPort, Reg::TRUE);
        engine.SetPortBool( rPort, Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortBool( qPort), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( q1Port), Some( Reg::FALSE));

        // Reset ( S=1, R=0) -> Q=0, Q1=1
        engine.SetPortBool( sPort, Reg::TRUE);
        engine.SetPortBool( rPort, Reg::FALSE);
        engine.Tick(); engine.Tick();
        assert_eq!( engine.GetPortBool( qPort), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( q1Port), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_compiled_heterogeneous_u32_bus_simulation()
    {
        let  	mut layout = Layout::New();
        let  	aluKernel = Arc::new( |inVals: &[Reg], outVals: &mut [Reg]| {
            let  	aVal = inVals[0].Val();
            let  	bVal = inVals[1].Val();
            let  	en = inVals[2].IsTrue();

            if en {
                let  	sum = aVal.wrapping_add( bVal) & 0xFFFF_FFFF;
                let  	overflow = ( aVal + bVal) > 0xFFFF_FFFF;
                outVals[0] = Reg::FromU32( U32( sum as u32));
                outVals[1] = Reg::FromBool( overflow);
            } else {
                outVals[0] = Reg::FromU32( U32( 0));
                outVals[1] = Reg::FALSE;
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

        // en=false -> sum=0, overflow=false
        engine.SetPortU32( portA, Reg::Known( 100));
        engine.SetPortU32( portB, Reg::Known( 200));
        engine.SetPortBool( portEn, Reg::FALSE);
        engine.Tick();
        assert_eq!( engine.GetPortU32( portSum), Some( Reg::Known( 0)));
        assert_eq!( engine.GetPortBool( portOverflow), Some( Reg::FALSE));

        // en=true -> sum=300, overflow=false
        engine.SetPortBool( portEn, Reg::TRUE);
        engine.Tick();
        assert_eq!( engine.GetPortU32( portSum), Some( Reg::Known( 300)));
        assert_eq!( engine.GetPortBool( portOverflow), Some( Reg::FALSE));

        // 0xFFFF_FFFF + 1 -> sum=0, overflow=true
        engine.SetPortU32( portA, Reg::Known( 0xFFFF_FFFF));
        engine.SetPortU32( portB, Reg::Known( 1));
        engine.Tick();
        assert_eq!( engine.GetPortU32( portSum), Some( Reg::Known( 0)));
        assert_eq!( engine.GetPortBool( portOverflow), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_bus_adder_32_simulation()
    {
        let  	mut layout = Layout::New();
        let  	busAdder = BusAdder32::New( &mut layout, "Adder32");
        let  	mut engine = layout.Compile().expect( "Compilation failed");

        engine.SetPortU32( busAdder._A, Reg::Known( 12345));
        engine.SetPortU32( busAdder._B, Reg::Known( 54321));
        engine.Tick();

        assert_eq!( engine.GetPortU32( busAdder._Sum), Some( Reg::Known( 66666)));
        assert_eq!( engine.GetPortBool( busAdder._Carry), Some( Reg::FALSE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_sim_context_delta_cycles()
    {
        let  	mut ctx = SimContext::New();

        let  	in0 = ctx.AddTrigger( "in0", Reg::FALSE);
        let  	in1 = ctx.AddTrigger( "in1", Reg::TRUE);
        let  	in2 = ctx.AddTrigger( "in2", Reg::FALSE);

        ctx.AddAction( ActionKind::Not { _In: in0, _Out: in1 }, &[ ( in0, TriggerSense::EDGE) ]);
        ctx.AddAction( ActionKind::Not { _In: in1, _Out: in2 }, &[ ( in1, TriggerSense::EDGE) ]);

        ctx.SetValue( in0, Reg::TRUE);

        let  	cycles = ctx.Drive().expect( "Drive failed");
        assert_eq!( cycles, 3);
        assert_eq!( ctx.GetValue( in0), Reg::TRUE);
        assert_eq!( ctx.GetValue( in1), Reg::FALSE);
        assert_eq!( ctx.GetValue( in2), Reg::TRUE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_sim_context_fanout_inverted_index()
    {
        let  	mut ctx = SimContext::New();

        let  	clk = ctx.AddTrigger( "clk", Reg::FALSE);
        let  	d = ctx.AddTrigger( "d", Reg::FALSE);
        let  	q1 = ctx.AddTrigger( "q1", Reg::FALSE);
        let  	q2 = ctx.AddTrigger( "q2", Reg::FALSE);

        ctx.AddAction( ActionKind::And { _In1: clk, _In2: d, _Out: q1 }, &[ ( clk, TriggerSense::POS_EDGE) ]);
        ctx.AddAction( ActionKind::Or { _In1: clk, _In2: d, _Out: q2 }, &[ ( clk, TriggerSense::POS_EDGE) ]);

        ctx.InitValue( d, Reg::TRUE);
        ctx.SetValue( clk, Reg::TRUE);

        let  	cycles = ctx.Drive().expect( "Drive failed");
        assert_eq!( cycles, 2);
        assert_eq!( ctx.GetValue( q1), Reg::TRUE);
        assert_eq!( ctx.GetValue( q2), Reg::TRUE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_sim_context_oscillation_detection()
    {
        let  	mut ctx = SimContext::New();

        let  	in0 = ctx.AddTrigger( "ring0", Reg::FALSE);

        // Ring oscillator feedback loop: in0 -> not(in0) -> in0 -> ...
        ctx.AddAction( ActionKind::Not { _In: in0, _Out: in0 }, &[ ( in0, TriggerSense::EDGE) ]);

        ctx.SetValue( in0, Reg::TRUE);

        let  	res = ctx.Drive();
        assert_eq!( res, Err( SimError::Oscillation));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------