#!/bin/bash
# Run the Kleis REPL
#
# Currently uses the standalone 'repl' binary because 'kleis repl' is a stub.
# TODO: Once REPL is integrated into unified binary, change to:
#   exec ./target/release/kleis repl "$@"

cd "$(dirname "$0")/.."
exec ./target/release/repl "$@"
