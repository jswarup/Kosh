//-- console.rs ---------------------------------------------------------------------------------------------------------------------

use	std::sync::OnceLock;

//---------------------------------------------------------------------------------------------------------------------------------

static IS_CONSOLE_ENABLED: OnceLock< bool> = OnceLock::new();

#[inline]
pub fn	IsConsoleEnabled() -> bool
{
    return *IS_CONSOLE_ENABLED.get_or_init( || std::env::var( "KOSH_TEST_CONSOLE").is_ok());
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Unconditionally prints to stdout bypassing libtest capture if KOSH_TEST_CONSOLE is active,
/// otherwise falls back to standard captured println!.
#[macro_export]
macro_rules! cprintln {
    ( $($arg:tt)* ) => {
        if $crate::silo::console::IsConsoleEnabled() {
            use std::io::Write;
            let _ = std::writeln!( std::io::stdout(), $($arg)*);
        } else {
            std::println!( $($arg)*);
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

#[macro_export]
macro_rules! cprint {
    ( $($arg:tt)* ) => {
        if $crate::silo::console::IsConsoleEnabled() {
            use std::io::Write;
            let _ = std::write!( std::io::stdout(), $($arg)*);
            let _ = std::io::stdout().flush();
        } else {
            std::print!( $($arg)*);
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

/// Concise and attribute-friendly ConsoleTest macro.
///
/// Supported syntaxes:
/// - `ConsoleTest!( TestName { ... } );`
/// - `ConsoleTest!( TestName, { ... } );`
/// - `ConsoleTest!( TestName() { ... } );`
/// - `ConsoleTest!( fn TestName() { ... } );`
/// - `ConsoleTest!( $(#[$meta])* fn TestName() -> Result<()> { ... } );`
#[macro_export]
macro_rules! ConsoleTest {
    ( $name:ident $body:block ) => {
        #[test]
        fn	$name() {
            $body
        }
    };
    ( $name:ident, $body:block ) => {
        #[test]
        fn	$name() {
            $body
        }
    };
    ( $name:ident() $body:block ) => {
        #[test]
        fn	$name() {
            $body
        }
    };
    ( $name:ident() -> $ret:ty $body:block ) => {
        #[test]
        fn	$name() -> $ret {
            $body
        }
    };
    ( $(#[$meta:meta])* fn $name:ident() $(-> $ret:ty)? $body:block ) => {
        $(#[$meta])*
        #[test]
        fn	$name() $(-> $ret)? {
            $body
        }
    };
}

//---------------------------------------------------------------------------------------------------------------------------------

