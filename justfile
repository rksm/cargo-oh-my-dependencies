set dotenv-load

export RUST_BACKTRACE := "1"
export RUST_LOG := "debug"

default:
    just --list

run:
    cargo run

open-log-dir:
    emacsclient -n /home/robert/.local/share/oh-my-dependencies

open-log:
    emacsclient -n /home/robert/.local/share/oh-my-dependencies/oh-my-dependencies.log
