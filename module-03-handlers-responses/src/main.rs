//! Module 03: Handlers and first responses.
//! Topics: handlers are plain async functions; the return types Axum
//! understands out of the box — `&'static str`, `String`, `Html`, `Json`,
//! `StatusCode`, tuples, `NoContent`, `Redirect` — and the `IntoResponse`
//! trait that unifies them. Ends with the health-check pattern every real
//! service ships.
//! Run: cargo run -p module-03-handlers-responses   (port 3003)
//!
//! ## Lesson Reading Guide
//!
//! Every handler here is a different *return type*, because in Axum the
//! response you get is decided by what the handler returns. Read top to
//! bottom with the server running, and `curl -i` each endpoint so you see
//! the status line and headers change — that is the whole lesson made
//! visible:
//!
//! - Lesson 1: text handlers — `&'static str` vs `String`
//! - Lesson 2: HTML pages — `Html<T>` and `content-type: text/html`
//! - Lesson 3: JSON endpoint — `Json<T>` and `application/json`
//! - Lesson 4: status codes with bodies — `(StatusCode, T)` tuples, `NoContent`
//! - Lesson 5: redirects — `Redirect::to/temporary/permanent`
//! - Lesson 6: a health check — the endpoint load balancers call first
//! - Lesson 7: `impl IntoResponse` returns, `.into_response()`, and what every
//!   built-in return type really is (a table you can query at `/types`)
//! - Lesson 8: more redirects (`temporary`) and `Json` with a status code
//!
//! Request *input* (extractors) is deliberately absent here; that is
//! Module 05.

use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json, NoContent, Redirect, Response},
    routing::get,
    Router,
};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// LESSON 1: Text handlers — &'static str vs String
// ============================================================================

/// The entry point: one route table, one port, serve forever.
///
/// What it does: registers every lesson endpoint below, binds port 3003, and
/// starts `axum::serve` (same skeleton you met in Module 01).
/// Why it matters: by now the shape should already feel familiar — the only
/// thing changing between modules is what each handler returns and what that
/// implies for the HTTP reply.
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/greeting", get(greeting))
        .route("/page", get(html_page))
        .route("/user", get(json_user))
        .route("/created", get(created_with_body))
        .route("/teapot", get(teapot_with_body))
        .route("/no-content", get(no_content))
        .route("/old-page", get(redirect_see_other))
        .route("/moved", get(redirect_permanent))
        .route("/health", get(health))
        .route("/maybe", get(maybe_response))
        .route("/types", get(response_types))
        .route("/temporary", get(redirect_temporary))
        .route("/json-created", get(json_created))
        .route("/plain-bytes", get(plain_bytes));

    let port = std::env::var("AXUM_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3003);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|error| panic!("could not bind port {port}: {error}"));

    println!("Module 03 (Handlers and first responses) listening on http://127.0.0.1:{port}");
    println!("Try it:  curl -si http://127.0.0.1:{port}/  then the routes it lists.");

    axum::serve(listener, app).await.expect("server stopped unexpectedly");
}

/// GET / — the dashboard, itself an `Html` handler (Lesson 2 in disguise).
///
/// What it does: returns a tiny HTML index linking every lesson endpoint.
/// Why it matters: HTML in a `&'static str` costs nothing at runtime — the
/// bytes live in the read-only section of your binary and get served
/// straight from it. For indexes, docs pages and error templates that is
/// ideal.
async fn dashboard() -> Html<&'static str> {
    Html(
        "<h1>Module 03 — handlers and responses</h1>\
         <ul>\
         <li>GET /greeting — String, built at request time</li>\
         <li>GET /page — Html static</li>\
         <li>GET /user — Json</li>\
         <li>GET /created — (201 Created, body) in a tuple</li>\
         <li>GET /teapot — (418, body) in a tuple</li>\
         <li>GET /no-content — 204 NoContent, empty body</li>\
         <li>GET /old-page — Redirect::to (303 See Other)</li>\
         <li>GET /moved — Redirect::permanent (308)</li>\
         <li>GET /health — the health check</li>\
         <li>GET /maybe?mode=json|html|text — impl IntoResponse with different branches</li>\
         <li>GET /types — every built-in return type in one table</li>\
         <li>GET /temporary — Redirect::temporary (307)</li>\
         <li>GET /json-created — (201, Json) tuple</li>\
         <li>GET /plain-bytes — Vec&lt;u8&gt; / [u8] bodies with a content type</li>\
         </ul>",
    )
}

/// GET /greeting — a dynamic text handler.
///
/// What it does: builds the sentence at request time and returns a `String`.
/// Why it matters: `String` = allocated bytes, fresh per request — good when
/// the response really changes (times, counters, DB rows later). `&'static
/// str` = free forever. Choosing between them is the tiny performance
/// decision every handler makes (Module 40 profiles both).
async fn greeting() -> String {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock before 1970")
        .as_secs();
    format!("Hello from the /greeting handler at epoch second {epoch}. Return a String when the body is dynamic.")
}

// ============================================================================
// LESSON 2: HTML (Html<T> wraps any body and claims text/html)
// ============================================================================

