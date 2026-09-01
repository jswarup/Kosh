//-- _tests.rs --------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests
{
    use	crate::{
        rube::{
            engine::SimEngine,
            adder::Adder,
            gates::{ AndGate, NandGate, NotGate, OrGate, XorGate },
            latches::{ CRSLatch, DLatch },
            layout::Layout,
            port::PortId,
            reg::Reg,
        },
        silo::U32,
    };

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_adder_basic()
    {
        let  	mut layout = Layout::New();
        const N: usize = 16;
        let  	adder = Adder::< N>::New( &mut layout, "Adder16");
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
        let  	crs = CRSLatch::New( &mut layout, "CRSLatch");
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
        let  	dLatch = DLatch::New( &mut layout, "DLatch");
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
        let  	dLatch = DLatch::New( &mut layout, "DLatch");
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

}
