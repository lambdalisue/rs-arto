[private]
default:
  @just --list

setup:
  @cd renderer && pnpm install

fmt:
  @cd desktop && cargo fmt --all
  @cd renderer && pnpm run fmt

check:
  @cd renderer && pnpm run check
  @cd desktop && cargo check --all-targets --all-features
  @cd desktop && cargo clippy --all-targets --all-features -- -D warnings

test:
  @cd desktop && cargo test --all-features --all-targets

# Run development server (dx serve handles everything via build.rs)
dev:
  @cd desktop && dx serve

build:
  @cd desktop && dx bundle --platform desktop \
    --package-types "macos" \
    --package-types "dmg"

install: build
  @cp -af desktop/target/dx/arto/bundle/macos/bundle/macos/Arto.app /Applications/.
