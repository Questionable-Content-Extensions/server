##################
#   Rust image   #
##################

FROM rust:trixie AS chef 
WORKDIR /qcext-server
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-chef 

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS rust
# Run SQLX in offline mode
ENV SQLX_OFFLINE=true

# Build dependencies - this is the caching Docker layer!
COPY --from=planner /qcext-server/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Dependencies are now cached, build for real.
COPY . .
RUN mkdir bindings
RUN rm -f ./bindings/*.ts; cargo test --release export_
RUN cargo build --release

##################
#  NodeJS image  #
##################

FROM debian:trixie AS nodejs

WORKDIR /qcext-server

# Download and import the Nodesource GPG key
RUN apt-get update && \
    apt-get install -y ca-certificates curl gnupg && \
    apt-get clean
RUN mkdir -p /etc/apt/keyrings && \
    curl -fsSL https://deb.nodesource.com/gpgkey/nodesource-repo.gpg.key | \
    gpg --dearmor -o /etc/apt/keyrings/nodesource.gpg

# Make sure we have npm and nodejs
ENV NODE_MAJOR=24
RUN echo "deb [signed-by=/etc/apt/keyrings/nodesource.gpg] https://deb.nodesource.com/node_$NODE_MAJOR.x nodistro main" | tee /etc/apt/sources.list.d/nodesource.list
RUN DEBIAN_FRONTEND=noninteractive \
    apt-get update && \
    apt-get install nodejs -y && \
    apt-get clean


# Next, let's run npm install
COPY package.json package-lock.json /qcext-server/
RUN npm install
COPY index.html tsconfig.json vite.config.ts /qcext-server/

# Copy only files relevant for Node (i.e. no Rust files)
RUN mkdir /qcext-server/src 
COPY src/client /qcext-server/src/client
COPY src/index.tsx src/vite-env.d.ts /qcext-server/src/

RUN npx update-browserslist-db@latest

COPY public /qcext-server/public
COPY services /qcext-server/services
RUN mkdir bindings
COPY --from=rust /qcext-server/bindings ./bindings
RUN npm run build

##################
#  Output image  #
##################

FROM gcr.io/distroless/cc-debian13

COPY --from=rust /qcext-server/target/release/qcext-server /usr/local/bin/
COPY --from=nodejs /qcext-server/build /build

ENV PORT=80
EXPOSE 80
HEALTHCHECK --interval=30s --timeout=5s \
    CMD wget --quiet --spider http://localhost/ || exit 1

ENV RUST_LOG=info
CMD ["/usr/local/bin/qcext-server"]