/// GET /page — a complete (if minimal) HTML page.
///
/// What it does: returns `Html<&'static str>`; the wrapper sets
/// `Content-Type: text/html; charset=utf-8` so browsers really render it.
/// Why it matters: `Html<T>` and `Json<T>` differ from `String` only by the
/// declared content type — Axum has no templates until Module 13 installs
/// Askama/Minijinja. Header + body from one tiny wrapper is the pattern you
/// will reuse (and generalize with `IntoResponseParts`, Module 06).
async fn html_page() -> Html<&'static str> {
    Html(
        "<!doctype html>\
         <html lang=\"en\">\
         <head><meta charset=\"utf-8\"><title>Module 03</title>\
         <style>body{font-family:system-ui;max-width:640px;margin:2rem auto;color:#333}</style></head>\
         <body>\
         <h1>Served by Axum, rendered by your browser</h1>\
         <p>This whole page is a <code>&'static str</code> wrapped in <code>Html</code>.</p>\
         <p>Clicking below leaves this page, hits <code>/old-page</code>, and the server answers <code>303 See Other</code> with <code>location: /page</code> — your browser quietly goes there.</p>\
         <p><a href=\"old-page\">follow the redirect</a></p>\
         </body></html>",
    )
}

// ============================================================================
// LESSON 3: JSON (Json<T> — structs as the API contract)
// ============================================================================

/// The typed response body; the same pattern Module 02's struct taught.
#[derive(Serialize)]
struct UserView {
    id: u64,
    username: String,
    is_admin: bool,
}

/// GET /user — the JSON endpoint.
///
/// What it does: serializes the typed view struct and hands it to `Json`,
/// which stamps `application/json` on the response.
/// Why it matters: real APIs return *views*, not database rows: the struct
/// decides which fields clients see. Later, Module 15's service layer fills
/// this exact struct from a repository; the shape clients see never changes.
async fn json_user() -> Json<UserView> {
    Json(UserView {
        id: 1,
        username: "aarambh-dev-hub".to_string(),
        is_admin: false,
    })
}

// ============================================================================
// LESSON 4: Status codes with bodies (tuples) and empty bodies (204)
// ============================================================================

/// GET /created — status + body in one tuple.
///
/// What it does: returns `(StatusCode::CREATED, &'static str)`; the tuple
/// applies status 201 and the body is served, no needed wrapper struct.
/// Why it matters: the tuple-implements-IntoResponse parts chain and style:
/// for `(StatusCode, T)` Axum guarantees the first element decides the
/// status line and the rest builds the body. `201 Created` without a body is
/// legal too — plain `StatusCode::CREATED` from Lesson Module 02 — but
/// clients benefit from a body, so fortune suggests one.
async fn created_with_body() -> (StatusCode, &'static str) {
    (
        StatusCode::CREATED,
        "Created (201). This body is the second part of a response tuple.",
    )
}

/// GET /teapot — the famous 418, this time with a body.
async fn teapot_with_body() -> (StatusCode, &'static str) {
    (
        StatusCode::IM_A_TEAPOT,
        "The teapot regrets that it cannot brew coffee (HTTP 418).",
    )
}

/// GET /no-content — the explicit 204 No Content.
///
/// What it does: returns `NoContent` (the axum::response::NoContent marker
/// type), producing 204 with content-length 0.
/// Why it matters: `204 No Content` is the professional acknowledgement —
/// "done, nothing to send back". DELETE success, dry-run checks, and any
/// request whose answer is silence all deserve exactly this status instead
/// of 200 with an empty string.
async fn no_content() -> NoContent {
    NoContent
}

// ============================================================================
// LESSON 5: Redirects (303 See Other / 308 Permanent Redirect)
// ============================================================================

/// GET /old-page — redirect with method change to GET.
///
/// What it does: returns `Redirect::to("/page")`, i.e. **303 See Other** and
/// a `location` header pointing at the dashboard.
/// Why it matters: 303 explicitly says "the answer was a step, now do a plain
/// GET" — the correct after-form-submission redirect: browsers refresh and
/// never re-POST (the post-redirect-get pattern, Module 13). The body is
/// empty on purpose: redirects say *where*, not *what*.
async fn redirect_see_other() -> Redirect {
    Redirect::to("/page")
}

/// GET /moved — permanent redirect preserving the method.
///
/// What it does: returns `Redirect::permanent("/")` → **308** plus location.
/// Why it matters: 308 (and its temporary cousin 307) explicitly preserve the
/// HTTP method and body; 301/302 leave method semantics to history, which
/// crawlers and caches exploit differently. Rules of thumb: URL change? → 308.
/// After a form? → 303 (`to`). Repeated short-lived rebinds → 307.
/// Verify all three with `curl -i` and `-L`.
async fn redirect_permanent() -> Redirect {
    Redirect::permanent("/")
}

// ============================================================================
// LESSON 6: A health check (the endpoint everything else points at)
// ============================================================================

/// A status field small enough to keep and typed enough to format.
#[derive(Serialize)]
struct HealthReport {
    status: String,
    epoch_seconds: u64,
}

