#!/bin/bash
set -e

git submodule update --remote

cargo build -p ffi

cargo fmt

cargo test
