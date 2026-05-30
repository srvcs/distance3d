use axum::body::Body;
use axum::extract::Json as AxumJson;
use axum::http::{Request, StatusCode};
use axum::routing::post;
use axum::{Json, Router as AxumRouter};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use srvcs_distance3d::{api::Deps, health, router, telemetry};
use tower::ServiceExt;

const DEAD_URL: &str = "http://127.0.0.1:1";

// --- Computing mocks for every srvcs primitive this family composes over.
//
// Each reads its operands from the request body and returns the *real* answer,
// so the orchestration is genuinely exercised against the asserted cases rather
// than fed canned values. distance3d only calls floatsubtract, floatmultiply,
// floatadd and sqrt; the rest are provided for completeness of the family's
// contract.

/// `srvcs-floatadd`: reads `{a, b}` -> `{"result": a + b}` (as f64).
async fn spawn_floatadd() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let a = body.get("a").and_then(Value::as_f64).unwrap_or(0.0);
            let b = body.get("b").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": a + b }))
        }),
    );
    serve(app).await
}

/// `srvcs-floatsubtract`: reads `{a, b}` -> `{"result": a - b}` (as f64).
async fn spawn_floatsubtract() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let a = body.get("a").and_then(Value::as_f64).unwrap_or(0.0);
            let b = body.get("b").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": a - b }))
        }),
    );
    serve(app).await
}

/// `srvcs-floatmultiply`: reads `{a, b}` -> `{"result": a * b}` (as f64).
async fn spawn_floatmultiply() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let a = body.get("a").and_then(Value::as_f64).unwrap_or(0.0);
            let b = body.get("b").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": a * b }))
        }),
    );
    serve(app).await
}

/// `srvcs-floatdivide`: reads `{a, b}` -> `{"result": a / b}` (as f64).
#[allow(dead_code)]
async fn spawn_floatdivide() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let a = body.get("a").and_then(Value::as_f64).unwrap_or(0.0);
            let b = body.get("b").and_then(Value::as_f64).unwrap_or(1.0);
            Json(json!({ "result": a / b }))
        }),
    );
    serve(app).await
}

/// `srvcs-sqrt`: reads `{value}` -> `{"result": sqrt(value)}` (as f64).
async fn spawn_sqrt() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": value.sqrt() }))
        }),
    );
    serve(app).await
}

/// `srvcs-sin`: reads `{value}` -> `{"result": sin(value)}` (as f64).
#[allow(dead_code)]
async fn spawn_sin() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": value.sin() }))
        }),
    );
    serve(app).await
}

/// `srvcs-cos`: reads `{value}` -> `{"result": cos(value)}` (as f64).
#[allow(dead_code)]
async fn spawn_cos() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": value.cos() }))
        }),
    );
    serve(app).await
}

/// `srvcs-tan`: reads `{value}` -> `{"result": tan(value)}` (as f64).
#[allow(dead_code)]
async fn spawn_tan() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|AxumJson(body): AxumJson<Value>| async move {
            let value = body.get("value").and_then(Value::as_f64).unwrap_or(0.0);
            Json(json!({ "result": value.tan() }))
        }),
    );
    serve(app).await
}

/// `srvcs-pi`: returns `{"result": PI}` for any body.
#[allow(dead_code)]
async fn spawn_pi() -> String {
    let app = AxumRouter::new().route(
        "/",
        post(|| async move { Json(json!({ "result": std::f64::consts::PI })) }),
    );
    serve(app).await
}

/// Spawn a mock returning a fixed status + body (used for error-path tests).
async fn spawn_fixed(status: StatusCode, body: Value) -> String {
    let app = AxumRouter::new().route(
        "/",
        post(move || {
            let body = body.clone();
            async move { (status, Json(body)) }
        }),
    );
    serve(app).await
}

async fn serve(app: AxumRouter) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}

