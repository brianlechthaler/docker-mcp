# Security

docker-mcp follows [NSA CSI MCP security design considerations](https://www.nsa.gov/Portals/75/documents/Cybersecurity/CSI_MCP_SECURITY.pdf) for agentic automation.

## Threat model

| Trust boundary | Participants | Risk |
|----------------|--------------|------|
| Agent → gateway | MCP client, optional proxy | Prompt injection, tool abuse |
| Gateway → server | docker-mcp over stdio | Unauthorized operations, data exfiltration |
| Server → Docker | Docker Engine API | Container escape, host compromise |

**Data classes:** container metadata, logs (may contain secrets), compose files on disk.

**Assumption:** A compromised agent can invoke any granted tool. Scopes limit blast radius per server instance.

## Controls implemented

| NSA requirement | Implementation |
|-----------------|----------------|
| Filtering proxy | Documented; server accepts gateway-only traffic in production |
| Content controls | JSON Schema tool params, name/path validation, output size caps |
| Prompt injection | Output sanitization, secret pattern redaction |
| Output logging | Structured audit before agent handoff |
| Tool inventory pinning | `name@1` tools + `inventory_fingerprint()` |
| SIEM-ready audit | JSONL `mcp.tool.result` on stderr |
| OS sandboxing | Non-root container, read-only socket, no privileged runs |
| Per-message signing | Hooks documented; ecosystem gap noted |
| Discovery | Tool list via MCP; bind via stdio only |

## Scopes (least privilege)

Each tool maps to exactly one scope. Authorization is checked on every invocation. Missing scope → deny + audit event.

## Audit events

Every tool call emits one JSON line to stderr:

```json
{
  "event_type": "mcp.tool.result",
  "tool_name": "docker_list_containers",
  "caller_identity": "cursor-agent",
  "authorization_scope": "docker:read",
  "params_fingerprint": "sha256:…",
  "result_status": "success",
  "result_bytes": 512,
  "correlation_id": "…",
  "session_id": "…"
}
```

Raw secrets are not logged. Parameter bodies are fingerprinted only.

## Deployment hardening

1. Run behind an MCP gateway with DLP and rate limits.
2. Mount Docker socket read-only; never grant `docker:write` unless required.
3. Set `DOCKER_MCP_CALLER_IDENTITY` to the agent principal.
4. Ship stderr audit logs to your SIEM.
5. Pin `inventory_fingerprint()` in deployment manifests.

## Known gaps

- MCP per-message signing is not yet standard; TLS on stdio is N/A. Use gateway mTLS where required.
- Compose tools execute `docker compose` subprocess; validate compose file paths are within allowed directories in your gateway policy.

## Related

- [Architecture](architecture.md)
- [Deployment](features/deployment.md)
