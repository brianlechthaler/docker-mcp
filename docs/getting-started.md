# Getting started

## Run the server

### Docker (recommended)

```bash
git clone https://github.com/brianlechthaler/docker-mcp.git
cd docker-mcp
docker compose build docker-mcp
docker compose run --rm docker-mcp
```

The server speaks MCP over stdio. Mount the Docker socket read-only (default in `compose.yaml`).

### Local build (requires Rust toolchain)

```bash
cargo build --release
DOCKER_MCP_SCOPES=docker:read,docker:write,docker:compose ./target/release/docker-mcp
```

For day-to-day development, use the containerized workflow in the README.

## Environment variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DOCKER_MCP_SCOPES` | `docker:read,docker:write,docker:compose` | Comma-separated scopes granted to this server instance |
| `DOCKER_MCP_CALLER_IDENTITY` | `unknown-agent` | Identity recorded in audit logs |
| `DOCKER_MCP_SESSION_ID` | random UUID | Correlation ID for audit events |
| `DOCKER_MCP_MAX_OUTPUT_BYTES` | `1048576` | Maximum tool response size before truncation |
| `DOCKER_MCP_AUDIT_JSON` | `true` | Emit structured JSON audit lines to stderr |
| `DOCKER_HOST` | local socket | Docker API endpoint |

## Scopes

| Scope | Capability |
|-------|------------|
| `docker:read` | List and inspect containers, images, networks, volumes; fetch logs |
| `docker:write` | Start, stop, restart, remove, run containers; pull/remove images |
| `docker:compose` | `docker compose ps` and `docker compose config` |

Grant only the scopes your agent needs. The server fails closed when a tool requires a missing scope.

## Cursor configuration

Add to `.cursor/mcp.json` (paths adjusted to your checkout):

```json
{
  "mcpServers": {
    "docker": {
      "command": "docker",
      "args": [
        "compose",
        "-f",
        "/absolute/path/to/docker-mcp/compose.yaml",
        "run",
        "--rm",
        "-T",
        "docker-mcp"
      ]
    }
  }
}
```

Use `-T` to disable TTY allocation for stdio MCP.

## Verify

With the server running, your MCP client should list 17 tools (see [Tools reference](features/tools.md)). A quick smoke test from another terminal:

```bash
docker compose run --rm dev cargo test
```

## Troubleshooting

**Permission denied on Docker socket** — add your user to the `docker` group or run the MCP container with a socket mount the container user can read.

**Scope denied errors** — set `DOCKER_MCP_SCOPES` to include the scope required by the tool (see error message).

**Compose tools fail** — ensure `docker compose` is available in the runtime image and the compose file path exists on the host filesystem visible to the container.
