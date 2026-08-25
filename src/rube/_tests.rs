//-- _tests.rs --------------------------------------------------------------------------------------------------------------------

//---------------------------------------------------------------------------------------------------------------------------------

#[cfg( test)]
mod _tests {
    use	crate::{
        rube::{
            adder::Adder,
            gates::{ AndGate, NandGate, NotGate, OrGate, XorGate },
            latches::{ CRSLatch, DLatch, RSLatch },
            reg::Reg,
            sim_context::SimContext,
            trigger::TriggerWad,
        },
        silo::U32,
    };

    #[test]
    fn	test_adder_basic()
    {
        let  	mut ctxt = SimContext::new();

        const N: usize = 16;
        let  	adder = Adder::< N>::New( &mut ctxt, "Adder16");

        let  	test_cases = [
            ( 0, 0, 0, false),
            ( 1, 1, 2, false),
            ( 5, 7, 12, false),
            ( 255, 255, 510, false),
            ( 0xFFFF, 1, 0, true), // 16-bit overflow
            ( 0x8000, 0x8000, 0, true),
            ( 12345, 54321, 66666, true), // 66666 is > 65535, so overflow
        ];

        for ( aVal, bVal, expected_sum, expected_carry) in test_cases {
            adder.SetA( &mut ctxt, U32( aVal));
            adder.SetB( &mut ctxt, U32( bVal));

            ctxt.drive();

            let  	sumVal = adder.GetSum( &ctxt);
            let  	carryVal = ctxt.get_value( adder.Carry()).is_true();

            // expected_sum is calculated as a 16-bit truncation
            let  	expected_truncated = expected_sum & 0xFFFF;
            let  	expected_overflow = expected_sum > 0xFFFF;

            assert_eq!(
                sumVal, expected_truncated,
                "Sum failed for {} + {}. Expected {}, got {}",
                aVal, bVal, expected_truncated, sumVal
            );
            assert_eq!(
                carryVal, expected_carry || expected_overflow,
                "Carry failed for {} + {}. Expected {}, got {}",
                aVal, bVal, expected_carry || expected_overflow, carryVal
            );
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_clocked_rs_latch()
    {
        let  	mut ctxt = SimContext::new();
        let  	crs = CRSLatch::new( &mut ctxt, "CRSLatch");

        // When clk=0, S and R changes have no effect
        ctxt.set_value( crs.clk(), Reg::FALSE);
        ctxt.set_value( crs.s(), Reg::TRUE);
        ctxt.set_value( crs.r(), Reg::FALSE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( crs.q()), Reg::FALSE);

        // Pulse clock high while S=1, R=0 -> Q becomes 1
        ctxt.set_value( crs.clk(), Reg::TRUE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( crs.q()), Reg::TRUE);
        assert_eq!( ctxt.get_value( crs.q1()), Reg::FALSE);

        // Clock back low -> retains Q=1
        ctxt.set_value( crs.clk(), Reg::FALSE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( crs.q()), Reg::TRUE);

        // Set R=1, S=0 with clk=0 -> still Q=1
        ctxt.set_value( crs.s(), Reg::FALSE);
        ctxt.set_value( crs.r(), Reg::TRUE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( crs.q()), Reg::TRUE);

        // Pulse clk=1 -> Q resets to 0
        ctxt.set_value( crs.clk(), Reg::TRUE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( crs.q()), Reg::FALSE);
        assert_eq!( ctxt.get_value( crs.q1()), Reg::TRUE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_d_latch()
    {
        let  	mut ctxt = SimContext::new();
        let  	d_latch = DLatch::new( &mut ctxt, "DLatch");

        // Enable = 0 ( latched): changing D has no effect
        ctxt.set_value( d_latch.e(), Reg::FALSE);
        ctxt.set_value( d_latch.d(), Reg::TRUE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( d_latch.q()), Reg::FALSE);

        // Enable = 1 ( transparent): D=1 passes to Q=1
        ctxt.set_value( d_latch.e(), Reg::TRUE);
        ctxt.set_value( d_latch.d(), Reg::TRUE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( d_latch.q()), Reg::TRUE);
        assert_eq!( ctxt.get_value( d_latch.q1()), Reg::FALSE);

        // D=0 passes to Q=0
        ctxt.set_value( d_latch.d(), Reg::FALSE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( d_latch.q()), Reg::FALSE);
        assert_eq!( ctxt.get_value( d_latch.q1()), Reg::TRUE);

        // Set D=1, then Enable=0 ( latch 1)
        ctxt.set_value( d_latch.d(), Reg::TRUE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( d_latch.q()), Reg::TRUE);

        ctxt.set_value( d_latch.e(), Reg::FALSE);
        ctxt.drive();

        // Now change D=0 while E=0 -> Q remains 1
        ctxt.set_value( d_latch.d(), Reg::FALSE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( d_latch.q()), Reg::TRUE, "Q should stay latched at 1");
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    #[test]
    fn	test_nand_gate()
    {
        let  	mut ctxt = SimContext::new();
        let  	in1 = ctxt.add_trigger( "in1", Reg::FALSE);
        let  	in2 = ctxt.add_trigger( "in2", Reg::FALSE);
        let  	out = ctxt.add_trigger( "out", Reg::TRUE);

        let  	_gate = NandGate::new( &mut ctxt, "Nand", in1, in2, out);

        let  	truth_table = [
            ( false, false, Reg::TRUE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::FALSE),
        ];

        for ( a, b, expected) in truth_table {
            ctxt.set_value( in1, Reg::from_bool( a));
            ctxt.set_value( in2, Reg::from_bool( b));
            ctxt.drive();
            assert_eq!( ctxt.get_value( out), expected, "NAND failed for {a} and {b}");
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    #[test]
    fn	test_not_gate()
    {
        let  	mut ctxt = SimContext::new();
        let  	in1 = ctxt.add_trigger( "in1", Reg::FALSE);
        let  	out = ctxt.add_trigger( "out", Reg::TRUE);

        let  	_gate = NotGate::new( &mut ctxt, "Not", in1, out);

        ctxt.set_value( in1, Reg::FALSE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( out), Reg::TRUE);

        ctxt.set_value( in1, Reg::TRUE);
        ctxt.drive();
        assert_eq!( ctxt.get_value( out), Reg::FALSE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    #[test]
    fn	test_and_gate()
    {
        let  	mut ctxt = SimContext::new();
        let  	in1 = ctxt.add_trigger( "in1", Reg::FALSE);
        let  	in2 = ctxt.add_trigger( "in2", Reg::FALSE);
        let  	out = ctxt.add_trigger( "out", Reg::FALSE);

        let  	_gate = AndGate::new( &mut ctxt, "And", in1, in2, out);

        let  	truth_table = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::FALSE),
            ( true, false, Reg::FALSE),
            ( true, true, Reg::TRUE),
        ];

        for ( a, b, expected) in truth_table {
            ctxt.set_value( in1, Reg::from_bool( a));
            ctxt.set_value( in2, Reg::from_bool( b));
            ctxt.drive();
            assert_eq!( ctxt.get_value( out), expected, "AND failed for {a} and {b}");
        }
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    #[test]
    fn	test_or_gate()
    {
        let  	mut ctxt = SimContext::new();
        let  	in1 = ctxt.add_trigger( "in1", Reg::FALSE);
        let  	in2 = ctxt.add_trigger( "in2", Reg::FALSE);
        let  	out = ctxt.add_trigger( "out", Reg::FALSE);

        let  	_gate = OrGate::new( &mut ctxt, "Or", in1, in2, out);

        let  	truth_table = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::TRUE),
        ];

        for ( a, b, expected) in truth_table {
            ctxt.set_value( in1, Reg::from_bool( a));
            ctxt.set_value( in2, Reg::from_bool( b));
            ctxt.drive();
            assert_eq!( ctxt.get_value( out), expected, "OR failed for {a} and {b}");
        }
    }
    
    //-----------------------------------------------------------------------------------------------------------------------------
    #[test]
    fn	test_xor_gate()
    {
        let  	mut ctxt = SimContext::new();
        let  	in1 = ctxt.add_trigger( "in1", Reg::FALSE);
        let  	in2 = ctxt.add_trigger( "in2", Reg::FALSE);
        let  	out = ctxt.add_trigger( "out", Reg::FALSE);

        let  	_gate = XorGate::new( &mut ctxt, "Xor", in1, in2, out);

        let  	truth_table = [
            ( false, false, Reg::FALSE),
            ( false, true, Reg::TRUE),
            ( true, false, Reg::TRUE),
            ( true, true, Reg::FALSE),
        ];

        for ( a, b, expected) in truth_table {
            ctxt.set_value( in1, Reg::from_bool( a));
            ctxt.set_value( in2, Reg::from_bool( b));
            ctxt.drive();
            assert_eq!( ctxt.get_value( out), expected, "XOR failed for {a} and {b}");
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
    fn	test_triggers_gate_methods()
    {
        let  	mut triggers = TriggerWad::new();
        let  	id_f = triggers.add( "f", Reg::FALSE);
        let  	id_t = triggers.add( "t", Reg::TRUE);
        let  	id_x = triggers.add( "x", Reg::X);

        // NAND
        assert_eq!( triggers.nand( id_t, id_t), Reg::FALSE);
        assert_eq!( triggers.nand( id_t, id_f), Reg::TRUE);
        assert_eq!( triggers.nand( id_f, id_f), Reg::TRUE);
        assert_eq!( triggers.nand( id_f, id_x), Reg::TRUE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    #[test]
    fn	test_triggers_edges()
    {
        let  	mut triggers = TriggerWad::new();
        // Trigger 0: False -> False
        let  	s0 = triggers.add( "s0", Reg::FALSE);
        triggers.set_future_value( s0, Reg::FALSE);
        triggers.advance( s0);

        // Trigger 1: False -> True
        let  	s1 = triggers.add( "s1", Reg::FALSE);
        triggers.set_future_value( s1, Reg::TRUE);
        triggers.advance( s1);

        // Trigger 2: True -> False
        let  	s2 = triggers.add( "s2", Reg::TRUE);
        triggers.set_future_value( s2, Reg::FALSE);
        triggers.advance( s2);

        assert!( !triggers.is_edge( s0));

        assert!( triggers.is_edge( s1));
        assert!( triggers.is_posedge( s1));
        assert!( !triggers.is_negedge( s1));

        assert!( triggers.is_edge( s2));
        assert!( !triggers.is_posedge( s2));
        assert!( triggers.is_negedge( s2));
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    #[test]
    fn	test_rs_latch_initialization()
    {
        let  	mut ctxt = SimContext::new();
        let  	latch = RSLatch::new( &mut ctxt, "LatchTest");

        assert_eq!( latch.read_q( &ctxt), Reg::FALSE);
        assert_eq!( latch.read_q1( &ctxt), Reg::TRUE);
        assert_eq!( ctxt.get_value( latch.s()), Reg::TRUE);
        assert_eq!( ctxt.get_value( latch.r()), Reg::TRUE);
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    #[test]
    fn	test_rs_latch_set_and_hold()
    {
        let  	mut ctxt = SimContext::new();
        let  	latch = RSLatch::new( &mut ctxt, "LatchTest");

        // S=0, R=1 -> Set state: Q=1, Q1=0
        latch.apply( &mut ctxt, Reg::from_bool( false), Reg::from_bool( true));
        assert_eq!( latch.read_q( &ctxt), Reg::TRUE, "Q should be 1 after Set");
        assert_eq!( latch.read_q1( &ctxt), Reg::FALSE, "Q1 should be 0 after Set");

        // S=1, R=1 -> Hold state: Q remains 1, Q1 remains 0
        latch.apply( &mut ctxt, Reg::from_bool( true), Reg::from_bool( true));
        assert_eq!( latch.read_q( &ctxt), Reg::TRUE, "Q must hold 1");
        assert_eq!( latch.read_q1( &ctxt), Reg::FALSE, "Q1 must hold 0");
    }

    //-----------------------------------------------------------------------------------------------------------------------------

    #[test]
    fn	test_rs_latch_reset_and_hold()
    {
        let  	mut ctxt = SimContext::new();
        let  	latch = RSLatch::new( &mut ctxt, "LatchTest");

        // First Set to 1
        latch.apply( &mut ctxt, Reg::from_bool( false), Reg::from_bool( true));
        assert_eq!( latch.read_q( &ctxt), Reg::TRUE);

        // S=1, R=0 -> Reset state: Q=0, Q1=1
        latch.apply( &mut ctxt, Reg::from_bool( true), Reg::from_bool( false));
        assert_eq!( latch.read_q( &ctxt), Reg::FALSE, "Q should be 0 after Reset");
        assert_eq!( latch.read_q1( &ctxt), Reg::TRUE, "Q1 should be 1 after Reset");

        // S=1, R=1 -> Hold state: Q remains 0, Q1 remains 1
        latch.apply( &mut ctxt, Reg::from_bool( true), Reg::from_bool( true));
        assert_eq!( latch.read_q( &ctxt), Reg::FALSE, "Q must hold 0");
        assert_eq!( latch.read_q1( &ctxt), Reg::TRUE, "Q1 must hold 1");
    }

    //-----------------------------------------------------------------------------------------------------------------------------
    
    #[test]
    fn	test_rs_latch_sequence()
    {
        let  	mut ctxt = SimContext::new();
        let  	latch = RSLatch::new( &mut ctxt, "LatchTest");

        let  	sequence = [
            // ( S, R, expected_Q, expected_Q1)
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

        for ( step, &( s, r, exp_q, exp_q1)) in sequence.iter().enumerate() {
            latch.apply( &mut ctxt, Reg::from_bool( s), Reg::from_bool( r));
            assert_eq!(
                latch.read_q( &ctxt),
                exp_q,
                "Step {}: Q mismatch for S={}, R={}",
                step,
                s,
                r
            );
            assert_eq!(
                latch.read_q1( &ctxt),
                exp_q1,
                "Step {}: Q1 mismatch for S={}, R={}",
                step,
                s,
                r
            );
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------