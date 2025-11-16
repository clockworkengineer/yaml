//! Example demonstrating enhanced error handling features
//!
//! This example shows the new error handling capabilities:
//! - Error codes for programmatic handling
//! - Suggestions for fixing common mistakes
//! - Error recovery strategies
//! - Enhanced error context with spans
//! - Multiple error collection

use yaml_lib::{
    EnhancedError, ErrorCode, ErrorCollection, ErrorKind, ErrorSuggestion, ParserState,
    RecoveryContext, RecoveryHandler, RecoveryStrategy, Span, SuggestionBuilder, YamlError,
};

pub fn main() {
    println!("=== Enhanced YAML Error Handling Example ===\n");

    // Example 1: Using error codes
    demo_error_codes();

    // Example 2: Error suggestions
    demo_error_suggestions();

    // Example 3: Error recovery
    demo_error_recovery();

    // Example 4: Enhanced errors with context
    demo_enhanced_errors();

    // Example 5: Suggestion builder patterns
    demo_suggestion_builder();

    // Example 6: Recovery context management
    demo_recovery_context();

    // Example 7: Error collection
    demo_error_collection();
}

/// Demonstrates using error codes for programmatic error handling
fn demo_error_codes() {
    println!("--- Example 1: Error Codes ---");

    let error_codes = vec![
        ErrorCode::E001,
        ErrorCode::E002,
        ErrorCode::E004,
        ErrorCode::E007,
    ];

    for code in error_codes {
        println!("\n{}: {}", code, code.description());
        println!("  Can be used to handle: {}", get_recovery_action(code));
    }
    println!();
}

fn get_recovery_action(code: ErrorCode) -> &'static str {
    match code {
        ErrorCode::E001 => "Add missing colon, skip line, or insert default",
        ErrorCode::E002 => "Scan ahead to find closing quote or skip string",
        ErrorCode::E004 => "Use default value or skip reference",
        ErrorCode::E007 => "Adjust indentation or skip to next valid indent",
        _ => "Apply generic recovery strategy",
    }
}

/// Demonstrates error suggestions for common mistakes
fn demo_error_suggestions() {
    println!("--- Example 2: Error Suggestions ---");

    // Example: Boolean typo
    println!("\nScenario: User typed 'ture' instead of 'true'");
    let suggestion = SuggestionBuilder::boolean_typo("ture");
    println!("  Suggestion: {}", suggestion.message);
    if let Some(replacement) = &suggestion.replacement {
        println!("  Replacement: {}", replacement);
    }

    // Example: Null typo
    println!("\nScenario: User typed 'nil' instead of 'null'");
    let suggestion = SuggestionBuilder::null_typo("nil");
    println!("  Suggestion: {}", suggestion.message);
    if let Some(replacement) = &suggestion.replacement {
        println!("  Replacement: {}", replacement);
    }

    // Example: Undefined alias
    println!("\nScenario: User referenced undefined alias 'ancor1'");
    let available = vec!["anchor1".to_string(), "myanchor".to_string()];
    let suggestion = SuggestionBuilder::undefined_alias("ancor1", &available);
    println!("  Suggestion: {}", suggestion.message);

    // Example: Indentation error
    println!("\nScenario: Wrong indentation (found 6, expected 4 spaces)");
    let suggestion = SuggestionBuilder::indentation(4, 6);
    println!("  Suggestion: {}", suggestion.message);

    println!();
}

/// Demonstrates error recovery strategies
fn demo_error_recovery() {
    println!("--- Example 3: Error Recovery Strategies ---");

    // Create different recovery handlers
    let handlers = vec![
        ("Default", RecoveryHandler::new()),
        ("Strict (no recovery)", RecoveryHandler::strict()),
        ("Lenient (always recover)", RecoveryHandler::lenient()),
    ];

    let test_errors = vec![
        YamlError::new(ErrorKind::SyntaxError, "Missing colon in mapping"),
        YamlError::new(ErrorKind::UnterminatedString, "String not closed"),
        YamlError::new(ErrorKind::UndefinedAlias, "Alias not found"),
        YamlError::new(ErrorKind::UnexpectedEof, "Unexpected end of file"),
    ];

    for (name, handler) in handlers {
        println!("\n{} recovery handler:", name);
        for error in &test_errors {
            let strategy = handler.recover(error);
            println!("  {} -> {:?}", error.kind(), strategy);
        }
    }

    println!();
}

