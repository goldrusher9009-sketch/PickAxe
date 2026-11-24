# Hosted Pickaxe MCP server (restricted mode, bearer token). Untested build: run it once before relying on it.
FROM rust:1-bookworm AS build
WORKDIR /src
COPY . .
RUN cargo build --release -p pickaxe-cli

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/pickaxe-cli /usr/local/bin/pickaxe-cli
ENV PORT=8787
EXPOSE 8787
# PICKAXE_TOKEN must be set in the host's environment; the server refuses requests without it.
CMD ["sh", "-c", "test -n \"$PICKAXE_TOKEN\" && exec pickaxe-cli mcp --http 0.0.0.0:${PORT} --token \"$PICKAXE_TOKEN\""]
