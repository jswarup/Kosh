//-- console.rs ---------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! _console_test_inner {
    ($d:tt) => {
        macro_rules! cprintln {
            ($d($d arg:tt)*) => {
                if std::env::var( "KOSH_TEST_CONSOLE").is_ok() {
                    use std::io::Write;
                    let _ = std::writeln!( std::io::stdout(), $d($d arg)*);
                } else {
                    std::println!( $d($d arg)*);
                }
            }
        }
        macro_rules! cprint {
            ($d($d arg:tt)*) => {
                if std::env::var( "KOSH_TEST_CONSOLE").is_ok() {
                    use std::io::Write;
                    let _ = std::write!( std::io::stdout(), $d($d arg)*);
                    let _ = std::io::stdout().flush();
                } else {
                    std::print!( $d($d arg)*);
                }
            }
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! ConsoleTest {
    ( fn $name:ident() $body:block ) => {
        #[test]
        fn $name() {
            $crate::_console_test_inner!($);
            $body
        }
    }
}

//---------------------------------------------------------------------------------------------------------------------------------
