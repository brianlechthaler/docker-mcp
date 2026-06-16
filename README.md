# docker-mcp

A secure [Model Context Protocol](https://modelcontextprotocol.io) server that exposes Docker operations to AI agents via stdio transport.

## Quick start

```bash
docker compose build docker-mcp
docker compose run --rm docker-mcp
```

Configure Cursor (or any MCP client) to run the server:

```json
{
  "mcpServers": {
    "docker": {
      "command": "docker",
      "args": ["compose", "-f", "/path/to/docker-mcp/compose.yaml", "run", "--rm", "docker-mcp"]
    }
  }
}
```

See [Getting started](docs/getting-started.md) for environment variables, scopes, and client setup.

## Documentation

- [Getting started](docs/getting-started.md)
- [Architecture](docs/architecture.md)
- [Security](docs/security.md)
- [Tools reference](docs/features/tools.md)
- [Deployment](docs/features/deployment.md)

## Requirements

- Docker Engine with API access (Unix socket or `DOCKER_HOST`)
- Docker Compose v2 for compose tools

## Development

```bash
make docker-test    # unit + integration tests (skips live Docker when unavailable)
make docker-lint    # rustfmt + clippy
make docker-coverage
```

## License

MIT
