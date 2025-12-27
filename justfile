[private]
default:
  @just --list

setup:
  @cd web && pnpm install

fmt:
  @cargo fmt --all
  @cd web && pnpm run fmt

check:
  @cd web && pnpm run check
  @cargo check --all-targets --all-features
  @cargo clippy --all-targets --all-features -- -D warnings

test:
  @cargo test --all-features --all-targets

# Run development server (dx serve handles everything via build.rs)
dev:
  @dx serve

build:
  @dx bundle --platform desktop \
    --package-types "macos" \
    --package-types "dmg"

install: build
  @cp -af target/dx/arto/bundle/macos/bundle/macos/Arto.app /Applications/.
