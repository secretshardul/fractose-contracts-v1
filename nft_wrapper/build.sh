#!/bin/bash
set -e

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
mkdir -p ../../out
cp target/wasm32-unknown-unknown/release/*.wasm ../../out/main.wasm

mkdir ./res
cp ../../out/main.wasm ./res/nft_wrapper.wasm
