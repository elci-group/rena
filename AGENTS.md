# AGENTS.md

## Project Overview
`rena` is a CLI utility for recursive and expressive file renaming, written in Rust. It supports pattern-based renaming (schemas) and AI-powered renaming (expressive/contextual modes) using Google's Gemini API.

## setup & Prerequisites
- **Language**: Rust (2021 edition)
- **Environment Variables**:
  - `GOOGLE_API_KEY`: Required for LLM features (`--contextual`, `--expressive`).

## Essential Commands

### Build & Run
- **Build**: `cargo build`
- **Run**: `cargo run -- [args]`
- **Release Build**: `cargo build --release`

### Testing
- **Test**: `cargo test`

### Common Usage Patterns
- **Dry Run**: By default, commands only show the plan. Use `--apply` to execute.
- **Schema Rename**: `cargo run -- -s "{orig}_{date}.{ext}"`
- **Recursive**: `cargo run -- -r -s ...`
- **AI Rename**: `cargo run -- --expressive` (requires API key)

## Code Structure

- **`src/main.rs`**: Entry point. Handles CLI argument parsing (`clap`) and orchestrates the flow.
- **`src/planner.rs`**: Core logic for generating and executing rename plans. Handles file walking and safety checks.
- **`src/schema.rs`**: Implementation of deterministic pattern-based renaming (hashing, dating, incrementing).
- **`src/llm.rs`**: Integration with Google Gemini API for intelligent renaming.
  - Model: `gemini-1.5-flash`
  - Modes:
    - `Contextual`: Renames based on filename lists (consistency).
    - `Expressive`: Renames based on file content analysis.

## Naming & Style
- **Rust Standard**: Follows standard Rust conventions (`snake_case` for functions/variables, `PascalCase` for types).
- **Error Handling**: Uses `anyhow` for error propagation.

## Key Gotchas
- **Dry-Run Safety**: The tool is designed to be safe. Always check the plan output before using `--apply`.
- **LLM Rate Limits**: Batch processing in `llm.rs` is chunked (5 files) to manage context/rate limits.
- **File Safety**: `planner.rs` includes checks to prevent overwriting existing target files.
