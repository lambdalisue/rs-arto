mod web

[private]
default:
  @just --list

setup: web::setup
  @rustup toolchain install stable
  @rustup target add wasm32-unknown-unknown
  @cargo binstall --no-confirm dioxus-cli

fmt: web::fmt
  @cargo fmt --all

check: web::check check-check check-clippy

[private]
check-check:
  @cargo check --all-targets --all-features

[private]
check-clippy:
  @cargo clippy --all-targets --all-features -- -D warnings

test:
  @cargo test --all-features --all-targets

[parallel]
dev: web::dev dev-dx

[private]
dev-dx:
  @dx serve

build:
  @dx bundle --platform desktop \
    --package-types "macos" \
    --package-types "dmg"

install: build
  @cp -af target/dx/arto/bundle/macos/bundle/macos/Arto.app /Applications/.
