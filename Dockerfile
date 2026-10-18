# 3D web interface, served as static files by nginx. Build tools stay in the build stages.
#   docker build -t rubik-web .
#   docker run --rm -p 8080:80 rubik-web      -> http://localhost:8080

# 1. Rust engine -> WebAssembly package (wasm/pkg)
FROM rust:1.93-slim AS wasm
ARG WASM_PACK_VERSION=0.15.0
ARG TARGETARCH
RUN rustup target add wasm32-unknown-unknown
# Prebuilt wasm-pack: `cargo install` would take minutes on every cold build.
RUN arch=$([ "$TARGETARCH" = "arm64" ] && echo aarch64 || echo x86_64) \
    && name="wasm-pack-v${WASM_PACK_VERSION}-${arch}-unknown-linux-musl" \
    && apt-get update && apt-get install -y --no-install-recommends curl && rm -rf /var/lib/apt/lists/* \
    && curl -fsSL "https://github.com/wasm-bindgen/wasm-pack/releases/download/v${WASM_PACK_VERSION}/${name}.tar.gz" \
       | tar -xz -C /usr/local/bin --strip-components=1 "${name}/wasm-pack"
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY engine engine
COPY cli cli
COPY wasm wasm
RUN wasm-pack build wasm --release --target web --out-dir pkg

# 2. Web front -> static files (web/dist)
FROM node:24-alpine AS web
WORKDIR /src/web
COPY web/package.json web/package-lock.json ./
COPY --from=wasm /src/wasm/pkg /src/wasm/pkg
RUN npm ci
COPY web .
RUN npm test && npm run build

# 3. Serve
FROM nginx:1.29-alpine
# Compress JS/CSS/WASM: the three.js bundle drops from 1.2 MB to about 380 kB.
RUN printf 'gzip on;\ngzip_types application/javascript text/css application/wasm;\n' \
    > /etc/nginx/conf.d/gzip.conf
COPY --from=web /src/web/dist /usr/share/nginx/html
