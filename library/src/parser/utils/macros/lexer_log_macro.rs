#[macro_export]
macro_rules! lexer_debug {
	($($arg:tt)*) => {
		#[cfg(feature = "debug-trace")]
		{
			$crate::parser::lexer::lexer_log(format!($($arg)*));
		}
	};
}

/*
 * Lexer Debug Logging Macro
 *
 * Provides a macro for centralized debug logging in the YAML lexer, wrapping conditional
 * logging for token emission and debug output. Enabled only when the `debug-trace` feature is active.
 *
 * Copyright (c) 2026 YAML Library Developers
 */


