#!/bin/sh
#
# Use this script to run your program LOCALLY.

set -e # Exit early if any commands fail

# - Edit this to change how your program compiles locally
(
  cd "$(dirname "$0")" # Ensure compile steps are run within the repository directory
  cargo build --release --target-dir=/tmp/redis-from-scratch-target --manifest-path Cargo.toml
)

# - Edit this to change how your program runs locally
exec /tmp/redis-from-scratch-target/release/redis-starter-rust "$@"
