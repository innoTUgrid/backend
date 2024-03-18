#!/bin/bash

# Install sqlx-cli
cargo install --version=0.7.4 sqlx-cli --locked --no-default-features --features postgres

# Run migrations
sqlx migrate run

# import inno2grid data (init only)
LOAD_INITIAL_DATA_PATH=assets/inno2grid_all_data_cleaned_and_aligned.meta.yaml cargo run --bin inno2grid-backend -- init
