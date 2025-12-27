[private]
default:
  @just --list

[private]
setup:
  @cd renderer && pnpm install

fmt: setup
  @cd desktop && cargo fmt --all
  @cd renderer && pnpm run fmt

check: setup
  @cd renderer && pnpm run check
  @cd desktop && cargo check --all-targets --all-features
  @cd desktop && cargo clippy --all-targets --all-features -- -D warnings

test: setup
  @cd desktop && cargo test --all-features --all-targets

verify: fmt check test

# Run development server (dx serve handles everything via build.rs)
dev: setup
  @cd desktop && dx serve

build: setup
  @cd desktop && dx bundle --platform desktop \
    --package-types "macos" \
    --package-types "dmg"

install: build
  @cp -af desktop/target/dx/arto/bundle/macos/bundle/macos/Arto.app /Applications/.
