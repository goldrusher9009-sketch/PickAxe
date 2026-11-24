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
# Set PICKAXE_TOKEN for a private server, or PICKAXE_PUBLIC=1 for a token-free restricted demo.
CMD ["sh", "-c", "if [ -n \"$PICKAXE_PUBLIC\" ]; then exec pickaxe-cli mcp --http 0.0.0.0:${PORT} --public; else test -n \"$PICKAXE_TOKEN\" && exec pickaxe-cli mcp --http 0.0.0.0:${PORT} --token \"$PICKAXE_TOKEN\"; fi"]
