FROM rust:1.73-buster as builder

WORKDIR /app

# Accept the build argument
ARG DATABASE_URL
# Make sure to use the ARG in ENV
ENV DATABASE_URL=$DATABASE_URL

COPY . .

RUN cargo build --release

# Runtime stage: use a minimal debian image and only the compiled binary for the final image for running the application.
FROM debian:buster-slim
# Copies the compiled binary from the builder stage into the root of this minimal image.
COPY --from=builder /app/target/release/inno2grid-backend .
# Sets the user ID to a non-root user for security reasons.
#USER 1000
CMD ["./inno2grid-backend"]