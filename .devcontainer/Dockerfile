ARG TARGETARCH
ARG RUST_VERSION=1.88
ARG YQ_VERSION=4.46.1

FROM mikefarah/yq:$YQ_VERSION AS yq
FROM mcr.microsoft.com/devcontainers/base:ubuntu-24.04

# Include lld linker to improve build times either by using environment variable
# RUSTFLAGS="-C link-arg=-fuse-ld=lld" or with Cargo's configuration file (i.e see .cargo/config.toml).
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
   && apt-get -y install clang lld libsqlite3-dev libmariadb-dev-compat libmariadb-dev libpq-dev pkg-config gh git-delta iputils-ping yq jsonnet cmake libtirpc-dev \
   && apt-get autoremove -y && apt-get clean -y

COPY --from=yq /usr/bin/yq /usr/local/bin/yq

USER vscode

ARG RUST_VERSION
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain ${RUST_VERSION} --no-modify-path -y

RUN /home/vscode/.cargo/bin/cargo install cargo-release
RUN /home/vscode/.cargo/bin/cargo install --locked cargo-outdated
