use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use utoipa::{OpenApi, ToSchema};

use crate::client::{self, DepError};

pub const SERVICE: &str = "srvcs-distance3d";
pub const CONCERN: &str = "geometry: distance between two 3D points";
pub const DEPENDS_ON: &[&str] = &[
    "srvcs-floatsubtract",
    "srvcs-floatmultiply",
    "srvcs-floatadd",
    "srvcs-sqrt",
];

/// Dependency endpoints, injected as router state so tests can point them at
/// mock services.
#[derive(Clone)]
pub struct Deps {
    pub floatsubtract_url: String,
    pub floatmultiply_url: String,
    pub floatadd_url: String,
    pub sqrt_url: String,
}

#[derive(Serialize, ToSchema)]
pub struct Info {
    pub service: &'static str,
    pub concern: &'static str,
    pub depends_on: Vec<&'static str>,
}

/// `GET /` — service identity (srvcs service standard).
#[utoipa::path(get, path = "/", responses((status = 200, body = Info)))]
pub async fn index() -> Json<Info> {
    Json(Info {
        service: SERVICE,
        concern: CONCERN,
        depends_on: DEPENDS_ON.to_vec(),
    })
}

#[derive(Deserialize, ToSchema)]
pub struct EvalRequest {
    /// X coordinate of the first point.
    #[schema(value_type = Object)]
    pub x1: Value,
    /// Y coordinate of the first point.
    #[schema(value_type = Object)]
    pub y1: Value,
    /// Z coordinate of the first point.
    #[schema(value_type = Object)]
    pub z1: Value,
    /// X coordinate of the second point.
    #[schema(value_type = Object)]
    pub x2: Value,
    /// Y coordinate of the second point.
    #[schema(value_type = Object)]
    pub y2: Value,
    /// Z coordinate of the second point.
    #[schema(value_type = Object)]
    pub z2: Value,
}

#[derive(Serialize, ToSchema)]
pub struct Distance3dResponse {
    #[schema(value_type = Object)]
    pub x1: Value,
    #[schema(value_type = Object)]
    pub y1: Value,
    #[schema(value_type = Object)]
    pub z1: Value,
    #[schema(value_type = Object)]
    pub x2: Value,
    #[schema(value_type = Object)]
    pub y2: Value,
    #[schema(value_type = Object)]
    pub z2: Value,
    pub result: f64,
}

#[allow(clippy::too_many_arguments)]
fn ok(x1: Value, y1: Value, z1: Value, x2: Value, y2: Value, z2: Value, result: f64) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "x1": x1, "y1": y1, "z1": z1,
            "x2": x2, "y2": y2, "z2": z2,
            "result": result,
        })),
    )
        .into_response()
}

fn degraded(dependency: &str) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "dependency unavailable", "dependency": dependency })),
    )
        .into_response()
}

fn forward(status: u16, body: Value) -> Response {
    let code = StatusCode::from_u16(status).unwrap_or(StatusCode::BAD_GATEWAY);
    (code, Json(body)).into_response()
}

/// A reachable dependency answered `200` but its body lacked a numeric
/// `result`. That is a contract violation we cannot recover from, so surface a
/// `500` rather than guessing.
fn malformed(dependency: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(
            json!({ "error": "dependency returned a malformed result", "dependency": dependency }),
        ),
    )
        .into_response()
}

/// Call one dependency at `url` with `body`, mapping its outcome to either the
/// numeric `result` (on `200`) or an early-return `Response` the caller should
/// surface verbatim:
///
/// - unreachable / non-`200`/`422` -> `503` degraded
/// - `422` -> forwarded `422` (the dependency rejected the input)
/// - `200` without a numeric `result` -> `500` malformed
async fn ask(url: &str, body: &Value, dependency: &str) -> Result<f64, Response> {
    match client::call(url, body).await {
        Err(DepError::Unreachable) => Err(degraded(dependency)),
        Ok((200, body)) => match body.get("result").and_then(Value::as_f64) {
            Some(v) => Ok(v),
            None => Err(malformed(dependency)),
        },
        Ok((422, body)) => Err(forward(422, body)),
        Ok(_) => Err(degraded(dependency)),
    }
}

