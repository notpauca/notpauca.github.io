#!/usr/bin/env bash
check() {
  if ! command -v $1; then
    printf "Can't find %s\n", $1
    exit 1
  fi
}

check openssl
openssl req -newkey rsa:2048 -x509 -days 365 -nodes -out cert.pem -keyout cert.pem