
# ============== Builder stage ==============
# Using the latest Rust stable release as base image
FROM rust:latest as builder

# Switching the working directory toi `app` (equivalent to `cd app`)
# The `app` folder will be created by Docker in case it does not exist already.
WORKDIR /app

# Install the require system dependencies for linking configurations
RUN apt update && apt install lld clang -y

# Copy all files from working environment to Docker image
COPY . .

# Environment variables
ENV SQLX_OFFLINE true

# Building binary using the release profile to make it faaaast
RUN cargo build --release

# ============= Runtime Stage ================
FROM debian:bullseye-slim AS runtime

WORKDIR /app

# Install OpenSSL - it is dynamically linked by some of the dependencies
# Install ca-certificates - it is needed to verify TLS certificates \
# when establihsing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm-rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder environent to runtime environmnet
COPY --from=builder /app/target/release/email-newsletter-api email-newsletter-api

# Copy configuration file at runtime
COPY configuration/ configuration/

# Environment variables
ENV APP_ENVIRONMENT production


# When `docker run` is executed
ENTRYPOINT ["./target/release/email-newsletter-api"]