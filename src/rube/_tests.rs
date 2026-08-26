//-- _tests.rs --------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests {
    use	crate::{
        rube::{
            adder::{ Adder, IAdder },
            gates::{ AndGate, NandGate, NotGate, OrGate, XorGate },
            latches::{ CRSLatch, DLatch, ICRSLatch, IDLatch, IRSLatch, RSLatch },
            portlayout::{ IPort, IPortLayout, Port, PortLayout },
            reg::{ IReg, IRegBool, Reg },
            sim_context::{ ISimContext, SimContext },
            trigger::{ ITriggerWad, ITriggerWadBool, TriggerWad },
        },
        silo::{ IEdgeBroadcast, IEdgeConnect, U32 },
    };

    #[test]
    fn	test_adder_basic()
    {
        let  	mut ctxt = SimContext::New();

        const N: usize = 16;
        let  	adder = Adder::< N>::New( &mut ctxt, "Adder16");

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
            adder.SetA( &mut ctxt, U32( aVal));
            adder.SetB( &mut ctxt, U32( bVal));

            ctxt.Drive();

            let  	sumVal = adder.GetSum( &ctxt);
            let  	carryVal = ctxt.GetValue( adder.Carry()).IsTrue();

            // expectedSum is calculated as a 16-bit truncation
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
        let  	mut ctxt = SimContext::New();
        let  	crs = CRSLatch::New( &mut ctxt, "CRSLatch");

        // When clk=0, S and R changes have no effect
        ctxt.SetValue( crs.Clk(), Reg::FALSE);
        ctxt.SetValue( crs.S(), Reg::TRUE);
        ctxt.SetValue( crs.R(), Reg::FALSE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( crs.Q()), Reg::FALSE);

        // Pulse clock high while S=1, R=0 -> Q becomes 1
        ctxt.SetValue( crs.Clk(), Reg::TRUE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( crs.Q()), Reg::TRUE);
        assert_eq!( ctxt.GetValue( crs.Q1()), Reg::FALSE);

        // Clock back low -> retains Q=1
        ctxt.SetValue( crs.Clk(), Reg::FALSE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( crs.Q()), Reg::TRUE);

        // Set R=1, S=0 with clk=0 -> still Q=1
        ctxt.SetValue( crs.S(), Reg::FALSE);
        ctxt.SetValue( crs.R(), Reg::TRUE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( crs.Q()), Reg::TRUE);

        // Pulse clk=1 -> Q resets to 0
        ctxt.SetValue( crs.Clk(), Reg::TRUE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( crs.Q()), Reg::FALSE);
        assert_eq!( ctxt.GetValue( crs.Q1()), Reg::TRUE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_d_latch()
    {
        let  	mut ctxt = SimContext::New();
        let  	dLatch = DLatch::New( &mut ctxt, "DLatch");

        // Enable = 0 ( latched): changing D has no effect
        ctxt.SetValue( dLatch.E(), Reg::FALSE);
        ctxt.SetValue( dLatch.D(), Reg::TRUE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( dLatch.Q()), Reg::FALSE);

        // Enable = 1 ( transparent): D=1 passes to Q=1
        ctxt.SetValue( dLatch.E(), Reg::TRUE);
        ctxt.SetValue( dLatch.D(), Reg::TRUE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( dLatch.Q()), Reg::TRUE);
        assert_eq!( ctxt.GetValue( dLatch.Q1()), Reg::FALSE);

        // D=0 passes to Q=0
        ctxt.SetValue( dLatch.D(), Reg::FALSE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( dLatch.Q()), Reg::FALSE);
        assert_eq!( ctxt.GetValue( dLatch.Q1()), Reg::TRUE);

        // Set D=1, then Enable=0 ( latch 1)
        ctxt.SetValue( dLatch.D(), Reg::TRUE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( dLatch.Q()), Reg::TRUE);

        ctxt.SetValue( dLatch.E(), Reg::FALSE);
        ctxt.Drive();

        // Now change D=0 while E=0 -> Q remains 1
        ctxt.SetValue( dLatch.D(), Reg::FALSE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( dLatch.Q()), Reg::TRUE, "Q should stay latched at 1");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_nand_gate()
    {
        let  	mut ctxt = SimContext::New();
        let  	in1 = ctxt.AddTrigger( "in1", Reg::FALSE);
        let  	in2 = ctxt.AddTrigger( "in2", Reg::FALSE);
        let  	out = ctxt.AddTrigger( "out", Reg::TRUE);

        let  	_gate = NandGate::New( &mut ctxt, "Nand", in1, in2, out);

        let  	truthTable = [
            ( false, false, Reg::TRUE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::FALSE),
        ];

        for ( a, b, expected) in truthTable {
            ctxt.SetValue( in1, Reg::FromBool( a));
            ctxt.SetValue( in2, Reg::FromBool( b));
            ctxt.Drive();
            assert_eq!( ctxt.GetValue( out), expected, "NAND failed for {a} and {b}");
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_not_gate()
    {
        let  	mut ctxt = SimContext::New();
        let  	in1 = ctxt.AddTrigger( "in1", Reg::FALSE);
        let  	out = ctxt.AddTrigger( "out", Reg::TRUE);

        let  	_gate = NotGate::New( &mut ctxt, "Not", in1, out);

        ctxt.SetValue( in1, Reg::FALSE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( out), Reg::TRUE);

        ctxt.SetValue( in1, Reg::TRUE);
        ctxt.Drive();
        assert_eq!( ctxt.GetValue( out), Reg::FALSE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_and_gate()
    {
        let  	mut ctxt = SimContext::New();
        let  	in1 = ctxt.AddTrigger( "in1", Reg::FALSE);
        let  	in2 = ctxt.AddTrigger( "in2", Reg::FALSE);
        let  	out = ctxt.AddTrigger( "out", Reg::FALSE);

        let  	_gate = AndGate::New( &mut ctxt, U32( 0), "And", in1, in2, out);

        let  	truthTable = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::FALSE),
            ( true, false, Reg::FALSE),
            ( true, true, Reg::TRUE),
        ];

        for ( a, b, expected) in truthTable {
            ctxt.SetValue( in1, Reg::FromBool( a));
            ctxt.SetValue( in2, Reg::FromBool( b));
            ctxt.Drive();
            assert_eq!( ctxt.GetValue( out), expected, "AND failed for {a} and {b}");
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_or_gate()
    {
        let  	mut ctxt = SimContext::New();
        let  	in1 = ctxt.AddTrigger( "in1", Reg::FALSE);
        let  	in2 = ctxt.AddTrigger( "in2", Reg::FALSE);
        let  	out = ctxt.AddTrigger( "out", Reg::FALSE);

        let  	_gate = OrGate::New( &mut ctxt, "Or", in1, in2, out);

        let  	truthTable = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::TRUE),
        ];

        for ( a, b, expected) in truthTable {
            ctxt.SetValue( in1, Reg::FromBool( a));
            ctxt.SetValue( in2, Reg::FromBool( b));
            ctxt.Drive();
            assert_eq!( ctxt.GetValue( out), expected, "OR failed for {a} and {b}");
        }
    }
    
    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_xor_gate()
    {
        let  	mut ctxt = SimContext::New();
        let  	in1 = ctxt.AddTrigger( "in1", Reg::FALSE);
        let  	in2 = ctxt.AddTrigger( "in2", Reg::FALSE);
        let  	out = ctxt.AddTrigger( "out", Reg::FALSE);

        let  	_gate = XorGate::New( &mut ctxt, "Xor", in1, in2, out);

        let  	truthTable = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::FALSE),
        ];

        for ( a, b, expected) in truthTable {
            ctxt.SetValue( in1, Reg::FromBool( a));
            ctxt.SetValue( in2, Reg::FromBool( b));
            ctxt.Drive();
            assert_eq!( ctxt.GetValue( out), expected, "XOR failed for {a} and {b}");
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

        let  	dynTriggers: &dyn ITriggerWad< U32> = &triggers;
        assert_eq!( dynTriggers.Len(), 2);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_triggers_gate_methods()
    {
        let  	mut triggers = TriggerWad::New();
        let  	idF = triggers.Add( "f", Reg::FALSE);
        let  	idT = triggers.Add( "t", Reg::TRUE);
        let  	idX = triggers.Add( "x", Reg::X);

        // NAND
        assert_eq!( triggers.Nand( idT, idT), Reg::FALSE);
        assert_eq!( triggers.Nand( idT, idF), Reg::TRUE);
        assert_eq!( triggers.Nand( idF, idF), Reg::TRUE);
        assert_eq!( triggers.Nand( idF, idX), Reg::TRUE);

        let  	dynBool: &dyn ITriggerWadBool = &triggers;
        assert_eq!( dynBool.Nand( idT, idT), Reg::FALSE);
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
        let  	mut ctxt = SimContext::New();
        let  	latch = RSLatch::New( &mut ctxt, "LatchTest");

        assert_eq!( latch.ReadQ( &ctxt), Reg::FALSE);
        assert_eq!( latch.ReadQ1( &ctxt), Reg::TRUE);
        assert_eq!( ctxt.GetValue( latch.S()), Reg::TRUE);
        assert_eq!( ctxt.GetValue( latch.R()), Reg::TRUE);

        let  	dynCtxt: &mut dyn ISimContext = &mut ctxt;
        let  	s = dynCtxt.AddTrigger( "sExtra", Reg::TRUE);
        assert_eq!( dynCtxt.GetValue( s), Reg::TRUE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rs_latch_set_and_hold()
    {
        let  	mut ctxt = SimContext::New();
        let  	latch = RSLatch::New( &mut ctxt, "LatchTest");

        // S=0, R=1 -> Set state: Q=1, Q1=0
        latch.Apply( &mut ctxt, Reg::FromBool( false), Reg::FromBool( true));
        assert_eq!( latch.ReadQ( &ctxt), Reg::TRUE, "Q should be 1 after Set");
        assert_eq!( latch.ReadQ1( &ctxt), Reg::FALSE, "Q1 should be 0 after Set");

        // S=1, R=1 -> Hold state: Q remains 1, Q1 remains 0
        latch.Apply( &mut ctxt, Reg::FromBool( true), Reg::FromBool( true));
        assert_eq!( latch.ReadQ( &ctxt), Reg::TRUE, "Q must hold 1");
        assert_eq!( latch.ReadQ1( &ctxt), Reg::FALSE, "Q1 must hold 0");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rs_latch_reset_and_hold()
    {
        let  	mut ctxt = SimContext::New();
        let  	latch = RSLatch::New( &mut ctxt, "LatchTest");

        // First Set to 1
        latch.Apply( &mut ctxt, Reg::FromBool( false), Reg::FromBool( true));
        assert_eq!( latch.ReadQ( &ctxt), Reg::TRUE);

        // S=1, R=0 -> Reset state: Q=0, Q1=1
        latch.Apply( &mut ctxt, Reg::FromBool( true), Reg::FromBool( false));
        assert_eq!( latch.ReadQ( &ctxt), Reg::FALSE, "Q should be 0 after Reset");
        assert_eq!( latch.ReadQ1( &ctxt), Reg::TRUE, "Q1 should be 1 after Reset");

        // S=1, R=1 -> Hold state: Q remains 0, Q1 remains 1
        latch.Apply( &mut ctxt, Reg::FromBool( true), Reg::FromBool( true));
        assert_eq!( latch.ReadQ( &ctxt), Reg::FALSE, "Q must hold 0");
        assert_eq!( latch.ReadQ1( &ctxt), Reg::TRUE, "Q1 must hold 1");
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    
    #[test]
    fn	test_rs_latch_sequence()
    {
        let  	mut ctxt = SimContext::New();
        let  	latch = RSLatch::New( &mut ctxt, "LatchTest");

        let  	sequence = [
            // ( S, R, expectedQ, expectedQ1)
            ( true, true, Reg::FALSE, Reg::TRUE),   // initial hold
            ( false, true, Reg::TRUE, Reg::FALSE),  // set
            ( true, true, Reg::TRUE, Reg::FALSE),   // hold
            ( true, false, Reg::FALSE, Reg::TRUE),  // reset
            ( true, true, Reg::FALSE, Reg::TRUE),   // hold
            ( false, true, Reg::TRUE, Reg::FALSE),  // set again
            ( true, true, Reg::TRUE, Reg::FALSE),   // hold
            ( false, false, Reg::TRUE, Reg::TRUE),  // active-low both active ( disallowed state in basic logic)
            ( true, false, Reg::FALSE, Reg::TRUE),  // recovery to reset
            ( true, true, Reg::FALSE, Reg::TRUE),   // hold
        ];

        for ( step, &( s, r, expQ, expQ1)) in sequence.iter().enumerate() {
            latch.Apply( &mut ctxt, Reg::FromBool( s), Reg::FromBool( r));
            assert_eq!(
                latch.ReadQ( &ctxt),
                expQ,
                "Step {}: Q mismatch for S={}, R={}",
                step,
                s,
                r
            );
            assert_eq!(
                latch.ReadQ1( &ctxt),
                expQ1,
                "Step {}: Q1 mismatch for S={}, R={}",
                step,
                s,
                r
            );
        }
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
        let  	port = Port::New( "clk", U32( 42));
        assert_eq!( port.Name(), "clk");
        assert_eq!( port.Trigger(), U32( 42));
    }
}

//---------------------------------------------------------------------------------------------------------------------------------