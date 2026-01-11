# Build the project
build:
    cargo build

# Build in release mode
release:
    cargo build --release

# Run the program with arguments
run *args:
    cargo run --release -- {{args}}

# Run tests
test:
    cargo test

# Check for compilation errors
check:
    cargo check

# Run the linter (clippy)
lint:
    cargo clippy

# Clean build artifacts
clean:
    cargo clean

# Format code
fmt:
    cargo fmt

# Show help
help:
    @just --list
