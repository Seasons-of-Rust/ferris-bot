FROM rust:1 as builder

WORKDIR /app

COPY . /app

RUN cargo build --release

# For the bot executor
FROM quay.io/podman/stable:latest

# Need this environment variable to tell ferris-bot it's inside a container
# This tells the bot to use podman-remote instead of podman
#ENV IS_RUNNING_IN_CONTAINER="true"
#ENV CONTAINER_HOST="unix:/run/podman/podman.sock"

# For local development
#COPY ./target/release/ferris-bot /app/ferris-bot
# For using the builder image
COPY --from=builder /app/target/release/ferris-bot /app/ferris-bot

ENTRYPOINT ["/app/ferris-bot"]