#!/bin/bash

# Install sqlx-cli
cargo install --version=0.7.3 sqlx-cli --no-default-features --features postgres

# Run migrations
sqlx migrate run

# import inno2grid data
cargo run -- import assets/inno2grid_all_data_cleaned_and_aligned.csv