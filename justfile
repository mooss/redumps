# Build the project
build:
    cargo build

# Build in release mode
release:
    RUSTFLAGS='-C target-cpu=native' cargo build --release

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

# Profile with perf (record)
profile *args:
    cargo build --profile profiling
    perf record -g --call-graph dwarf -o perf.data target/profiling/redumps {{args}}
    ./scripts/generate-profiling-graphs.sh

# Show help
help:
    @just --list
