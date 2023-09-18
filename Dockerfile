##################
#   Rust image   #
##################

FROM rust:1-bookworm AS chef 
WORKDIR /usr/src/qcext-server
RUN cargo install cargo-chef 

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS rust
# Run SQLX in offline mode
ENV SQLX_OFFLINE=true

# Build dependencies - this is the caching Docker layer!
COPY --from=planner /usr/src/qcext-server/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Dependencies are now cached, build for real.
COPY . .
RUN cargo build --release

##################
#  NodeJS image  #
##################

FROM debian:bookworm AS nodejs

WORKDIR /usr/src/qcext-server

# Download and import the Nodesource GPG key
RUN apt-get update && \
    apt-get install -y ca-certificates curl gnupg && \
    apt-get clean
RUN mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | \
    gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg

# Make sure we have npm and nodejs
ENV NODE_MAJOR=18
RUN echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
RUN DEBIAN_FRONTEND=noninteractive \
    apt-get update && \
    apt-get install nodejs -y && \
    apt-get clean


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

FROM gcr.io/distroless/cc-debian12

COPY --from=rust /usr/src/qcext-server/target/release/qcext-server /usr/local/bin/
COPY --from=nodejs /usr/src/qcext-server/build /build

ENV RUST_LOG=info
CMD ["/usr/local/bin/qcext-server"]