/// GET /health — a real-world ready endpoint.
///
/// What it does: returns 200 with a small JSON report while this process is
/// alive; load balancers, Kubernetes probes, and uptime monitors call it
/// louder than any other route.
/// Why it matters: two conventions worth copying: keep it *cheap* (no
/// database pools or locks here — Module 37 adds DB-aware "ready" checks) and
/// keep it *JSON* so dashboards parse it. Once `axum::serve` is running,
/// this endpoint existing is proof the server came up.
async fn health() -> Json<HealthReport> {
    Json(HealthReport {
        status: "ok".to_string(),
        epoch_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before 1970")
            .as_secs(),
    })
}

// ============================================================================
// LESSON 7: `impl IntoResponse` and the built-in return types
// ============================================================================

/// GET /maybe — one handler, three different concrete return types.
///
/// What it does: returns `Response` after calling `.into_response()` on
/// whichever value the branch produced (the query string is read by hand here
/// because extractors are Module 05).
/// Why it matters: a handler must return ONE type. `if` branches that build a
/// `Json`, an `Html` and a `String` are three types, so the compiler refuses —
/// until each is converted with `.into_response()` into the common `Response`.
/// `impl IntoResponse` as a return type is the same trick when ALL branches
/// share one type (see `plain_bytes` below). Try `?mode=json`, `?mode=html`.
async fn maybe_response(request: axum::extract::Request) -> Response {
    let query = request.uri().query().unwrap_or("");
    if query.contains("mode=json") {
        Json(serde_json::json!({ "mode": "json" })).into_response()
    } else if query.contains("mode=html") {
        Html("<p>mode = <b>html</b></p>").into_response()
    } else {
        "mode = text\n".into_response()
    }
}

/// One row of the "what does each return type become" table.
#[derive(Serialize)]
struct TypeRow {
    returns: &'static str,
    status: u16,
    content_type: &'static str,
    note: &'static str,
}

/// GET /types — the reference table for this module, as JSON.
///
/// What it does: lists every return type taught here and the response it
/// produces. Why it matters: this is the "explain each built-in" cheat sheet
/// — check it against `curl -si` of the matching endpoint.
async fn response_types() -> Json<Vec<TypeRow>> {
    Json(vec![
        TypeRow { returns: "&'static str / String", status: 200, content_type: "text/plain; charset=utf-8", note: "dynamic text -> String" },
        TypeRow { returns: "Html<T>", status: 200, content_type: "text/html; charset=utf-8", note: "T = &str or String" },
        TypeRow { returns: "Json<T: Serialize>", status: 200, content_type: "application/json", note: "serialisation failure -> 500" },
        TypeRow { returns: "StatusCode", status: 0, content_type: "(none)", note: "status only, empty body" },
        TypeRow { returns: "(StatusCode, T)", status: 0, content_type: "from T", note: "first element overrides the status" },
        TypeRow { returns: "NoContent", status: 204, content_type: "(none)", note: "the polite 'done, nothing to say'" },
        TypeRow { returns: "Redirect::to", status: 303, content_type: "(none)", note: "after a POST, go GET elsewhere" },
        TypeRow { returns: "Redirect::temporary", status: 307, content_type: "(none)", note: "same method, short-lived move" },
        TypeRow { returns: "Redirect::permanent", status: 308, content_type: "(none)", note: "same method, forever" },
        TypeRow { returns: "Vec<u8> / Bytes / &'static [u8]", status: 200, content_type: "application/octet-stream", note: "set your own type with a header tuple" },
        TypeRow { returns: "Result<T, E>", status: 0, content_type: "from the arm", note: "both arms must be IntoResponse (Module 08)" },
        TypeRow { returns: "()", status: 200, content_type: "(none)", note: "empty 200" },
    ])
}

// ============================================================================
// LESSON 8: more redirects, Json with a status, and raw bytes
// ============================================================================

/// GET /temporary — 307 Temporary Redirect.
///
/// What it does: `Redirect::temporary("/page")`. Why it matters: unlike 303,
/// a 307 tells the client to REPEAT the same method and body — right for
/// "this endpoint is briefly served elsewhere".
async fn redirect_temporary() -> Redirect {
    Redirect::temporary("/page")
}

/// GET /json-created — a typed JSON body with a non-200 status.
///
/// What it does: returns `(StatusCode::CREATED, Json<UserView>)`.
/// Why it matters: this is the real-world shape of "POST created it, here is
/// the new resource". Add a `Location` header in Module 06.
async fn json_created() -> (StatusCode, Json<UserView>) {
    (
        StatusCode::CREATED,
        Json(UserView { id: 2, username: "new-user".to_string(), is_admin: false }),
    )
}

/// GET /plain-bytes — raw bytes need an explicit content type.
///
/// What it does: returns `impl IntoResponse` — one concrete type, so the
/// opaque return type is allowed — a header array plus a byte slice.
/// Why it matters: bytes alone default to `application/octet-stream`; the
/// array `[(HeaderName, &str); 1]` is an `IntoResponseParts` (Module 06) and
/// is how you label what the bytes are.
async fn plain_bytes() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], &b"raw bytes, labelled as text\n"[..])
}
