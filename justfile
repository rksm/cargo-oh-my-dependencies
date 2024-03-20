set dotenv-load

export RUST_BACKTRACE := "1"
export RUST_LOG := "debug"

default:
    just --list

run:
    cargo run

dev:
    cargo watch -x run