/// `POST /` — Euclidean distance between two 3D points.
///
/// This service owns the *control flow* but delegates every arithmetic step to
/// its dependencies, exactly as specified:
///
/// 1. `dx = x2 - x1`, `dy = y2 - y1`, `dz = z2 - z1` via `srvcs-floatsubtract`;
/// 2. square each via `srvcs-floatmultiply`;
/// 3. `sum = (dx2 + dy2) + dz2` by chaining two `srvcs-floatadd` calls;
/// 4. `result = sqrt(sum)` via `srvcs-sqrt`.
///
/// If a dependency is unreachable it reports itself degraded (`503`); if a
/// dependency rejects an input it forwards the `422`.
#[utoipa::path(
    post,
    path = "/",
    request_body = EvalRequest,
    responses(
        (status = 200, body = Distance3dResponse),
        (status = 422, description = "a dependency rejected an input (forwarded)"),
        (status = 500, description = "a dependency returned a malformed result"),
        (status = 503, description = "a dependency is unavailable")
    )
)]
pub async fn evaluate(State(deps): State<Deps>, Json(req): Json<EvalRequest>) -> Response {
    // 1. dx, dy, dz via floatsubtract (second axis - first axis).
    let dx = match ask(
        &deps.floatsubtract_url,
        &json!({ "a": req.x2, "b": req.x1 }),
        "srvcs-floatsubtract",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let dy = match ask(
        &deps.floatsubtract_url,
        &json!({ "a": req.y2, "b": req.y1 }),
        "srvcs-floatsubtract",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let dz = match ask(
        &deps.floatsubtract_url,
        &json!({ "a": req.z2, "b": req.z1 }),
        "srvcs-floatsubtract",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    // 2. square each via floatmultiply.
    let dx2 = match ask(
        &deps.floatmultiply_url,
        &json!({ "a": dx, "b": dx }),
        "srvcs-floatmultiply",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let dy2 = match ask(
        &deps.floatmultiply_url,
        &json!({ "a": dy, "b": dy }),
        "srvcs-floatmultiply",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let dz2 = match ask(
        &deps.floatmultiply_url,
        &json!({ "a": dz, "b": dz }),
        "srvcs-floatmultiply",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    // 3. sum = (dx2 + dy2) + dz2 by chaining two floatadd calls.
    let partial = match ask(
        &deps.floatadd_url,
        &json!({ "a": dx2, "b": dy2 }),
        "srvcs-floatadd",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };
    let sum = match ask(
        &deps.floatadd_url,
        &json!({ "a": partial, "b": dz2 }),
        "srvcs-floatadd",
    )
    .await
    {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    // 4. result = sqrt(sum) via sqrt.
    let result = match ask(&deps.sqrt_url, &json!({ "value": sum }), "srvcs-sqrt").await {
        Ok(v) => v,
        Err(resp) => return resp,
    };

    ok(req.x1, req.y1, req.z1, req.x2, req.y2, req.z2, result)
}

#[derive(OpenApi)]
#[openapi(
    paths(index, evaluate),
    components(schemas(Info, EvalRequest, Distance3dResponse))
)]
pub struct ApiDoc;

/// Serve OpenAPI document
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openapi_documents_routes() {
        let doc = ApiDoc::openapi();
        let root = doc.paths.paths.get("/").expect("path / present");
        assert!(root.get.is_some());
        assert!(root.post.is_some());
    }

    #[tokio::test]
    async fn index_reports_all_dependencies() {
        let Json(info) = index().await;
        assert_eq!(info.service, "srvcs-distance3d");
        assert_eq!(info.concern, "geometry: distance between two 3D points");
        assert_eq!(
            info.depends_on,
            vec![
                "srvcs-floatsubtract",
                "srvcs-floatmultiply",
                "srvcs-floatadd",
                "srvcs-sqrt"
            ]
        );
    }
}
