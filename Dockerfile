##################
#   Rust image   #
##################

FROM rust:latest AS rust

WORKDIR /usr/src/qcext-server

# Run SQLX in offline mode
ENV SQLX_OFFLINE=true

# Use spare registry protocol
ENV CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse

# Build the dependencies in a separate step to avoid rebuilding all of them
# every time the source code changes. This takes advantage of Docker's layer
# caching, and it works by doing a build using the Cargo.{toml,lock} files with
# dummy source code.
COPY Cargo.lock Cargo.toml /usr/src/qcext-server/
RUN mkdir -p /usr/src/qcext-server/database
COPY database/Cargo.toml database/sqlx-data.json /usr/src/qcext-server/database/
RUN mkdir -p /usr/src/qcext-server/shared
COPY shared/Cargo.toml /usr/src/qcext-server/shared/
RUN mkdir -p /usr/src/qcext-server/src && \
    echo "fn main() {}" > /usr/src/qcext-server/src/main.rs
RUN mkdir -p /usr/src/qcext-server/database/src && \
    touch /usr/src/qcext-server/database/src/lib.rs
RUN mkdir -p /usr/src/qcext-server/shared/src && \
    touch /usr/src/qcext-server/shared/src/lib.rs
RUN cargo fetch
RUN cargo build --release

# Dependencies are now cached, copy the actual source code and do another full
# build. The touch on all the .rs files is needed, otherwise cargo assumes the
# source code didn't change thanks to mtime weirdness.
RUN rm -rf /usr/src/qcext-server/src /usr/src/qcext-server/database/src /usr/src/qcext-server/shared/src
COPY src /usr/src/qcext-server/src
RUN rm -rf /usr/src/qcext-server/src/client
COPY database/src /usr/src/qcext-server/database/src
COPY shared/src /usr/src/qcext-server/shared/src
RUN find src -name "*.rs" -exec touch {} \; && \
    find database/src -name "*.rs" -exec touch {} \; && \
    find shared/src -name "*.rs" -exec touch {} \; && \
    cargo build --release

##################
#  NodeJS image  #
##################

FROM debian:bullseye AS nodejs

WORKDIR /usr/src/qcext-server

# Make sure we have npm and nodejs
RUN apt-get update
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y curl
RUN curl -fsSL https://deb.nodesource.com/setup_lts.x | bash -
RUN DEBIAN_FRONTEND=noninteractive apt-get install -y nodejs
RUN node --version
RUN npm --version

# Next, let's run npm install
COPY package.json package-lock.json /usr/src/qcext-server/
RUN npm install

# Copy only files relevant for Node (i.e. no Rust files)
RUN mkdir /usr/src/qcext-server/src 
COPY src/client /usr/src/qcext-server/src/client
COPY src/index.js /usr/src/qcext-server/src

RUN npx browserslist@latest --update-db

COPY public /usr/src/qcext-server/public
RUN npm run build

##################
#  Output image  #
##################

FROM debian:bullseye-slim AS binary

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y \
    ca-certificates rsync
RUN apt-get clean

COPY --from=rust /usr/src/qcext-server/target/release/qcext-server /usr/local/bin/
COPY --from=nodejs /usr/src/qcext-server/build /build

ENV RUST_LOG=info
CMD ["/usr/local/bin/qcext-server"]
