# srvcs-distance3d

## Name

| Field | Value |
| --- | --- |
| Service | `srvcs-distance3d` |
| Slug | `distance3d` |
| Repository | `srvcs/distance3d` |
| Package | `srvcs-distance3d` |
| Kind | `orchestrator` |

## Function

geometry: distance between two 3D points

## Dependencies

| Dependency | Repository |
| --- | --- |
| `srvcs-floatsubtract` | [srvcs/floatsubtract](https://github.com/srvcs/floatsubtract) |
| `srvcs-floatmultiply` | [srvcs/floatmultiply](https://github.com/srvcs/floatmultiply) |
| `srvcs-floatadd` | [srvcs/floatadd](https://github.com/srvcs/floatadd) |
| `srvcs-sqrt` | [srvcs/sqrt](https://github.com/srvcs/sqrt) |

## API

| Method | Path | Purpose |
| --- | --- | --- |
| `GET` | `/` | Service identity |
| `POST` | `/` | Evaluate the service function |
| `GET` | `/healthz` | Liveness probe |
| `GET` | `/readyz` | Readiness probe |
| `GET` | `/metrics` | Prometheus metrics |
| `GET` | `/openapi.json` | OpenAPI document |

## Inputs

| Name | Type | Required |
| --- | --- | --- |
| `x1` | `json` | yes |
| `y1` | `json` | yes |
| `z1` | `json` | yes |
| `x2` | `json` | yes |
| `y2` | `json` | yes |
| `z2` | `json` | yes |

## Outputs

| Name | Type |
| --- | --- |
| `x1` | `json` |
| `y1` | `json` |
| `z1` | `json` |
| `x2` | `json` |
| `y2` | `json` |
| `z2` | `json` |
| `result` | `number` |

## Configuration

| Variable | Default | Purpose |
| --- | --- | --- |
| `SRVCS_BIND_ADDR` | `0.0.0.0:8080` | Bind address |
| `SRVCS_ENV` | `development` | Environment label for logs |
| `RUST_LOG` | `info,tower_http=info` | Tracing filter |
| `SRVCS_FLOATADD_URL` | `http://127.0.0.1:8092` | Base URL for srvcs-floatadd |
| `SRVCS_FLOATMULTIPLY_URL` | `` | Base URL for srvcs-floatmultiply |
| `SRVCS_FLOATSUBTRACT_URL` | `` | Base URL for srvcs-floatsubtract |
| `SRVCS_SQRT_URL` | `http://127.0.0.1:8093` | Base URL for srvcs-sqrt |

## Error Behavior

- `422` means the request could not be evaluated for the documented input shape.
- `503` means a required dependency was unavailable or returned an unexpected response.
- Dependency validation errors are forwarded when this service delegates validation.

## Local Checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

See the [srvcs service standard](https://github.com/srvcs/platform/blob/main/STANDARD.md) for the full operational contract.

## Metadata

Machine-readable service metadata lives in `srvcs.yaml`. Keep it aligned with this README when the service contract changes.
