mod web

tmux_session := "arto-dev"

[private]
default:
  @just --list

[private]
rust-setup:
  @rustup toolchain install stable
  @rustup target add wasm32-unknown-unknown

[private]
rust-fmt:
  @cargo fmt --all

[private]
rust-check:
  @cargo check --all-targets --all-features
  @cargo clippy --all-targets --all-features -- -D warnings

[private]
rust-dev:
  @dx serve

setup: web::setup rust-setup

fmt: web::fmt rust-fmt

check: web::check rust-check

test:
  @cargo test --all-features --all-targets

# Run dev servers in tmux split panes (left: web::dev, right: dx serve)
dev:
  #!/usr/bin/env bash
  set -euo pipefail
  # Kill existing session if present
  tmux kill-session -t "{{tmux_session}}" 2>/dev/null || true
  # Create new session with web::dev in the first pane
  tmux new-session -d -s "{{tmux_session}}" "just web::dev"
  # Split horizontally and run dx serve in the right pane
  tmux split-window -h -t "{{tmux_session}}" "just rust-dev"
  # Attach to the session
  tmux attach-session -t "{{tmux_session}}"

attach:
  # Attach to the session
  tmux attach-session -t "{{tmux_session}}"

build:
  @dx bundle --platform desktop \
    --package-types "macos" \
    --package-types "dmg"

install: build
  @cp -af target/dx/arto/bundle/macos/bundle/macos/Arto.app /Applications/.
