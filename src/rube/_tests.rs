//-- _tests.rs --------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests
{
    use	crate::{
        rube::{
            adder::{ Adder, AdderPipeline, BusAdder32, FullAdder, HalfAdder },
            coro_kernel::CoroPorts,
            engine::{ SimEngine, SimEngineMode },
            gates::{ AndGate, NandGate, NotGate, OrGate, XorGate },
            latches::{ CRSLatch, DLatch },
            layout::Layout,
            module::{ IModule, KernelKind, ModuleId },
            netlist::INetlist,
            port::{ IPort, PortDesc, PortId },
            reg::Reg,
            vcd::VcdWriter,
        },
        silo::{ ConsoleTest, Stash, U8, U32, USeg },
        stalks::Coro,
    };

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_adder_basic()
    {
        let  	mut layout = Layout::New();
        const N: usize = 16;
        let  	adder = Adder::< N>::New( &mut layout, "Adder16", None);
        layout.Freeze().expect( "Compilation failed");
        let  	mut engine = SimEngine::Create(&layout);

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
            for _ in 0..( N * 3) { engine.Drive(); }

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
        let  	crs = CRSLatch::New( &mut layout, "CRSLatch", None);
        layout.Freeze().expect( "Compilation failed");
        let  	mut engine = SimEngine::Create(&layout);

        // clk=0, S=1, R=0 -> no effect
        crs.SetClk( &mut engine, Reg::FALSE);
        crs.SetS( &mut engine, Reg::TRUE);
        crs.SetR( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Drive(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::FALSE));

        // Pulse clk=1 -> Q becomes 1, Q1 becomes 0
        crs.SetClk( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Drive(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( crs.Q1()), Some( Reg::FALSE));

        // clk=0 -> holds Q=1
        crs.SetClk( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Drive(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));

        // S=0, R=1, clk=0 -> still holds Q=1
        crs.SetS( &mut engine, Reg::FALSE);
        crs.SetR( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Drive(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::TRUE));

        // Pulse clk=1 -> Q resets to 0, Q1 to 1
        crs.SetClk( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Drive(); }
        assert_eq!( engine.GetPortBool( crs.Q()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( crs.Q1()), Some( Reg::TRUE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_d_latch()
    {
        let  	mut layout = Layout::New();
        let  	dLatch = DLatch::New( &mut layout, "DLatch", None);
        layout.Freeze().expect( "Compilation failed");
        let  	mut engine = SimEngine::Create(&layout);

        // Enable=0: latched
        dLatch.SetE( &mut engine, Reg::FALSE);
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Drive(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::FALSE));

        // Enable=1: transparent ( D=1 -> Q=1)
        dLatch.SetE( &mut engine, Reg::TRUE);
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Drive(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( dLatch.Q1()), Some( Reg::FALSE));

        // D=0 -> Q=0
        dLatch.SetD( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Drive(); }
        assert_eq!( engine.GetPortBool( dLatch.Q()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( dLatch.Q1()), Some( Reg::TRUE));

        // Set D=1, latch with E=0, change D=0 -> Q stays 1
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 { engine.Drive(); }
        dLatch.SetE( &mut engine, Reg::FALSE);
        dLatch.SetD( &mut engine, Reg::FALSE);
        for _ in 0..4 { engine.Drive(); }
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
        layout.Freeze().expect( "Compilation failed");
        let  	mut engine = SimEngine::Create(&layout);
        for &( a, b, exp) in table {
            engine.SetPortBool( in1( &gate), Reg::FromBool( a));
            engine.SetPortBool( in2( &gate), Reg::FromBool( b));
            engine.Drive();
            assert_eq!( engine.GetPortBool( out( &gate)), Some( exp));
        }
    }

    #[test]
    fn	test_nand_gate()
    {
        testGate2(
            |layout, name| NandGate::New(layout, name, None), |g| g.In1(), |g| g.In2(), |g| g.Out(),
            &[ ( false, false, Reg::TRUE), ( false, true, Reg::TRUE), ( true, false, Reg::TRUE), ( true, true, Reg::FALSE) ],
        );
    }

    #[test]
    fn	test_and_gate()
    {
        testGate2(
            |layout, name| AndGate::New(layout, name, None), |g| g.In1(), |g| g.In2(), |g| g.Out(),
            &[ ( false, false, Reg::FALSE), ( false, true, Reg::FALSE), ( true, false, Reg::FALSE), ( true, true, Reg::TRUE) ],
        );
    }

    #[test]
    fn	test_or_gate()
    {
        testGate2(
            |layout, name| OrGate::New(layout, name, None), |g| g.In1(), |g| g.In2(), |g| g.Out(),
            &[ ( false, false, Reg::FALSE), ( false, true, Reg::TRUE), ( true, false, Reg::TRUE), ( true, true, Reg::TRUE) ],
        );
    }

    #[test]
    fn	test_xor_gate()
    {
        testGate2(
            |layout, name| XorGate::New(layout, name, None), |g| g.In1(), |g| g.In2(), |g| g.Out(),
            &[ ( false, false, Reg::FALSE), ( false, true, Reg::TRUE), ( true, false, Reg::TRUE), ( true, true, Reg::FALSE) ],
        );
    }

    #[test]
    fn	test_not_gate()
    {
        let  	mut layout = Layout::New();
        let  	gate = NotGate::New( &mut layout, "Not", None);
        layout.Freeze().expect( "Compilation failed");
        let  	mut engine = SimEngine::Create(&layout);

        engine.SetPortBool( gate.In(), Reg::FALSE);
        engine.Drive();
        assert_eq!( engine.GetPortBool( gate.Out()), Some( Reg::TRUE));

        engine.SetPortBool( gate.In(), Reg::TRUE);
        engine.Drive();
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
    fn	test_vcd_writer()
    {
        use crate::rube::VcdWriter;

        let  	mut layout = Layout::New();
        let  	dLatch = DLatch::New( &mut layout, "DLatch", None);
        layout.Freeze().expect( "Compilation failed");
        let  	mut engine = SimEngine::Create(&layout);

        let  	vcdWriter = VcdWriter::New( &layout, &engine);
        let  	mut vcdStr = String::new();

        vcdWriter.WriteHeader( &layout, &engine, &mut vcdStr);

        dLatch.SetE( &mut engine, Reg::FALSE);
        dLatch.SetD( &mut engine, Reg::TRUE);
        for _ in 0..4 {
            engine.Drive();
            vcdWriter.DumpCycle( &engine, &mut vcdStr);
        }

        assert!( vcdStr.contains( "$timescale 1ns $end"));
        assert!( vcdStr.contains( "$scope module DLatch.Inv $end"));
        assert!( vcdStr.contains( "$var wire 1"));
        assert!( vcdStr.contains( "$dumpvars"));
        assert!( vcdStr.contains( "#1"));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_vcd_parser()
    {
        use crate::rube::vcdio::ParseVcd;

        let  	vcdContent = r#"
$version
   Kosh Rube Engine
$end
$timescale 1ns $end
$date 2026-09-01 $end
$scope module top $end
$var wire 1 ! sig1 $end
$var wire 4 " sig2 $end
$upscope $end
$enddefinitions $end
$dumpvars
0!
b1010 "
$end
#1
1!
#2
0!
b1100 "
"#;

        let  	model = ParseVcd( vcdContent).expect( "Failed to parse VCD");

        assert_eq!( model._Version.trim(), "Kosh Rube Engine");
        assert_eq!( model._Timescale.trim(), "1ns");
        assert_eq!( model._Date.trim(), "2026-09-01");

        assert_eq!( model._Scopes.Size().0, 1);
        assert_eq!( model._Scopes[U32( 0)]._Type, "module");
        assert_eq!( model._Scopes[U32( 0)]._Name, "top");

        assert_eq!( model._TimeSteps.Size().0, 3);

        let  	ts0 = &model._TimeSteps[U32( 0)];
        assert_eq!( ts0._Time, 0);
        assert_eq!( ts0._Values.Size().0, 2);
        assert_eq!( ts0._Values[U32( 0)]._Id, "!");
        assert_eq!( ts0._Values[U32( 0)]._ValStr, "0");
        assert_eq!( ts0._Values[U32( 1)]._Id, "\"");
        assert_eq!( ts0._Values[U32( 1)]._ValStr, "1010");

        let  	ts1 = &model._TimeSteps[U32( 1)];
        assert_eq!( ts1._Time, 1);
        assert_eq!( ts1._Values.Size().0, 1);
        assert_eq!( ts1._Values[U32( 0)]._Id, "!");
        assert_eq!( ts1._Values[U32( 0)]._ValStr, "1");

        let  	ts2 = &model._TimeSteps[U32( 2)];
        assert_eq!( ts2._Time, 2);
        assert_eq!( ts2._Values.Size().0, 2);
        assert_eq!( ts2._Values[U32( 0)]._Id, "!");
        assert_eq!( ts2._Values[U32( 0)]._ValStr, "0");
        assert_eq!( ts2._Values[U32( 1)]._Id, "\"");
        assert_eq!( ts2._Values[U32( 1)]._ValStr, "1100");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_vcd_parser_hierarchy()
    {
        use	crate::rube::vcdio::ParseVcd;

        let  	vcdContent = r#"
$version Kosh Rube Engine $end
$timescale 1ns $end
$scope module top $end
$var wire 1 ! clk $end
$scope module alu $end
$var wire 4 " a $end
$var wire 4 # b $end
$var wire 4 % out $end
$upscope $end
$upscope $end
$enddefinitions $end
$dumpvars
0!
b0000 "
b0000 #
b0000 %
$end
#5
1!
b0011 "
b0101 #
#10
0!
b1000 %
"#;

        let  	model = ParseVcd( vcdContent).expect( "Failed to parse hierarchical VCD");

        assert_eq!( model._Scopes.Size().0, 1);
        let  	top = &model._Scopes[U32( 0)];
        assert_eq!( top._Name, "top");
        assert_eq!( top._Vars.Size().0, 1);
        assert_eq!( top._Vars[U32( 0)]._Name, "clk");
        assert_eq!( top._Scopes.Size().0, 1);

        let  	alu = &top._Scopes[U32( 0)];
        assert_eq!( alu._Name, "alu");
        assert_eq!( alu._Vars.Size().0, 3);
        assert_eq!( alu._Vars[U32( 0)]._Name, "a");
        assert_eq!( alu._Vars[U32( 1)]._Name, "b");
        assert_eq!( alu._Vars[U32( 2)]._Name, "out");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_vcd_display_model()
    {
        use	crate::rube::{ vcdio::ParseVcd, VcdDisplayModel };

        let  	vcdContent = r#"
$version Kosh Rube Engine $end
$timescale 1ns $end
$scope module top $end
$var wire 1 ! clk $end
$scope module alu $end
$var wire 4 " a $end
$var wire 4 % out $end
$upscope $end
$upscope $end
$enddefinitions $end
$dumpvars
0!
b0001 "
b0001 %
$end
#10
1!
b0010 "
#20
0!
b0100 %
"#;

        let  	model = ParseVcd( vcdContent).expect( "Failed to parse VCD");
        let  	display = VcdDisplayModel::FromVcdModel( &model);

        assert_eq!( display.SignalCount().0, 3);
        assert_eq!( display._TimeMin, 0);
        assert_eq!( display._TimeMax, 20);

        let  	clk = display.Signal( U32( 0)).unwrap();
        assert_eq!( clk._FullName, "top.clk");
        assert!( clk.IsSingleBit());
        assert_eq!( clk.ValueAt( 0), "0");
        assert_eq!( clk.ValueAt( 5), "0");
        assert_eq!( clk.ValueAt( 10), "1");
        assert_eq!( clk.ValueAt( 15), "1");
        assert_eq!( clk.ValueAt( 20), "0");
        assert_eq!( clk.ValueAt( 100), "0");

        let  	aluA = display.Signal( U32( 1)).unwrap();
        assert_eq!( aluA._FullName, "top.alu.a");
        assert!( !aluA.IsSingleBit());
        assert_eq!( aluA.ValueAt( 0), "0001");
        assert_eq!( aluA.ValueAt( 9), "0001");
        assert_eq!( aluA.ValueAt( 10), "0010");
        assert_eq!( aluA.ValueAt( 25), "0010");

        let  	aluOut = display.Signal( U32( 2)).unwrap();
        assert_eq!( aluOut._FullName, "top.alu.out");
        assert_eq!( aluOut.ValueAt( 0), "0001");
        assert_eq!( aluOut.ValueAt( 19), "0001");
        assert_eq!( aluOut.ValueAt( 20), "0100");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_fifo()
    {
        use crate::rube::fifo::Fifo;
        use crate::rube::{ SimEngine, Layout, Reg };

        let  	mut layout = Layout::New();
        let  	fifo = Fifo::New( &mut layout, "MyFifo", 4, 32, None);

        let  	mut engine = SimEngine::Create( &layout);
        engine.SetPortBool( fifo.Reset(), Reg::TRUE);
        engine.SetPortBool( fifo.Clk(), Reg::FALSE);
        engine.Drive();

        // Check empty initially due to reset
        assert!( engine.GetPortBool( fifo.Empty()).unwrap().IsTrue());

        engine.SetPortBool( fifo.Reset(), Reg::FALSE);
        engine.SetPortBool( fifo.Push(), Reg::TRUE);
        engine.SetPortBool( fifo.Pop(), Reg::FALSE);

        // Push 4 values (0xA1, 0xA2, 0xA3, 0xA4)
        for i in 1..=4 {
            engine.SetPortValue( fifo.DataIn(), Reg::Known( 0xA0 + i));
            engine.SetPortBool( fifo.Clk(), Reg::TRUE);
            engine.Drive();
            engine.SetPortBool( fifo.Clk(), Reg::FALSE);
            engine.Drive();
        }

        assert!( engine.GetPortBool( fifo.Full()).unwrap().IsTrue());
        assert!( engine.GetPortBool( fifo.Empty()).unwrap().IsFalse());

        // Pop values
        engine.SetPortBool( fifo.Push(), Reg::FALSE);
        engine.SetPortBool( fifo.Pop(), Reg::TRUE);

        for i in 1..=4 {
            let  	dataOut = engine.GetPortValue( fifo.DataOut()).unwrap().Val();
            assert_eq!( dataOut, 0xA0 + i);

            engine.SetPortBool( fifo.Clk(), Reg::TRUE);
            engine.Drive();
            engine.SetPortBool( fifo.Clk(), Reg::FALSE);
            engine.Drive();
        }

        assert!( engine.GetPortBool( fifo.Empty()).unwrap().IsTrue());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_simt_warps()
    {
        let  	mut layout = Layout::New();
        let  	gateCount = U32( 128);

        // Add 128 AND gates and 128 XOR gates
        let  	mut andIn1 = Stash::WithCapacity( gateCount);
        let  	mut andIn2 = Stash::WithCapacity( gateCount);
        let  	mut andOut = Stash::WithCapacity( gateCount);

        let  	mut xorIn1 = Stash::WithCapacity( gateCount);
        let  	mut xorIn2 = Stash::WithCapacity( gateCount);
        let  	mut xorOut = Stash::WithCapacity( gateCount);

        USeg::New( U32::_0, gateCount).Traverse( |i| {
            let  	andMod = AndGate::New( &mut layout, &format!( "And_{}", i), None);
            andIn1.Push( andMod._In1);
            andIn2.Push( andMod._In2);
            andOut.Push( andMod._Out);

            let  	xorMod = XorGate::New( &mut layout, &format!( "Xor_{}", i), None);
            xorIn1.Push( xorMod._In1);
            xorIn2.Push( xorMod._In2);
            xorOut.Push( xorMod._Out);
        });

        assert!( layout.Freeze().is_ok());

        let  	mut engine = SimEngine::Create( &layout);

        // Verify warps were compiled
        assert!( engine._FastWarps.Size() >= U32( 2));

        // Drive even AND gates with (1, 1) -> 1, odd with (1, 0) -> 0
        USeg::New( U32::_0, gateCount).Traverse( |i| {
            engine.SetPortBool( andIn1[i], Reg::TRUE);
            let  	in2Val = if i.0 % 2 == 0 { Reg::TRUE } else { Reg::FALSE };
            engine.SetPortBool( andIn2[i], in2Val);

            // Drive XOR gates with (1, 0) -> 1
            engine.SetPortBool( xorIn1[i], Reg::TRUE);
            engine.SetPortBool( xorIn2[i], Reg::FALSE);
        });

        engine.Drive();

        // Check outputs
        USeg::New( U32::_0, gateCount).Traverse( |i| {
            let  	expectedAnd = if i.0 % 2 == 0 { Reg::TRUE } else { Reg::FALSE };
            assert_eq!( engine.GetPortBool( andOut[i]).unwrap(), expectedAnd);
            assert_eq!( engine.GetPortBool( xorOut[i]).unwrap(), Reg::TRUE);
        });
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_module_hierarchy_tree()
    {
        let  	mut layout = Layout::New();

        let  	topId = layout.AddModule( "TopBlock", None, &[], &[], KernelKind::None);
        let  	aluId = layout.AddModule( "ALU", Some( topId), &[], &[], KernelKind::None);

        let  	andGate = AndGate::New( &mut layout, "ALU_And", Some( aluId));
        let  	notGate = NotGate::New( &mut layout, "ALU_Not", Some( aluId));



        layout.Connect( andGate._Out, notGate._In);

        layout.Freeze().expect( "Compilation failed");

        // Find mapped IDs after QSort
        let  	mut newTopId = None;
        let  	mut newAluId = None;
        let  	mut newAndId = None;

        layout.Modules().iter().for_each( |m| {
            if m._Name == "TopBlock" { newTopId = Some( m._Id); }
            if m._Name == "ALU" { newAluId = Some( m._Id); }
            if m._Name == "ALU_And" { newAndId = Some( m._Id); }
        });

        let  	nTopId = newTopId.unwrap();
        let  	nAluId = newAluId.unwrap();
        let  	nAndId = newAndId.unwrap();

        assert!( layout.IsContainer( nTopId));
        assert!( layout.IsContainer( nAluId));
        assert!( !layout.IsContainer( nAndId));

        let  	topChildren = layout.SubModules( nTopId);
        assert_eq!( topChildren.len(), 1);
        assert_eq!( topChildren[0], nAluId);

        let  	aluChildren = layout.SubModules( nAluId);
        assert_eq!( aluChildren.len(), 2);

        let  	topDesc = layout.Descendents( nTopId);
        assert_eq!( topDesc.len(), 3);
        assert!( topDesc.contains( &nAluId));
        assert!( topDesc.contains( &nAndId));

        let  	aluDesc = layout.Descendents( nAluId);
        assert_eq!( aluDesc.len(), 2);
        assert!( aluDesc.contains( &nAndId));

        let  	andDesc = layout.Descendents( nAndId);
        assert_eq!( andDesc.len(), 0);

        let  	roots = layout.RootModules();
        assert_eq!( roots.Size().0, 1);
        assert_eq!( roots[U32( 0)], nTopId);

        let  	mut engine = SimEngine::Create( &layout);

        engine.SetPortBool( andGate._In1, Reg::TRUE);
        engine.SetPortBool( andGate._In2, Reg::TRUE);
        engine.Drive();

        assert_eq!( engine.GetPortBool( andGate._Out), Some( Reg::TRUE));
        // NOTE: combinational gates take multiple Drive cycles or automatically chain.
        // Let's call Drive() again just in case there's a latency of 1 tick.
        engine.Drive();
        assert_eq!( engine.GetPortBool( notGate._Out), Some( Reg::FALSE));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    #[should_panic( expected = "port is not visible beyond its immediate parent")]
    fn	test_hierarchy_scope_violation()
    {
        let  	mut layout = Layout::New();

        let  	topId = layout.AddModule( "Top", None, &[ PortDesc::Bool( "in") ], &[], KernelKind::None);
        let  	subId = layout.AddModule( "Sub", Some( topId), &[], &[], KernelKind::None);
        let  	leaf = AndGate::New( &mut layout, "Leaf", Some( subId));

        let  	topIn = layout.InPort( topId, 0).unwrap();

        // Attempt connecting grandparent topIn directly to grandchild leaf.In1()
        // Must panic with hierarchy visibility violation!
        layout.Connect( topIn, leaf.In1());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    #[should_panic( expected = "Connection validation failed")]
    fn	test_duplicate_input_driver_rejection()
    {
        let  	mut layout = Layout::New();

        let  	topId = layout.AddModule( "Top", None, &[], &[], KernelKind::None);
        let  	gate1 = AndGate::New( &mut layout, "G1", Some( topId));
        let  	gate2 = OrGate::New( &mut layout, "G2", Some( topId));
        let  	gate3 = NotGate::New( &mut layout, "G3", Some( topId));

        layout.Connect( gate1.Out(), gate3.In());
        // Attempt connecting a second driver to gate3.In()
        // Must panic with DuplicateInputDriver!
        layout.Connect( gate2.Out(), gate3.In());
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_incremental_trigger_assignment()
    {
        let  	mut layout = Layout::New();

        let  	topId = layout.AddModule( "Top", None, &[], &[], KernelKind::None);
        let  	subId = layout.AddModule(
            "SubBlock",
            Some( topId),
            &[ PortDesc::Bool( "subIn") ],
            &[ PortDesc::Bool( "subOut") ],
            KernelKind::None,
        );

        let  	subIn = layout.InPort( subId, 0).unwrap();
        let  	subOut = layout.OutPort( subId, 0).unwrap();

        let  	and1 = AndGate::New( &mut layout, "SubBlock.And1", Some( subId));
        let  	and2 = AndGate::New( &mut layout, "SubBlock.And2", Some( subId));

        // Connect boundary pass-down and internal sibling
        layout.Connect( subIn, and1.In1());
        layout.Connect( and1.Out(), and2.In1());
        layout.Connect( and2.Out(), subOut);

        // Before sealing subId, internal net between and1.Out() and and2.In1() has no trigger
        assert!( !layout._Netlist.HasTrigger( and1.Out()));

        // Seal subId
        layout.SealModule( subId);

        // After sealing subId:
        // 1. Internal net (and1.Out -> and2.In1) MUST have an assigned TriggerId!
        assert!( layout._Netlist.HasTrigger( and1.Out()));
        assert!( layout._Netlist.HasTrigger( and2.In1()));

        // 2. Boundary connected nets (subIn and subOut) are NOT yet assigned triggers because top is unsealed
        assert!( !layout._Netlist.HasTrigger( subIn));
        assert!( !layout._Netlist.HasTrigger( subOut));

        // Now seal topId
        layout.SealModule( topId);

        // All nets must now have assigned triggers!
        assert!( layout._Netlist.HasTrigger( subIn));
        assert!( layout._Netlist.HasTrigger( subOut));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_framework_hierarchy_and_traits()
    {
        let  	mut layout = Layout::New();

        let  	fa = FullAdder::New( &mut layout, "FA", None);

        // 1. Verify IModule::Id()
        assert_eq!( fa.Id(), fa._Id);
        assert_eq!( fa._HA1.Id(), fa._HA1._Id);
        assert_eq!( fa._HA1._Xor.Id(), fa._HA1._Xor._Id);

        // 2. Verify nested internal signals can be read
        let  	ha1XorOut = fa._HA1._Xor.Out();
        let  	ha1AndOut = fa._HA1._And.Out();
        assert!( ha1XorOut.IsOut());
        assert!( ha1AndOut.IsOut());

        // 3. Verify 4-bit adder holds concrete FullAdders in _Bits
        let  	adder4 = Adder::< 4>::New( &mut layout, "Adder4", None);
        assert_eq!( adder4._Bits.Size(), U32( 4));
        assert_eq!( adder4._Bits[0]._HA1._Xor.Out().Dir(), crate::rube::port::PortDir::Out);

        // 4. Test IModule and IPort traits directly
        let  	ha = HalfAdder::New( &mut layout, "HAStandalone", None);
        assert_eq!( IModule::Id( &ha), ha._Id);
        assert_eq!( IPort::Id( &ha.Sum()), ha._Sum);

        // 5. Freeze and run simulation to inspect internal submodule signals
        layout.Freeze().unwrap();
        let  	mut engine = SimEngine::Create( &layout);

        // Set A=1, B=1, CIn=0 on fa
        engine.SetPortBool( fa.In1(), Reg::TRUE);
        engine.SetPortBool( fa.In2(), Reg::TRUE);
        engine.SetPortBool( fa.CIn(), Reg::FALSE);
        for _ in 0..4 {
            engine.Drive();
        }

        // Check top boundary outputs: Sum=0, Carry=1
        assert_eq!( engine.GetPortBool( fa.Sum()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( fa.Carry()), Some( Reg::TRUE));

        // Directly check nested submodule internal outputs through dot notation!
        assert_eq!( engine.GetPortBool( fa._HA1._Xor.Out()), Some( Reg::FALSE));
        assert_eq!( engine.GetPortBool( fa._HA1._And.Out()), Some( Reg::TRUE));
        assert_eq!( engine.GetPortBool( fa._Or.Out()), Some( Reg::TRUE));
    }

    #[test]
    fn	test_fifo_serialization()
    {
        use crate::rube::fifo::Fifo;
        use crate::rube::Layout;
        use crate::flux::{ IFluxImportSource, FieldExp, FieldImp, jsonoutstrm::JsonOutStream, fluxexport::IFluxExportSink };
        use crate::flux::instream::FixedStream;
        use crate::shard::{ JSon, parser::Parser };
        use crate::silo::U32;

        let  	mut layout = Layout::New();
        let  	fifo = Fifo::New( &mut layout, "MyFifo", 4, 32, None);

        let  	mut jsonStr = String::new();
        let  	mut outStream = JsonOutStream::New( &mut jsonStr, false);
        outStream.DispatchFieldExp( FieldExp::FluxSource( &fifo));

        // Debug output
        println!( "{}", jsonStr);

        let  	mut importedFifo = Fifo::default();

        let  	mut stream = FixedStream::from( jsonStr.as_str());
        let  	mut parser = Parser::New( &mut stream);

        let  	mut field = FieldImp::Null;
        importedFifo.FetchFieldImp( &mut field);
        let  	json_parser = JSon::New( field);

        assert!( parser.ParseGrammar( &json_parser, U32( 0)).is_some());
        drop(json_parser);

        assert_eq!( fifo._Id, importedFifo._Id);
        assert_eq!( fifo._Clk, importedFifo._Clk);
        assert_eq!( fifo._Reset, importedFifo._Reset);
        assert_eq!( fifo._Push, importedFifo._Push);
        assert_eq!( fifo._Pop, importedFifo._Pop);
        assert_eq!( fifo._DataIn, importedFifo._DataIn);
        assert_eq!( fifo._DataOut, importedFifo._DataOut);
        assert_eq!( fifo._Empty, importedFifo._Empty);
        assert_eq!( fifo._Full, importedFifo._Full);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_coro_module_with_output()
    {
        use	crate::{
            rube::{
                coro_kernel::CoroPorts,
                engine::SimEngine,
                layout::Layout,
                port::{ PortDesc, PortType },
                reg::Reg,
            },
            stalks::Coro,
        };

        let  	mut layout = Layout::New();
        let  	inPorts = [PortDesc::New( "In", PortType::U32Val)];
        let  	outPorts = [PortDesc::New( "Out", PortType::U32Val)];

        let  	modId = layout.AddCoroModule( "Accumulator", None, &inPorts[..], &outPorts[..], || {
            Coro::New( move |yielder, mut inPorts: CoroPorts| {
                let  	mut counter: u64 = 0;
                loop {
                    let  	inVal = inPorts[0].Val();
                    counter += inVal;
                    inPorts = yielder.Suspend( CoroPorts::Single( Reg::Known( counter) ) );
                }
            })
        });

        layout.Freeze().unwrap();
        let  	mut engine = SimEngine::Create( &layout);
        let  	inPortId = layout.InPort( modId, 0).unwrap();
        let  	outPortId = layout.OutPort( modId, 0).unwrap();

        // Cycle 0: initial evaluation
        engine.Drive();
        assert_eq!( engine.GetPortValue( outPortId), Some( Reg::Known( 0) ) );

        // Cycle 1: inport changes from 0 to 5
        engine.SetPortValue( inPortId, Reg::Known( 5));
        engine.Drive();
        assert_eq!( engine.GetPortValue( outPortId), Some( Reg::Known( 5) ) );

        // Cycle 2: inport unchanged (still 5) -> coroutine NOT resumed
        engine.Drive();
        assert_eq!( engine.GetPortValue( outPortId), Some( Reg::Known( 5) ) );

        // Cycle 3: inport changes from 5 to 10 -> counter accumulates 10 (total 15)
        engine.SetPortValue( inPortId, Reg::Known( 10));
        engine.Drive();
        assert_eq!( engine.GetPortValue( outPortId), Some( Reg::Known( 15) ) );
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_coro_module_without_output()
    {
        use	std::sync::{
            atomic::{ AtomicU64, Ordering },
            Arc,
        };
        use	crate::{
            rube::{
                coro_kernel::CoroPorts,
                engine::SimEngine,
                layout::Layout,
                port::{ PortDesc, PortType },
                reg::Reg,
            },
            stalks::Coro,
        };

        let  	received = Arc::new( AtomicU64::new( 0) );
        let  	recClone = Arc::clone( &received);

        let  	mut layout = Layout::New();
        let  	inPorts = [PortDesc::New( "DataIn", PortType::U32Val)];
        let  	outPorts: [PortDesc; 0] = [];

        let  	modId = layout.AddCoroModule( "SinkMonitor", None, &inPorts[..], &outPorts[..], move || {
            let  	r = Arc::clone( &recClone);
            Coro::New( move |yielder, mut inPorts: CoroPorts| {
                loop {
                    let  	val = inPorts[0].Val();
                    r.store( val, Ordering::SeqCst);
                    inPorts = yielder.Suspend( CoroPorts::Empty() );
                }
            })
        });

        layout.Freeze().unwrap();
        let  	mut engine = SimEngine::Create( &layout);
        let  	inPortId = layout.InPort( modId, 0).unwrap();

        // Cycle 0: initial evaluation (inport is 0)
        engine.Drive();
        assert_eq!( received.load( Ordering::SeqCst), 0);

        // Cycle 1: send 42
        engine.SetPortValue( inPortId, Reg::Known( 42));
        engine.Drive();
        assert_eq!( received.load( Ordering::SeqCst), 42);

        // Cycle 2: unchanged
        engine.Drive();
        assert_eq!( received.load( Ordering::SeqCst), 42);

        // Cycle 3: send 99
        engine.SetPortValue( inPortId, Reg::Known( 99));
        engine.Drive();
        assert_eq!( received.load( Ordering::SeqCst), 99);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_coro_module_multistep_protocol()
    {
        use	crate::{
            rube::{
                coro_kernel::CoroPorts,
                engine::SimEngine,
                layout::Layout,
                port::{ PortDesc, PortType },
                reg::Reg,
            },
            stalks::Coro,
        };

        let  	mut layout = Layout::New();
        let  	inPorts = [
            PortDesc::New( "Req", PortType::Bool),
            PortDesc::New( "Data", PortType::U32Val),
        ];
        let  	outPorts = [
            PortDesc::New( "Ack", PortType::Bool),
            PortDesc::New( "Result", PortType::U32Val),
        ];

        // Handshake coroutine:
        // Waits for Req=1, then asserts Ack=1 and Result = Data * 2.
        // Then waits for Req=0, then de-asserts Ack=0.
        let  	modId = layout.AddCoroModule( "ProtocolServer", None, &inPorts[..], &outPorts[..], || {
            Coro::New( move |yielder, mut inPorts: CoroPorts| {
                loop {
                    // Idle state: Ack = 0, Result = 0
                    while !inPorts[0].IsTrue() {
                        inPorts = yielder.Suspend( CoroPorts::Pair( Reg::FALSE, Reg::Known( 0) ) );
                    }
                    // Req received: compute result and Ack = 1
                    let  	dataVal = inPorts[1].Val();
                    let  	res = dataVal * 2;
                    while inPorts[0].IsTrue() {
                        inPorts = yielder.Suspend( CoroPorts::Pair( Reg::TRUE, Reg::Known( res) ) );
                    }
                    // Req de-asserted: return to Ack = 0
                    inPorts = yielder.Suspend( CoroPorts::Pair( Reg::FALSE, Reg::Known( 0) ) );
                }
            })
        });

        layout.Freeze().unwrap();
        let  	mut engine = SimEngine::Create( &layout);
        let  	reqPort = layout.InPort( modId, 0).unwrap();
        let  	dataPort = layout.InPort( modId, 1).unwrap();
        let  	ackPort = layout.OutPort( modId, 0).unwrap();
        let  	resultPort = layout.OutPort( modId, 1).unwrap();

        // Cycle 0: initial idle state
        engine.Drive();
        assert_eq!( engine.GetPortBool( ackPort), Some( Reg::FALSE));
        assert_eq!( engine.GetPortValue( resultPort), Some( Reg::Known( 0) ) );

        // Cycle 1: Request with Data=21
        engine.SetPortValue( dataPort, Reg::Known( 21));
        engine.SetPortBool( reqPort, Reg::TRUE);
        engine.Drive();
        assert_eq!( engine.GetPortBool( ackPort), Some( Reg::TRUE));
        assert_eq!( engine.GetPortValue( resultPort), Some( Reg::Known( 42) ) );

        // Cycle 2: Keep Req=1
        engine.Drive();
        assert_eq!( engine.GetPortBool( ackPort), Some( Reg::TRUE));
        assert_eq!( engine.GetPortValue( resultPort), Some( Reg::Known( 42) ) );

        // Cycle 3: Deassert Req=0
        engine.SetPortBool( reqPort, Reg::FALSE);
        engine.Drive();
        assert_eq!( engine.GetPortBool( ackPort), Some( Reg::FALSE));
        assert_eq!( engine.GetPortValue( resultPort), Some( Reg::Known( 0) ) );
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    ConsoleTest!( TestParallelDeterminism {
        let  	mut layout = Layout::New();
        let  	adder = Adder::< 8>::New( &mut layout, "test_adder", None);
        layout.Freeze().unwrap();

        let  	mut serialEngine = SimEngine::Create( &layout);
        let  	mut parallelEngine = SimEngine::Create( &layout).WithMode( SimEngineMode::Parallel( U8( 4)));

        let  	isConsole = crate::silo::IsConsoleEnabled();
        let  	mut vcdOpt = if isConsole {
            let  	w = VcdWriter::New( &layout, &parallelEngine);
            let  	mut s = String::new();
            w.WriteHeader( &layout, &parallelEngine, &mut s);
            Some( ( w, s))
        } else {
            None
        };

        // 4 distinct test vectors x 25 cycles each = 100 cycles total
        let  	testCases = [
            ( 0x55u32, 0xAAu32, Reg::FALSE, 0xFFusize, false),
            ( 0xFFu32, 0x01u32, Reg::FALSE, 0x00usize, true),
            ( 0x7Fu32, 0x7Fu32, Reg::TRUE,  0xFFusize, false),
            ( 0xFFu32, 0xFFu32, Reg::TRUE,  0xFFusize, true),
        ];

        for ( aVal, bVal, cinVal, expectedSum, expectedCarry) in testCases {
            adder.SetA( &mut serialEngine, U32( aVal));
            adder.SetB( &mut serialEngine, U32( bVal));
            adder.SetCarryIn( &mut serialEngine, cinVal);

            adder.SetA( &mut parallelEngine, U32( aVal));
            adder.SetB( &mut parallelEngine, U32( bVal));
            adder.SetCarryIn( &mut parallelEngine, cinVal);

            for _ in 0..25 {
                serialEngine.Drive();
                parallelEngine.Drive();

                assert_eq!( serialEngine._CycleCount, parallelEngine._CycleCount);
                // Verify strict bit-for-bit parity across every single trigger in the circuit
                assert_eq!( serialEngine._Triggers._CurrentVals, parallelEngine._Triggers._CurrentVals);

                if let Some( ( ref w, ref mut s)) = vcdOpt {
                    w.DumpCycle( &parallelEngine, s);
                }
            }

            // Verify final converged arithmetic results on both engines
            assert_eq!( adder.GetSum( &serialEngine), expectedSum);
            assert_eq!( adder.GetSum( &parallelEngine), expectedSum);
            assert_eq!( serialEngine.GetPortBool( adder.Carry()).unwrap().IsTrue(), expectedCarry);
            assert_eq!( parallelEngine.GetPortBool( adder.Carry()).unwrap().IsTrue(), expectedCarry);
        }

        assert_eq!( parallelEngine._CycleCount, 100);

        if let Some( ( _, ref vcdStr)) = vcdOpt {
            crate::cprintln!( "{}", vcdStr);
            let  	_ = std::fs::write( "test_parallel_determinism.vcd", vcdStr);
        }
    });

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_static_module_encapsulation()
    {
        let  	mut layout = Layout::New();
        let  	pipeline = AdderPipeline::New( &mut layout, "AdderPipeline", None);
        layout.Freeze().expect( "Compilation failed");

        let  	mut engine = SimEngine::Create( &layout);

        // Test vector 1: 15 + 25 + 60 = 100
        engine.SetPortU32( pipeline.A(), Reg::Known( 15));
        engine.SetPortU32( pipeline.B(), Reg::Known( 25));
        engine.SetPortU32( pipeline.C(), Reg::Known( 60));

        USeg::New( U32::_0, U32( 5)).Traverse( |_| {
            engine.Drive();
        });

        let  	resVal1 = engine.GetPortU32( pipeline.Sum()).unwrap();
        assert_eq!( resVal1, Reg::Known( 100));

        // Test vector 2: 1000 + 2000 + 3000 = 6000
        engine.SetPortU32( pipeline.A(), Reg::Known( 1000));
        engine.SetPortU32( pipeline.B(), Reg::Known( 2000));
        engine.SetPortU32( pipeline.C(), Reg::Known( 3000));

        USeg::New( U32::_0, U32( 5)).Traverse( |_| {
            engine.Drive();
        });

        let  	resVal2 = engine.GetPortU32( pipeline.Sum()).unwrap();
        assert_eq!( resVal2, Reg::Known( 6000));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub struct NestedAdderPipeline
    {
        pub _Id:      ModuleId,
        pub _Stage1:  BusAdder32,
        pub _Stage2:  BusAdder32,
        pub _V1:      PortId,
        pub _V2:      PortId,
        pub _V3:      PortId,
        pub _VOut:    PortId,
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    impl NestedAdderPipeline
    {
        pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>) -> Self
        {
            let  	modId = layout.AddModule(
                name,
                parent,
                &[ PortDesc::U32( "v1"), PortDesc::U32( "v2"), PortDesc::U32( "v3") ],
                &[ PortDesc::U32( "vout") ],
                KernelKind::None,
            );

            let  	v1 = layout.InPort( modId, 0).unwrap();
            let  	v2 = layout.InPort( modId, 1).unwrap();
            let  	v3 = layout.InPort( modId, 2).unwrap();
            let  	vout = layout.OutPort( modId, 0).unwrap();

            let  	stage1 = BusAdder32::New( layout, &format!( "{name}.Stage1"), Some( modId));
            let  	stage2 = BusAdder32::New( layout, &format!( "{name}.Stage2"), Some( modId));

            layout.Connect( v1, stage1.A());
            layout.Connect( v2, stage1.B());
            layout.Connect( stage1.Sum(), stage2.A());
            layout.Connect( v3, stage2.B());
            layout.Connect( stage2.Sum(), vout);

            layout.SealModule( modId);

            return Self {
                _Id:      modId,
                _Stage1:  stage1,
                _Stage2:  stage2,
                _V1:      v1,
                _V2:      v2,
                _V3:      v3,
                _VOut:    vout,
            };
        }

        #[inline]
        pub const fn	Id( &self) -> ModuleId
        {
            return self._Id;
        }

        #[inline]
        pub const fn	V1( &self) -> PortId
        {
            return self._V1;
        }

        #[inline]
        pub const fn	V2( &self) -> PortId
        {
            return self._V2;
        }

        #[inline]
        pub const fn	V3( &self) -> PortId
        {
            return self._V3;
        }

        #[inline]
        pub const fn	VOut( &self) -> PortId
        {
            return self._VOut;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_static_nested_containers()
    {
        let  	mut layout = Layout::New();
        let  	pipeline = NestedAdderPipeline::New( &mut layout, "DUT", None);
        layout.Freeze().expect( "Compilation failed");

        let  	mut engine = SimEngine::Create( &layout);

        engine.SetPortU32( pipeline.V1(), Reg::Known( 111));
        engine.SetPortU32( pipeline.V2(), Reg::Known( 222));
        engine.SetPortU32( pipeline.V3(), Reg::Known( 333));

        USeg::New( U32::_0, U32( 5)).Traverse( |_| {
            engine.Drive();
        });

        assert_ne!( pipeline.Id(), pipeline._Stage1.Id());
        assert_eq!( engine.GetPortU32( pipeline.VOut()).unwrap(), Reg::Known( 666));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub struct AdderTestIO
    {
        pub _Id:     ModuleId,
        pub _Sum:    PortId,
        pub _Carry:  PortId,
        pub _A:      PortId,
        pub _B:      PortId,
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    impl AdderTestIO
    {
        pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>, passedFlag: std::sync::Arc< std::sync::atomic::AtomicBool>) -> Self
        {
            let  	passed = std::sync::Arc::clone( &passedFlag);
            let  	coroKernel = KernelKind::Coro( std::sync::Arc::new( move || {
                let  	passedRef = std::sync::Arc::clone( &passed);
                Coro::New( move |yielder, _inPorts: CoroPorts| {
                    // Cycle 0: Drive 15 + 25
                    let  	mut out = CoroPorts::New();
                    out._Vals[0] = Reg::Known( 15);
                    out._Vals[1] = Reg::Known( 25);
                    out._Len = U32( 2);
                    let  	mut inPorts = yielder.Suspend( out);

                    // Cycle 1: Check 15 + 25 = 40, Drive 100 + 200
                    assert_eq!( inPorts[0].Val(), 40);
                    assert!( !inPorts[1].IsTrue());
                    let  	mut out = CoroPorts::New();
                    out._Vals[0] = Reg::Known( 100);
                    out._Vals[1] = Reg::Known( 200);
                    out._Len = U32( 2);
                    inPorts = yielder.Suspend( out);

                    // Cycle 2: Check 100 + 200 = 300, Drive 0xFFFF_FFFF + 1
                    assert_eq!( inPorts[0].Val(), 300);
                    assert!( !inPorts[1].IsTrue());
                    let  	mut out = CoroPorts::New();
                    out._Vals[0] = Reg::Known( 0xFFFF_FFFF);
                    out._Vals[1] = Reg::Known( 1);
                    out._Len = U32( 2);
                    inPorts = yielder.Suspend( out);

                    // Cycle 3: Check 0xFFFF_FFFF + 1 = 0 (carry=true)
                    assert_eq!( inPorts[0].Val(), 0);
                    assert!( inPorts[1].IsTrue());

                    passedRef.store( true, std::sync::atomic::Ordering::SeqCst);

                    loop {
                        let  	_ = yielder.Suspend( CoroPorts::New());
                    }
                })
            }));

            let  	modId = layout.AddModule(
                name,
                parent,
                &[ PortDesc::U32( "sum"), PortDesc::Bool( "carry") ],
                &[ PortDesc::U32( "a"), PortDesc::U32( "b") ],
                coroKernel,
            );

            let  	sum = layout.InPort( modId, 0).unwrap();
            let  	carry = layout.InPort( modId, 1).unwrap();
            let  	a = layout.OutPort( modId, 0).unwrap();
            let  	b = layout.OutPort( modId, 1).unwrap();

            layout.SealModule( modId);

            return Self {
                _Id:     modId,
                _Sum:    sum,
                _Carry:  carry,
                _A:      a,
                _B:      b,
            };
        }

        #[inline]
        pub const fn	Id( &self) -> ModuleId
        {
            return self._Id;
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
        pub const fn	A( &self) -> PortId
        {
            return self._A;
        }

        #[inline]
        pub const fn	B( &self) -> PortId
        {
            return self._B;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub struct RubeTest_Adder
    {
        pub _Id:      ModuleId,
        pub _DUT:     BusAdder32,
        pub _TestIO:  AdderTestIO,
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    impl RubeTest_Adder
    {
        pub fn	New( layout: &mut Layout, name: &str, passedFlag: std::sync::Arc< std::sync::atomic::AtomicBool>) -> Self
        {
            let  	topId = layout.AddModule(
                name,
                None,
                &[],
                &[],
                KernelKind::None,
            );

            let  	dut = BusAdder32::New( layout, &format!( "{name}.DUT"), Some( topId));
            let  	testIo = AdderTestIO::New( layout, &format!( "{name}.TestIO"), Some( topId), passedFlag);

            layout.Connect( testIo.A(), dut.A());
            layout.Connect( testIo.B(), dut.B());
            layout.Connect( dut.Sum(), testIo.Sum());
            layout.Connect( dut.Carry(), testIo.Carry());

            layout.SealModule( topId);

            return Self {
                _Id:      topId,
                _DUT:     dut,
                _TestIO:  testIo,
            };
        }

        #[inline]
        pub const fn	Id( &self) -> ModuleId
        {
            return self._Id;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rube_adder_top_module()
    {
        let  	mut layout = Layout::New();
        let  	passed = std::sync::Arc::new( std::sync::atomic::AtomicBool::new( false));
        let  	_testTop = RubeTest_Adder::New( &mut layout, "RubeTest_Adder", std::sync::Arc::clone( &passed));
        layout.Freeze().expect( "Compilation failed");

        let  	mut engine = SimEngine::Create( &layout);
        USeg::New( U32::_0, U32( 10)).Traverse( |_| {
            engine.Drive();
        });

        assert_ne!( _testTop.Id(), _testTop._DUT.Id());
        assert_ne!( _testTop._TestIO.Id(), _testTop._DUT.Id());
        assert!( passed.load( std::sync::atomic::Ordering::SeqCst));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub struct DLatchTestIO
    {
        pub _Id:   ModuleId,
        pub _Q:    PortId,
        pub _Q1:   PortId,
        pub _D:    PortId,
        pub _En:   PortId,
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    impl DLatchTestIO
    {
        pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>, passedFlag: std::sync::Arc< std::sync::atomic::AtomicBool>) -> Self
        {
            let  	passed = std::sync::Arc::clone( &passedFlag);
            let  	coroKernel = KernelKind::Coro( std::sync::Arc::new( move || {
                let  	passedRef = std::sync::Arc::clone( &passed);
                Coro::New( move |yielder, _inPorts: CoroPorts| {
                    // Vector 1: en=1, d=1 -> q becomes 1, q1 becomes 0
                    let  	mut out = CoroPorts::New();
                    out._Vals[0] = Reg::TRUE;
                    out._Vals[1] = Reg::TRUE;
                    out._Len = U32( 2);
                    let  	mut inPorts = yielder.Suspend( out);

                    while !( inPorts[0].IsTrue() && inPorts[1].IsFalse()) {
                        inPorts = yielder.Suspend( CoroPorts::New());
                    }

                    // Vector 2: en=1, d=0 -> q transitions to 0, q1 to 1
                    let  	mut out = CoroPorts::New();
                    out._Vals[0] = Reg::FALSE;
                    out._Vals[1] = Reg::TRUE;
                    out._Len = U32( 2);
                    inPorts = yielder.Suspend( out);

                    while !( inPorts[0].IsFalse() && inPorts[1].IsTrue()) {
                        inPorts = yielder.Suspend( CoroPorts::New());
                    }

                    // Vector 3: en=1, d=1 -> q transitions back to 1, q1 to 0
                    let  	mut out = CoroPorts::New();
                    out._Vals[0] = Reg::TRUE;
                    out._Vals[1] = Reg::TRUE;
                    out._Len = U32( 2);
                    inPorts = yielder.Suspend( out);

                    while !( inPorts[0].IsTrue() && inPorts[1].IsFalse()) {
                        inPorts = yielder.Suspend( CoroPorts::New());
                    }

                    passedRef.store( true, std::sync::atomic::Ordering::SeqCst);

                    loop {
                        let  	_ = yielder.Suspend( CoroPorts::New());
                    }
                })
            }));

            let  	modId = layout.AddModule(
                name,
                parent,
                &[ PortDesc::Bool( "q"), PortDesc::Bool( "q1") ],
                &[ PortDesc::Bool( "d"), PortDesc::Bool( "en") ],
                coroKernel,
            );

            let  	q = layout.InPort( modId, 0).unwrap();
            let  	q1 = layout.InPort( modId, 1).unwrap();
            let  	d = layout.OutPort( modId, 0).unwrap();
            let  	en = layout.OutPort( modId, 1).unwrap();

            layout.SealModule( modId);

            return Self {
                _Id:   modId,
                _Q:    q,
                _Q1:   q1,
                _D:    d,
                _En:   en,
            };
        }

        #[inline]
        pub const fn	Id( &self) -> ModuleId
        {
            return self._Id;
        }

        #[inline]
        pub const fn	Q( &self) -> PortId
        {
            return self._Q;
        }

        #[inline]
        pub const fn	Q1( &self) -> PortId
        {
            return self._Q1;
        }

        #[inline]
        pub const fn	D( &self) -> PortId
        {
            return self._D;
        }

        #[inline]
        pub const fn	En( &self) -> PortId
        {
            return self._En;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub struct RubeTest_DLatch
    {
        pub _Id:      ModuleId,
        pub _DUT:     DLatch,
        pub _TestIO:  DLatchTestIO,
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    impl RubeTest_DLatch
    {
        pub fn	New( layout: &mut Layout, name: &str, passedFlag: std::sync::Arc< std::sync::atomic::AtomicBool>) -> Self
        {
            let  	topId = layout.AddModule(
                name,
                None,
                &[],
                &[],
                KernelKind::None,
            );

            let  	dut = DLatch::New( layout, &format!( "{name}.DUT"), Some( topId));
            let  	testIo = DLatchTestIO::New( layout, &format!( "{name}.TestIO"), Some( topId), passedFlag);

            layout.Connect( testIo.D(), dut.D());
            layout.Connect( testIo.D(), dut.DInv());
            layout.Connect( testIo.En(), dut.E1());
            layout.Connect( testIo.En(), dut.E2());
            layout.Connect( dut.Q(), testIo.Q());
            layout.Connect( dut.Q1(), testIo.Q1());

            layout.SealModule( topId);

            return Self {
                _Id:      topId,
                _DUT:     dut,
                _TestIO:  testIo,
            };
        }

        #[inline]
        pub const fn	Id( &self) -> ModuleId
        {
            return self._Id;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rube_dlatch_top_module()
    {
        let  	mut layout = Layout::New();
        let  	passed = std::sync::Arc::new( std::sync::atomic::AtomicBool::new( false));
        let  	_testTop = RubeTest_DLatch::New( &mut layout, "RubeTest_DLatch", std::sync::Arc::clone( &passed));
        layout.Freeze().expect( "Compilation failed");

        let  	mut engine = SimEngine::Create( &layout);
        USeg::New( U32::_0, U32( 20)).Traverse( |_| {
            engine.Drive();
        });

        assert_ne!( _testTop.Id(), _testTop._DUT.Id());
        assert_ne!( _testTop._TestIO.Id(), _testTop._DUT.Id());
        assert!( passed.load( std::sync::atomic::Ordering::SeqCst));
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub struct AdderPipelineTestIO
    {
        pub _Id:     ModuleId,
        pub _Sum:    PortId,
        pub _Carry:  PortId,
        pub _A:      PortId,
        pub _B:      PortId,
        pub _C:      PortId,
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    impl AdderPipelineTestIO
    {
        pub fn	New( layout: &mut Layout, name: &str, parent: Option< ModuleId>, passedFlag: std::sync::Arc< std::sync::atomic::AtomicBool>) -> Self
        {
            let  	passed = std::sync::Arc::clone( &passedFlag);
            let  	coroKernel = KernelKind::Coro( std::sync::Arc::new( move || {
                let  	passedRef = std::sync::Arc::clone( &passed);
                Coro::New( move |yielder, _inPorts: CoroPorts| {
                    // Vector 1: 15 + 25 + 60
                    let  	mut out = CoroPorts::New();
                    out._Vals[0] = Reg::Known( 15);
                    out._Vals[1] = Reg::Known( 25);
                    out._Vals[2] = Reg::Known( 60);
                    out._Len = U32( 3);
                    let  	_ = yielder.Suspend( out);

                    // Wait 1 cycle for 2-stage adder pipeline propagation
                    let  	mut inPorts = yielder.Suspend( CoroPorts::New());
                    assert_eq!( inPorts[0].Val(), 100);

                    // Vector 2: 1000 + 2000 + 3000
                    let  	mut out = CoroPorts::New();
                    out._Vals[0] = Reg::Known( 1000);
                    out._Vals[1] = Reg::Known( 2000);
                    out._Vals[2] = Reg::Known( 3000);
                    out._Len = U32( 3);
                    let  	_ = yielder.Suspend( out);

                    inPorts = yielder.Suspend( CoroPorts::New());
                    assert_eq!( inPorts[0].Val(), 6000);

                    passedRef.store( true, std::sync::atomic::Ordering::SeqCst);

                    loop {
                        let  	_ = yielder.Suspend( CoroPorts::New());
                    }
                })
            }));

            let  	modId = layout.AddModule(
                name,
                parent,
                &[ PortDesc::U32( "sum"), PortDesc::Bool( "carry") ],
                &[ PortDesc::U32( "a"), PortDesc::U32( "b"), PortDesc::U32( "c") ],
                coroKernel,
            );

            let  	sum = layout.InPort( modId, 0).unwrap();
            let  	carry = layout.InPort( modId, 1).unwrap();
            let  	a = layout.OutPort( modId, 0).unwrap();
            let  	b = layout.OutPort( modId, 1).unwrap();
            let  	c = layout.OutPort( modId, 2).unwrap();

            layout.SealModule( modId);

            return Self {
                _Id:     modId,
                _Sum:    sum,
                _Carry:  carry,
                _A:      a,
                _B:      b,
                _C:      c,
            };
        }

        #[inline]
        pub const fn	Id( &self) -> ModuleId
        {
            return self._Id;
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
        pub const fn	A( &self) -> PortId
        {
            return self._A;
        }

        #[inline]
        pub const fn	B( &self) -> PortId
        {
            return self._B;
        }

        #[inline]
        pub const fn	C( &self) -> PortId
        {
            return self._C;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    pub struct RubeTest_AdderPipeline
    {
        pub _Id:      ModuleId,
        pub _DUT:     AdderPipeline,
        pub _TestIO:  AdderPipelineTestIO,
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    impl RubeTest_AdderPipeline
    {
        pub fn	New( layout: &mut Layout, name: &str, passedFlag: std::sync::Arc< std::sync::atomic::AtomicBool>) -> Self
        {
            let  	topId = layout.AddModule(
                name,
                None,
                &[],
                &[],
                KernelKind::None,
            );

            let  	dut = AdderPipeline::New( layout, &format!( "{name}.DUT"), Some( topId));
            let  	testIo = AdderPipelineTestIO::New( layout, &format!( "{name}.TestIO"), Some( topId), passedFlag);

            layout.Connect( testIo.A(), dut.A());
            layout.Connect( testIo.B(), dut.B());
            layout.Connect( testIo.C(), dut.C());
            layout.Connect( dut.Sum(), testIo.Sum());
            layout.Connect( dut.Carry(), testIo.Carry());

            layout.SealModule( topId);

            return Self {
                _Id:      topId,
                _DUT:     dut,
                _TestIO:  testIo,
            };
        }

        #[inline]
        pub const fn	Id( &self) -> ModuleId
        {
            return self._Id;
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rube_adder_pipeline_top_module()
    {
        let  	passed = std::sync::Arc::new( std::sync::atomic::AtomicBool::new( false));
        let  	mut layout = Layout::New();
        let  	_testTop = RubeTest_AdderPipeline::New( &mut layout, "RubeTest_AdderPipeline", std::sync::Arc::clone( &passed));
        layout.Freeze().expect( "Compilation failed");

        let  	mut engine = SimEngine::Create( &layout);
        USeg::New( U32::_0, U32( 10)).Traverse( |_| {
            engine.Drive();
        });

        assert_ne!( _testTop.Id(), _testTop._DUT.Id());
        assert_ne!( _testTop._TestIO.Id(), _testTop._DUT.Id());
        assert!( passed.load( std::sync::atomic::Ordering::SeqCst));
    }
}


