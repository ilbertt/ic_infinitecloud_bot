#!/bin/bash

source .env

set -e

echo -e "\nBuilding canister..."

TELEGRAM_SECRET_TOKEN=$TELEGRAM_SECRET_TOKEN \
cargo build --target wasm32-unknown-unknown --release -p backend --locked

echo -e "\nDone!\n"
