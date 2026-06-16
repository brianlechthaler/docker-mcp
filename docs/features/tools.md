# Tools reference

All tools are version `1` (suffix `@1` in inventory). Responses are JSON unless noted.

## Read scope (`docker:read`)

### `docker_system_info`

Docker daemon summary: ID, container/image counts, driver, OS, architecture.

### `docker_list_containers`

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `all` | bool | `false` | Include stopped containers |

### `docker_inspect_container`

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Container ID or name |

Returns full inspect JSON (may be truncated per `DOCKER_MCP_MAX_OUTPUT_BYTES`).

### `docker_container_logs`

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `id` | string | — | Container ID or name |
| `tail` | u64 | `100` | Lines to return (1–10000) |

### `docker_list_images`

Lists images with ID, tags, size, created timestamp.

### `docker_inspect_image`

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | string | Image name or ID |

### `docker_list_networks`

Lists networks (ID, name, driver).

### `docker_list_volumes`

Lists volumes (name, driver, mountpoint).

## Write scope (`docker:write`)

### `docker_start_container` / `docker_stop_container` / `docker_restart_container`

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `id` | string | — | Container ID or name |
| `timeout_secs` | u64 | `10` | Stop/restart grace period |

### `docker_remove_container`

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `id` | string | — | Container ID or name |
| `force` | bool | `false` | Force removal |

### `docker_run_container`

Runs a **non-privileged** container on bridge network.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `image` | string | — | Image reference |
| `name` | string | optional | Container name |
| `command` | string[] | `[]` | Command override |
| `env` | [key, value][] | `[]` | Environment variables |
| `detach` | bool | `true` | Detached mode |

### `docker_pull_image` / `docker_remove_image`

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `name` | string | — | Image reference |
| `force` | bool | `false` | Force remove (remove only) |

## Compose scope (`docker:compose`)

### `docker_compose_ps` / `docker_compose_config`

| Parameter | Type | Description |
|-----------|------|-------------|
| `compose_file` | string | Path to compose file (no `..` segments) |

`docker_compose_ps` returns `docker compose ps --format json` output.  
`docker_compose_config` returns validated/rendered compose YAML.

## Validation rules

- Container/image names: alphanumeric plus `-_.:/`, max 128 chars
- Compose paths: no `..`, max 4096 chars
- Command args: max 32 args, 512 chars each
- Env keys: `[A-Za-z0-9_]+` only

## Related

- [Getting started](../getting-started.md)
- [Security](../security.md)
