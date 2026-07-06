#!/usr/bin/env bash

#TODO: watching for changes in dir/some form of hot reloading?

check() {
  if ! command -v $1; then
    printf "Can't find %s\n", $1
    exit 1
  fi
}

check wasm-pack
check python3

wasm-pack build --dev --target web && ./run.py ip=127.0.0.1 port=8000