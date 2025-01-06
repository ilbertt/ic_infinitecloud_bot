#!/bin/bash

TELEGRAM_SECRET_TOKEN=test-secret-token \
cargo clippy --all-targets --all-features --workspace -- -Dwarnings
