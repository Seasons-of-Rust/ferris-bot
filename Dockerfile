# --- Build ---
FROM rust:1 as builder

WORKDIR /app

COPY . /app

RUN cargo build --release

# --- Executor ---
FROM quay.io/podman/stable:latest

# Commenting this out as the current podman design does not face any additional
# restrictrions when running in a container
# TODO: see how well this works long-term, then drop the "am i in a container"
# logic
#ENV IS_RUNNING_IN_CONTAINER="true"
#ENV CONTAINER_HOST="unix:/run/podman/podman.sock"

COPY --from=builder /app/target/release/ferris-bot /app/ferris-bot

USER podman

ENTRYPOINT ["/app/ferris-bot"]
