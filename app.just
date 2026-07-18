# app.just — skillport's build & dev commands.
#
# PROJECT-OWNED. The template-managed root `justfile` imports this
# (`import? 'app.just'`), so these run as `just build`, `just test`, … alongside
# the workflow commands (`status`, `new-spec`, …). Keep app recipes HERE so a
# template update never clobbers them. Mirrors AGENTS.md §6.
#
# NOTE: skillport is mid-build. The library substrate (parse/walk/rules/report)
# works and is tested; the real `lint` CLI is a pending STAGE-002 spec, so
# `main.rs` is still a stub. To see the library run today, use the example:
#   cargo run --example lint_demo -- <path>      (default path: lint-fixtures)

# --- build --------------------------------------------------------------------

# Dev build
build:
    cargo build

# Optimized release binary -> target/release/skillport
build-release:
    cargo build --release

# Fast type/borrow check, no codegen (all targets incl. tests/examples)
check:
    cargo check --all-targets

# Run the binary with args (currently a stub until the lint CLI spec lands)
run *ARGS:
    cargo run -- {{ARGS}}

# Install the binary into ~/.cargo/bin
install:
    cargo install --path .

# --- test ---------------------------------------------------------------------

# Full test suite (unit + integration + fixtures)
test:
    cargo test

# Run tests matching a name substring, e.g. `just test-one name.charset`
test-one NAME:
    cargo test {{NAME}}

# --- lint / format ------------------------------------------------------------

# Clippy with warnings denied (matches CI)
clippy:
    cargo clippy --all-targets -- -D warnings

# Format the code in place
fmt:
    cargo fmt

# Check formatting without writing (matches CI)
fmt-check:
    cargo fmt --check

# The full pre-ship gate: format check + clippy + tests
verify-all: fmt-check clippy test

# --- docs / deps / housekeeping ----------------------------------------------

# Build + open the API docs for eyeballing the substrate
doc:
    cargo doc --no-deps --open

# Show the dependency tree
tree:
    cargo tree

# Update dependencies within semver ranges (writes Cargo.lock)
update:
    cargo update

# Remove build artifacts (target/)
clean:
    cargo clean
