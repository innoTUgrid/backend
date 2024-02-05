FROM rust:1.73-buster as builder

WORKDIR /app

# Accept the build argument
ARG DATABASE_URL
# Make sure to use the ARG in ENV
ENV DATABASE_URL=$DATABASE_URL

COPY . .

RUN ./scripts/init-db.sh
RUN cargo build --release

# Runtime stage: use a minimal debian image and only the compiled binary for the final image for running the application.
FROM debian:bookworm-slim
WORKDIR /app
# Copies the compiled binary from the builder stage into the root of this minimal image.
COPY --from=builder /app/target/release/inno2grid-backend .

ENV RUN_MIGRATIONS=true
ENV LOAD_INITIAL_DATA_PATH=assets/inno2grid_all_data_cleaned_and_aligned.meta.yaml

COPY migrations migrations
COPY assets assets
# Sets the user ID to a non-root user for security reasons.
#USER 1000
CMD ["./inno2grid-backend"]