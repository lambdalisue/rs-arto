mod web

[private]
default:
  @just --list

setup: web::setup

fmt: web::fmt

check: web::check
