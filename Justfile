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

# Checks that every crate packages cleanly, listed in publication order.
package:
    cargo package --list --allow-dirty -p rsynko-download > /dev/null
    cargo package --list --allow-dirty -p rsynko-session > /dev/null
    cargo package --list --allow-dirty -p rsynko-x > /dev/null
    cargo package --list --allow-dirty -p rsynko-media > /dev/null
    cargo package --list --allow-dirty -p rsynko-manager > /dev/null
    cargo package --list --allow-dirty -p rsynko-rsync > /dev/null
    cargo package --list --allow-dirty -p rsynko-ui > /dev/null
    cargo package --list --allow-dirty -p rsynko-yt > /dev/null
    cargo package --list --allow-dirty -p rsynko-memory > /dev/null
    cargo package --list --allow-dirty -p rsynko-process > /dev/null
    cargo package --list --allow-dirty -p rsynko-reqwest > /dev/null
    cargo package --list --allow-dirty -p rsynko-ratatui > /dev/null
    cargo package --list --allow-dirty -p rsynko > /dev/null

# Runs the complete local quality gate.
ci: fmt build clippy doc test package

# Installs the `rsynko` executable onto the PATH, built with the release profile.
install:
    cargo install --path crates/rsynko

# Bumps and publishes one crate; level is patch, minor, or major.
release-crate package level:
    cargo release --package {{package}} {{level}} --execute

# The executable is released alone, because a push carrying more than three tags creates no event
# for the tag the binary release needs.
# Bumps and publishes every crate; level is patch, minor, or major.
release level:
    cargo release --workspace --exclude rsynko {{level}} --execute
    cargo release --package rsynko {{level}} --execute

# Removes build artifacts.
clean:
    cargo clean
