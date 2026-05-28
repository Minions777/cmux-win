use vte::{Parser, Perform};

// Re-export the VT parser
// This module provides a thin wrapper around the vte crate
// for consistent API usage across the codebase.

pub fn create_parser() -> Parser {
    Parser::new()
}

// The actual parsing is done in state.rs via the Perform trait
// This module exists for potential future extensions:
// - Custom parser hooks
// - libghostty-vt integration
// - Parser performance optimizations
