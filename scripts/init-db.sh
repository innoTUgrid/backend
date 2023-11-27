#!/bin/bash

# Install sqlx-cli
cargo install --version=0.7.3 sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run