/// All four dependencies as computing mocks — the common happy-path fixture.
async fn computing_deps() -> Deps {
    Deps {
        floatsubtract_url: spawn_floatsubtract().await,
        floatmultiply_url: spawn_floatmultiply().await,
        floatadd_url: spawn_floatadd().await,
        sqrt_url: spawn_sqrt().await,
    }
}

fn app(deps: Deps) -> axum::Router {
    router(telemetry::metrics_handle_for_tests(), deps)
}

fn dead_deps() -> Deps {
    Deps {
        floatsubtract_url: DEAD_URL.to_string(),
        floatmultiply_url: DEAD_URL.to_string(),
        floatadd_url: DEAD_URL.to_string(),
        sqrt_url: DEAD_URL.to_string(),
    }
}

async fn distance(deps: Deps, req: Value) -> (StatusCode, Value) {
    let res = app(deps)
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(req.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = res.status();
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

async fn status_of(uri: &str) -> StatusCode {
    app(dead_deps())
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
        .status()
}

fn result_f64(body: &Value) -> f64 {
    body["result"].as_f64().expect("result is a JSON number")
}

// --- Standard endpoints. ---

#[tokio::test]
async fn healthz_ok() {
    assert_eq!(status_of("/healthz").await, StatusCode::OK);
}

#[tokio::test]
async fn readyz_reflects_state() {
    health::set_ready(true);
    assert_eq!(status_of("/readyz").await, StatusCode::OK);
}

#[tokio::test]
async fn metrics_ok() {
    assert_eq!(status_of("/metrics").await, StatusCode::OK);
}

#[tokio::test]
async fn openapi_ok() {
    assert_eq!(status_of("/openapi.json").await, StatusCode::OK);
}

#[tokio::test]
async fn generates_request_id_when_absent() {
    let res = app(dead_deps())
        .oneshot(
            Request::builder()
                .uri("/healthz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert!(
        res.headers().contains_key("x-request-id"),
        "response must carry a generated x-request-id"
    );
}

#[tokio::test]
async fn index_reports_identity() {
    let res = app(dead_deps())
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["service"], "srvcs-distance3d");
    assert_eq!(body["concern"], "geometry: distance between two 3D points");
    assert_eq!(
        body["depends_on"],
        json!([
            "srvcs-floatsubtract",
            "srvcs-floatmultiply",
            "srvcs-floatadd",
            "srvcs-sqrt"
        ])
    );
}

// --- Correctness cases, against the computing mocks. ---

#[tokio::test]
async fn distance_0_0_0_to_1_2_2_is_3() {
    let (status, body) = distance(
        computing_deps().await,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // sqrt(1 + 4 + 4) = sqrt(9) = 3.0
    assert!((result_f64(&body) - 3.0).abs() < 1e-9);
}

#[tokio::test]
async fn distance_echoes_coordinates() {
    let (status, body) = distance(
        computing_deps().await,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["x1"], json!(0));
    assert_eq!(body["y1"], json!(0));
    assert_eq!(body["z1"], json!(0));
    assert_eq!(body["x2"], json!(1));
    assert_eq!(body["y2"], json!(2));
    assert_eq!(body["z2"], json!(2));
}

#[tokio::test]
async fn distance_identical_points_is_zero() {
    let (status, body) = distance(
        computing_deps().await,
        json!({ "x1": 3, "y1": -2, "z1": 7, "x2": 3, "y2": -2, "z2": 7 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!((result_f64(&body) - 0.0).abs() < 1e-9);
}

#[tokio::test]
async fn distance_axis_aligned_is_the_delta() {
    let (status, body) = distance(
        computing_deps().await,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 0, "y2": 0, "z2": 5 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert!((result_f64(&body) - 5.0).abs() < 1e-9);
}

#[tokio::test]
async fn distance_unit_diagonal_is_sqrt3() {
    let (status, body) = distance(
        computing_deps().await,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 1, "z2": 1 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // sqrt(3)
    assert!((result_f64(&body) - 3.0_f64.sqrt()).abs() < 1e-9);
}

#[tokio::test]
async fn distance_negative_and_fractional_coords() {
    let (status, body) = distance(
        computing_deps().await,
        json!({ "x1": -1.5, "y1": 2.0, "z1": 0.5, "x2": 1.5, "y2": -2.0, "z2": -0.5 }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    // dx=3, dy=-4, dz=-1; sqrt(9 + 16 + 1) = sqrt(26)
    assert!((result_f64(&body) - 26.0_f64.sqrt()).abs() < 1e-9);
}

// --- Error / edge cases. ---

#[tokio::test]
async fn degrades_when_floatsubtract_unreachable() {
    let deps = Deps {
        floatsubtract_url: DEAD_URL.to_string(),
        floatmultiply_url: spawn_floatmultiply().await,
        floatadd_url: spawn_floatadd().await,
        sqrt_url: spawn_sqrt().await,
    };
    let (status, body) = distance(
        deps,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-floatsubtract");
}

#[tokio::test]
async fn degrades_when_floatmultiply_unreachable() {
    let deps = Deps {
        floatsubtract_url: spawn_floatsubtract().await,
        floatmultiply_url: DEAD_URL.to_string(),
        floatadd_url: spawn_floatadd().await,
        sqrt_url: spawn_sqrt().await,
    };
    let (status, body) = distance(
        deps,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-floatmultiply");
}

#[tokio::test]
async fn degrades_when_floatadd_unreachable() {
    let deps = Deps {
        floatsubtract_url: spawn_floatsubtract().await,
        floatmultiply_url: spawn_floatmultiply().await,
        floatadd_url: DEAD_URL.to_string(),
        sqrt_url: spawn_sqrt().await,
    };
    let (status, body) = distance(
        deps,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-floatadd");
}

#[tokio::test]
async fn degrades_when_sqrt_unreachable() {
    let deps = Deps {
        floatsubtract_url: spawn_floatsubtract().await,
        floatmultiply_url: spawn_floatmultiply().await,
        floatadd_url: spawn_floatadd().await,
        sqrt_url: DEAD_URL.to_string(),
    };
    let (status, body) = distance(
        deps,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["dependency"], "srvcs-sqrt");
}

#[tokio::test]
async fn forwards_422_from_floatsubtract() {
    // Validation propagates from dependencies: floatsubtract rejects a
    // non-numeric coordinate with 422, which distance3d forwards verbatim.
    let deps = Deps {
        floatsubtract_url: spawn_fixed(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "value is not a number" }),
        )
        .await,
        floatmultiply_url: spawn_floatmultiply().await,
        floatadd_url: spawn_floatadd().await,
        sqrt_url: spawn_sqrt().await,
    };
    let (status, body) = distance(
        deps,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": "nope", "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["error"], "value is not a number");
}

#[tokio::test]
async fn forwards_422_from_sqrt() {
    let deps = Deps {
        floatsubtract_url: spawn_floatsubtract().await,
        floatmultiply_url: spawn_floatmultiply().await,
        floatadd_url: spawn_floatadd().await,
        sqrt_url: spawn_fixed(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({ "error": "negative radicand" }),
        )
        .await,
    };
    let (status, _) = distance(
        deps,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn malformed_floatmultiply_result_is_500() {
    let deps = Deps {
        floatsubtract_url: spawn_floatsubtract().await,
        floatmultiply_url: spawn_fixed(StatusCode::OK, json!({ "result": "not-a-number" })).await,
        floatadd_url: spawn_floatadd().await,
        sqrt_url: spawn_sqrt().await,
    };
    let (status, body) = distance(
        deps,
        json!({ "x1": 0, "y1": 0, "z1": 0, "x2": 1, "y2": 2, "z2": 2 }),
    )
    .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["dependency"], "srvcs-floatmultiply");
}
