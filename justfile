set dotenv-load

export RUST_BACKTRACE := "1"
export RUST_LOG := "debug"

default:
    just --list

run *args="":
    cargo run -- {{ args }}

ex-metadata:
    cargo run --example metadata

open-log-dir:
    emacsclient -n /home/robert/.local/share/oh-my-dependencies

open-log:
    #!/usr/bin/env bash
    set -e
    if [[ $(uname) == "Darwin" ]]; then
        emacsclient -n "$HOME/Library/Application Support/hn.kra.cargo-oh-my-dependencies/cargo_oh_my_dependencies.log"
    else
        emacsclient -n $HOME/.local/share/oh-my-dependencies/oh-my-dependencies.log
    fi
