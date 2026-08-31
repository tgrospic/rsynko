# Checks formatting without rewriting files.
fmt:
    cargo fmt --all -- --check

# Compiles every crate, feature, and target.
build:
    cargo build --workspace --all-features --all-targets

# Lints every target and rejects warnings.
clippy:
    cargo clippy --workspace --all-features --all-targets -- -D warnings

# Builds public documentation and rejects rustdoc warnings.
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps

# Runs unit, integration, and documentation tests.
test:
    cargo test --workspace --all-features

# Checks that every crate has a complete package surface.
package:
    cargo package --list --allow-dirty -p rsynko-media > /dev/null
    cargo package --list --allow-dirty -p rsynko-yt > /dev/null
    cargo package --list --allow-dirty -p rsynko-manager > /dev/null
    cargo package --list --allow-dirty -p rsynko-ui > /dev/null
    cargo package --list --allow-dirty -p rsynko-download > /dev/null
    cargo package --list --allow-dirty -p rsynko-memory > /dev/null
    cargo package --list --allow-dirty -p rsynko-rsync > /dev/null
    cargo package --list --allow-dirty -p rsynko-session > /dev/null
    cargo package --list --allow-dirty -p rsynko-x > /dev/null
    cargo package --list --allow-dirty -p rsynko-process > /dev/null
    cargo package --list --allow-dirty -p rsynko-reqwest > /dev/null
    cargo package --list --allow-dirty -p rsynko-ratatui > /dev/null
    cargo package --list --allow-dirty -p rsynko > /dev/null

# Runs the complete local quality gate.
ci: fmt build clippy doc test package

# Installs the `rsynko` executable onto the PATH, built with the release profile.
install:
    cargo install --path crates/rsynko
