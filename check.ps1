$ErrorActionPreference = "Stop"

cargo check --quiet --workspace --all-targets
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo check --quiet --workspace --all-features --lib --target wasm32-unknown-unknown
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo clippy --quiet --workspace --all-targets --all-features -- -D warnings -W clippy::all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo test --quiet --workspace --all-targets --all-features
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

cargo test --quiet --workspace --doc
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

trunk build
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
