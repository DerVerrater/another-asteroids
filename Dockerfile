FROM rust:1.89 AS builder

RUN apt-get update
RUN apt-get install -y --no-install-recommends libasound2-dev libudev-dev
RUN rustup target add wasm32-unknown-unknown
RUN cargo install --locked wasm-bindgen-cli

# Copy in only the parts we care about. This is to prevent Docker from re-
# running some steps because unimportant files changed (e.g.: the .git/ folder)
COPY src/ ./src
COPY Cargo.toml ./Cargo.toml
# WARN: The lockfile doesn't exist in the repo. You will have to create it
# before building the Docker image (i.e.: run `cargo update` first)
COPY Cargo.lock ./Cargo.lock
COPY www/ ./www
COPY Makefile ./Makefile

# Oops. There's no text output in the Docker build command line (it still works, though)
RUN make

FROM busybox:musl
RUN mkdir -p /var/www
COPY --from=builder ./out/ /var/www
WORKDIR /var/www

# TODO: Make httpd accept interrupt signals so the container exits properly.
# (<ctrl>+c and `docker stop ...` don't work because Busybox doesn't answer)
CMD ["httpd", "-f", "-p", "8080"]