/// Demonstrates enhanced errors with rich context
fn demo_enhanced_errors() {
    println!("--- Example 4: Enhanced Errors with Context ---");

    // Create a base error
    let base_error = YamlError::new(ErrorKind::SyntaxError, "Missing colon in mapping")
        .with_position(5, 10);

    // Enhance with error code and suggestions
    let enhanced = EnhancedError::new(base_error)
        .with_code(ErrorCode::E001)
        .with_span(Span::new(5, 8, 5, 15))
        .with_snippet(
            r#"
    3 | name: John Doe
    4 | age: 30
    5 | address 123 Main St
              ^^^^ missing colon here
    6 | city: Springfield
"#,
        )
        .with_suggestion(
            ErrorSuggestion::new("Add ':' after key name")
                .with_replacement("address: 123 Main St")
                .with_span(Span::new(5, 8, 5, 23)),
        )
        .with_note("YAML mappings require 'key: value' pairs");

    // Display the enhanced error
    println!("\n{}", enhanced.format_detailed());

    // Demonstrate programmatic access
    println!("\nProgrammatic access:");
    if let Some(code) = enhanced.code() {
        println!("  Error code: {}", code);
    }
    println!("  Suggestions: {}", enhanced.suggestions().len());
    println!("  Notes: {}", enhanced.notes().len());

    println!();
}

/// Demonstrates suggestion builder patterns
fn demo_suggestion_builder() {
    println!("--- Example 5: Suggestion Builder Patterns ---");

    println!("\n1. Missing colon:");
    let suggestion = SuggestionBuilder::missing_colon("address");
    println!("   {}", suggestion.message);
    if let Some(repl) = &suggestion.replacement {
        println!("   Try: {}", repl);
    }

    println!("\n2. Unclosed quote:");
    let suggestion = SuggestionBuilder::unclosed_quote('"');
    println!("   {}", suggestion.message);

    println!("\n3. Boolean typo 'flase':");
    let suggestion = SuggestionBuilder::boolean_typo("flase");
    println!("   {}", suggestion.message);
    if let Some(repl) = &suggestion.replacement {
        println!("   Try: {}", repl);
    }

    println!("\n4. Alias with typo 'anchro':");
    let available = vec!["anchor".to_string(), "basenode".to_string()];
    let suggestion = SuggestionBuilder::undefined_alias("anchro", &available);
    println!("   {}", suggestion.message);

    println!("\n5. No available anchors:");
    let suggestion = SuggestionBuilder::undefined_alias("myalias", &vec![]);
    println!("   {}", suggestion.message);
    if let Some(repl) = &suggestion.replacement {
        println!("   Example: {}", repl);
    }

    println!();
}

/// Demonstrates recovery context management
fn demo_recovery_context() {
    println!("--- Example 6: Recovery Context Management ---");

    let mut ctx = RecoveryContext::new();
    println!("Initial state: {:?}", ctx.state);

    // Simulate parsing a block mapping
    println!("\nEntering block mapping...");
    ctx.enter_block_mapping();
    println!("  State: {:?}", ctx.state);
    println!("  Can recover with SkipLine: {}", ctx.can_recover(RecoveryStrategy::SkipLine));
    println!("  Can recover with SkipToNextMapping: {}", ctx.can_recover(RecoveryStrategy::SkipToNextMapping));

    // Simulate entering a flow mapping
    println!("\nEntering flow mapping...");
    ctx.enter_flow_mapping();
    println!("  State: {:?}", ctx.state);
    println!("  In flow context: {}", ctx.in_flow);
    println!("  Bracket depth: {}", ctx.bracket_depth);
    println!("  Can recover with SkipCollection: {}", ctx.can_recover(RecoveryStrategy::SkipCollection));

    // Exit flow
    println!("\nExiting flow...");
    ctx.exit_flow();
    println!("  In flow context: {}", ctx.in_flow);
    println!("  Bracket depth: {}", ctx.bracket_depth);

    // Test abort strategy
    println!("\nTesting abort strategy:");
    println!("  Can recover with Abort: {}", ctx.can_recover(RecoveryStrategy::Abort));

    println!();
}

/// Demonstrates collecting multiple errors
fn demo_error_collection() {
    println!("--- Example 7: Error Collection ---");

    let mut collection = ErrorCollection::new();
    println!("Created error collection");
    println!("  Continue on error: true");
    println!("  Max errors: 100");

    // Simulate collecting multiple errors
    println!("\nCollecting errors...");

    let error1 = EnhancedError::new(
        YamlError::new(ErrorKind::SyntaxError, "Missing colon").with_position(5, 10),
    )
    .with_code(ErrorCode::E001);

    collection.add(error1);
    println!("  Added error 1 (E001)");
    println!("  Total errors: {}", collection.len());
    println!("  Should continue: {}", collection.should_continue());

    let error2 = EnhancedError::new(
        YamlError::new(ErrorKind::UnterminatedString, "String not closed").with_position(8, 15),
    )
    .with_code(ErrorCode::E002);

    collection.add(error2);
    println!("\n  Added error 2 (E002)");
    println!("  Total errors: {}", collection.len());
    println!("  Should continue: {}", collection.should_continue());

    // Display all errors
    println!("\nAll collected errors:");
    for (i, error) in collection.errors().iter().enumerate() {
        println!("  {}. {} - {}", i + 1, error.code().unwrap(), error.base().message());
    }

    println!();
}
