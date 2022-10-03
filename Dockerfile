# FROM rust:1 as builder
# 
# WORKDIR /app
# 
# COPY . /app
# 
# RUN cargo build --release

# For the bot executor
FROM centos:stream9

# We are doing podman-alongside-podman with podman remote
# Just use podman remote instead of trying to nest containers
RUN dnf install -y podman-remote

# Clean out dnf caches to save space
RUN rm -rf /var/cache /var/log/dnf* /var/log/yum.*

RUN useradd ferris; \
echo ferris:10000:5000 > /etc/subuid; \
echo ferris:10000:5000 > /etc/subgid;


# Need this environment variable to tell ferris-bot it's inside a container
# This tells the bot to use podman-remote instead of podman
ENV IS_RUNNING_IN_CONTAINER="true"
ENV CONTAINER_HOST="unix:/run/podman/podman.sock"

# For local development
COPY ./target/release/ferris-bot /app/ferris-bot
# For using the builder image
# COPY --from=builder /app/target/release/ferris-bot /app/ferris-bot

USER ferris;

ENTRYPOINT ["/app/ferris-bot"]