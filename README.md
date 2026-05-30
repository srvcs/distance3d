# srvcs-distance3d

Euclidean distance between two points in 3D space, for srvcs.cloud.

This is an **orchestrator** over float primitives: it owns the control flow but
delegates every arithmetic step to its dependencies. It does **not** call
`srvcs-isnumber` directly — input validation propagates from the dependencies
(their `422`s are forwarded verbatim).

## Concern

`geometry: distance between two 3D points`

Given two points `(x1, y1, z1)` and `(x2, y2, z2)`, the distance is

```
sqrt((x2 - x1)^2 + (y2 - y1)^2 + (z2 - z1)^2)
```

## Algorithm

Like `srvcs-distance2d` with a third axis:

1. `dx = x2 - x1`, `dy = y2 - y1`, `dz = z2 - z1` via `srvcs-floatsubtract`;
2. square each delta via `srvcs-floatmultiply`;
3. `sum = (dx^2 + dy^2) + dz^2` by chaining two `srvcs-floatadd` calls;
4. `result` (an `f64`) `= sqrt(sum)` via `srvcs-sqrt`.

For example, `distance3d(0,0,0, 1,2,2) = 3.0`.

## Dependencies

| Service               | Env var                   | Default                |
| --------------------- | ------------------------- | ---------------------- |
| `srvcs-floatsubtract` | `SRVCS_FLOATSUBTRACT_URL` | `http://127.0.0.1:8090` |
| `srvcs-floatmultiply` | `SRVCS_FLOATMULTIPLY_URL` | `http://127.0.0.1:8091` |
| `srvcs-floatadd`      | `SRVCS_FLOATADD_URL`      | `http://127.0.0.1:8092` |
| `srvcs-sqrt`          | `SRVCS_SQRT_URL`          | `http://127.0.0.1:8093` |

## HTTP API

### `GET /`

Service identity.

```json
{
  "service": "srvcs-distance3d",
  "concern": "geometry: distance between two 3D points",
  "depends_on": ["srvcs-floatsubtract", "srvcs-floatmultiply", "srvcs-floatadd", "srvcs-sqrt"]
}
```

### `POST /`

Request:

```json
{ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }
```

Response `200`:

```json
{ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2, "result": 3.0 }
```

`result` is an `f64`.

Error responses:

- `422` — a dependency rejected an input (forwarded from the dependency).
- `500` — a dependency returned a malformed (non-numeric) `result`.
- `503` — a dependency is unavailable (`{"error", "dependency"}`).

## Local checks

```sh
nix flake check -L
nix develop -c sh -euc 'cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test'
nix build .#default -L
```

The Linux container is exposed as `.#container`.

See [`srvcs/platform`](https://github.com/srvcs/platform) for the shared service
standard and CI workflow.
