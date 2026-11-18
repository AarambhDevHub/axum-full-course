//! Module 04: Routing.
//! Topics: the route table (`Router::new().route` + one method selector per
//! row), path parameters `/{id}` and wildcards `/{*rest}`, `nest` and
//! `merge`, `fallback` (custom 404s) and `method_not_allowed_fallback`
//! (custom 405s), `reset_fallback`, `route_service`/`nest_service`, versioned
//! APIs, trailing slashes and automatic HEAD handling.
//! Run: cargo run -p module-04-routing   (port 3004)
//!
//! ## Lesson Reading Guide
//!
//! Routing is the map between URLs and handlers, and Axum builds it eagerly:
//! every `.route(...)` call is checked for conflicts *when the Router is
//! constructed*, so route bugs panic at startup instead of failing on
//! request 500. Read the 6 lessons below with the server up and the README's
//! curl table open:
//!
//! - Lesson 1: one route table — method selectors and a live CRUD demo
//! - Lesson 2: data inside URLs — `/{param}` and `{*wildcard}`
//! - Lesson 3: composition — `nest` an API under a prefix
//! - Lesson 4: composition — `merge` routers + `reset_fallback`
//! - Lesson 5: custom 404 and 405 — `fallback` + `method_not_allowed_fallback`
//! - Lesson 6: services as routes — `route_service`/`nest_service` primer
//! - Lesson 7: `any()`, `on(MethodFilter..)`, explicit HEAD, typed `Path<u32>`
//! - Lesson 8: trailing slashes (`route_with_tsr`), `route_service` with a
//!   hand-written `service_fn`, and `fallback_service`
//!
//! One honest stop-gap on display: the item store below is a `static` Mutex
//! holding a `HashMap`, because State (Module 07) has not been taught yet.
//! As soon as `Arc` + `Mutex` are introduced (Module 02, Lesson 3) this is
//! the minimal, honest way to keep data until then. URLs carry the item data
//! in this module on purpose — bodies are parsed by extractors, which land
//! in Module 05.

use axum::{
    body::Body,
    extract::{Path, Request},
    http::{HeaderValue, Method, StatusCode},
    response::{Html, IntoResponse, Json, NoContent, Response},
    routing::{any, get, on, post, put, MethodFilter},
    Router,
};
use axum_extra::routing::RouterExt; // adds `route_with_tsr` to Router
use std::{collections::HashMap, convert::Infallible, sync::Mutex};
use tower::service_fn;

// ============================================================================
// LESSON 1: The route table (methods + a live CRUD demo)
// ============================================================================

/// The tiny in-memory store, and how it becomes borrowable from anywhere.
///
/// What it does: holds `HashMap<u32, String>` (item id -> name) behind a
/// `Mutex`, inside a `static` that `OnceLock` initializes on first use.
/// Why it matters: `static` items live for the whole program (the `'static`
/// lifetime from earlier lessons) so the server remembers between requests.
/// Rust 1.80 lets every constructor be `const`, and `OnceLock` is the
/// textbook "initialize once, lazily, for the entire binary" tool. The
/// `Mutex` keeps mutations serialized safely (Module 02 Lesson 3 story).
/// Module 07 replaces all of this with `State` plumbing, and Module 14 with
/// a real database pool — the handlers stay identical.
fn item_store() -> &'static Mutex<HashMap<u32, String>> {
    static ITEMS: std::sync::OnceLock<Mutex<HashMap<u32, String>>> = std::sync::OnceLock::new();
    ITEMS.get_or_init(|| {
        Mutex::new(HashMap::from([(1_u32, "ring".to_string()), (2, "amulet".to_string())]))
    })
}

/// The entry point: this is the biggest `app` of the course so far, and the
/// nesting/merging lessons make it a small route forest.
///
/// What it does: assembles the tree you can find in the README's endpoints
/// table, binds port 3004, serves forever.
/// Why it matters: `Router` is built by chained method calls, and each call
/// returns the same router with one more rule. Order of the chain does not
/// matter for matching (the tree is exact-match based), but reading it top
/// down should read like a table of contents.
#[tokio::main]
async fn main() {
    // Sub-routers: built exactly like the main one, then attached below.
    let api_routes = Router::new()
        .route("/status", get(api_status))
        .route("/version", get(api_version))
        // A nested router can carry its own fallback for unknown paths *in
        // its own subtree only* — see Lesson 5 for the main app's fallback.
        .fallback(api_unknown);

    // Lesson 4: `merge` bolts routers together WITHOUT adding a prefix
    // (nest adds prefixes, merge adds routes — that's the difference). So a
    // merged router writes its full paths itself, here `/admin/keys`.
    // `merge` refuses two routers that both claim an explicit fallback, so
    // the admin router resets its (empty) fallback first — the pattern to
    // remember when merging two routers that DO both have one.
    let admin_routes = Router::new()
        .route("/admin/keys", get(admin_keys))
        .reset_fallback();

    let v1_routes = Router::new().route("/hello", get(v1_hello));
    let v2_routes = Router::new().route("/hello", get(v2_hello));

    // Lesson 6: `nest_service` mounts ANY tower service under a prefix —
    // routers are services too, so a tiny router inside nest_service is the
    // smallest legal example. (Module 13 mounts tower-http's real services:
    // `ServeDir`/`ServeFile` go here, exactly in this slot.)
    let legacy_routes = Router::new()
        .route("/", get(legacy_hello))
        .route("/{*rest}", get(legacy_rest));

    // Lesson 8: a router whose *fallback* is a service (not a handler).
    let svc_routes = Router::new()
        .route("/known", get(|| async { "svc/known is a normal handler row\n" }))
        .fallback_service(service_fn(hand_written_service));

    let app = Router::new()
        // ---- Lesson 1: the CRUD table ---------------------------------
        .route("/", get(dashboard))
        // BOTH selectors on one path register one MethodRouter for the row;
        // `post` creates by name-in-path, `get` reads by numeric id.
        .route("/items", get(list_items))
        .route(
            "/items/{item}",
            post(create_item).get(read_item).delete(delete_item),
        )
        // Two path params in one row (id and new name): Lesson 2 material.
        .route("/items/{item}/{name}", put(rename_item))
        // ---- Lesson 2: wildcards --------------------------------------
        .route("/files/{*path}", get(wildcard_files))
        // ---- Lesson 3: nesting ----------------------------------------
        .nest("/api", api_routes)
        // ---- Lesson 4: merging (with reset_fallback above) -------------
        .merge(admin_routes)
        // ---- Versioning with two nested routers ------------------------
        .nest("/v1", v1_routes)
        .nest("/v2", v2_routes)
        // ---- Lesson 6: a router mounted as a service -------------------
        .nest_service("/legacy", legacy_routes)
        // ---- Lesson 7: any(), on(), HEAD, typed Path ------------------
        .route("/echo-method", any(echo_method))
        .route("/read-or-write", on(MethodFilter::GET.or(MethodFilter::POST), read_or_write))
        .route("/head-demo", get(head_demo_get).head(head_demo_head))
        .route("/typed-items/{id}", get(typed_item))
        // ---- Lesson 8: trailing slash redirect, services, fallback_service
        .route_with_tsr("/docs", get(docs_page))
        .route_service("/service", service_fn(hand_written_service))
        .nest("/svc", svc_routes)
        // ---- Lesson 5: the app-wide 404 + 405 (defined below) ----------
        .fallback(not_found_json)
        .method_not_allowed_fallback(method_not_allowed_json);

    let port = std::env::var("AXUM_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3004);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|error| panic!("could not bind port {port}: {error}"));

    println!("Module 04 (Routing) listening on http://127.0.0.1:{port}");
    println!("Try it:  curl -si http://127.0.0.1:{port}/  for the route tree.");

    axum::serve(listener, app).await.expect("server stopped unexpectedly");
}

/// GET / — the route tree, self-documented.
async fn dashboard() -> Html<&'static str> {
    Html(
        "<h1>Module 04 — Routing</h1>\
         <ul>\
         <li>CRUD: GET /items &middot; POST /items/{name} &middot; GET|DELETE /items/{id} &middot; PUT /items/{id}/{name}</li>\
         <li>Wildcard: GET /files/{*path} — try /files/assets/css/main.css</li>\
         <li>Nest: GET /api/status — and GET /api/anything-else hits the nested fallback</li>\
         <li>Merge: GET /admin/keys (merge adds routes, no prefix; reset_fallback demo)</li>\
         <li>Versioning: GET /v1/hello vs GET /v2/hello</li>\
         <li>Service mount: GET /legacy/anything</li>\
         <li>Fallbacks: any unknown path — and PATCH /items/1 for the 405 one</li>\
         <li>any(): ANY /echo-method &middot; on(): GET|POST /read-or-write &middot; HEAD: /head-demo &middot; typed: /typed-items/{id}</li>\
         <li>Trailing slash: /docs and /docs/ &middot; services: /service, /svc/known, /svc/anything</li>\
         </ul>",
    )
}

// ---------------------------------------------------------------------------
// The CRUD handlers (routing + the Path extractor in its minimal form)
// ---------------------------------------------------------------------------

/// GET /items — list what the store holds.
///
/// Why it matters for a routing lesson: `.route("/items", get(...))` is the
/// *collection* row of a REST table; the parametrized rows below are the
/// *item* rows. Lock, clone the pairs out, drop the lock — never hold a
/// Mutex across building a whole response.
async fn list_items() -> String {
    let pairs: Vec<String> = item_store()
        .lock()
        .expect("store lock is fine")
        .iter()
        .map(|(id, name)| format!("{id}: {name}"))
        .collect();
    format!("items now: [{}]", pairs.join(", "))
}

/// POST /items/{name} — the create row (201 with the new id).
///
/// What it does: takes the name from the path (no body parsing until Module
/// 05), assigns the next free id, stores it, answers 201.
/// Why it matters: `.route(path, post(...).get(...).delete(...))` chains
/// method selectors onto ONE row of the table — one MethodRouter per path
/// pattern. Note registered methods automatically serve HEAD too (Module 03
/// takeaway).
async fn create_item(Path(name): Path<String>) -> (StatusCode, String) {
    let mut store = item_store().lock().expect("store lock is fine");
    let next_id = store.keys().max().copied().unwrap_or(0) + 1;
    let inserted = store.insert(next_id, name.clone());
    drop(store);
    match inserted {
        None => (StatusCode::CREATED, format!("created item {next_id}: '{name}'\n")),
        // Unreachable with fresh ids, but keep the door open for reuse bugs.
        Some(old) => (StatusCode::CONFLICT, format!("id {next_id} already held '{old}'\n")),
    }
}

/// GET /items/{item} — the read row.
///
/// What it does: parses the segment by hand; digits look the item up, any
/// other text answers as `banana`-style 404. (Module 05's `Path<u32>` does
/// this parse for you — here it is kept explicit so nothing is hidden.)
async fn read_item(Path(item): Path<String>) -> (StatusCode, String) {
    match item.parse::<u32>() {
        Ok(id) => match item_store().lock().expect("store lock is fine").get(&id) {
            Some(name) => (StatusCode::OK, format!("item {id} is '{name}'\n")),
            None => (StatusCode::NOT_FOUND, format!("no item {id}\n")),
        },
        Err(_) => (StatusCode::NOT_FOUND, format!("'{item}' is not an item id (numbers only)\n")),
    }
}

/// PUT /items/{item}/{name} — the update row: rename the item.
async fn rename_item(Path((item, name)): Path<(String, String)>) -> (StatusCode, String) {
    match item.parse::<u32>() {
        Ok(id) => {
            let old = item_store()
                .lock()
                .expect("store lock is fine")
                .insert(id, name.clone());
            match old {
                Some(_) => (StatusCode::OK, format!("item {id} renamed to '{name}'\n")),
                None => (StatusCode::NOT_FOUND, format!("no item {id} to rename\n")),
            }
        }
        Err(_) => (StatusCode::NOT_FOUND, format!("'{item}' is not an item id (numbers only)\n")),
    }
}

/// DELETE /items/{item} — the delete row: 204 when something really left.
///
/// What it does: removes the item when the id parses and exists; the tuple's
/// first element carries the verdict (204 for the hit, 404 for any miss)
/// while `NoContent` supplies the by-now-familiar empty body.
/// Why it matters: the same alternating-status trick as every other CRU:D
/// handler, shown with the "success that says nothing" status.
async fn delete_item(Path(item): Path<String>) -> (StatusCode, NoContent) {
    let removed = match item.parse::<u32>() {
        Ok(id) => item_store().lock().expect("store lock is fine").remove(&id),
        Err(_) => None,
    };
    if removed.is_some() {
        (StatusCode::NO_CONTENT, NoContent)
    } else {
        (StatusCode::NOT_FOUND, NoContent)
    }
}

// ============================================================================
// LESSON 2: The wildcard route ({*path} matches whole remainder)
// ============================================================================

/// GET /files/{*path} — the wildcard row.
///
/// What it does: matches `/files/a/b/c.css` and hands the handler the whole
/// remainder. Try `curl -s http://127.0.0.1:3004/files/assets/css/main.css`.
/// Why it matters: `{*path}` is the 0.8 wildcard syntax (old `*path` panics at
/// startup now). Params and wildcards — plus this same `Path` extractor with
/// typed targets and proper rejections — are Module 05's core lesson; this
/// row just proves routing's vocabulary first. Module 13 turns exactly this
/// row into real `ServeDir` file serving via `nest_service`.
async fn wildcard_files(Path(path): Path<String>) -> String {
    format!("wildcard matched remainder: '{path}'\n")
}

// ============================================================================
// LESSON 3: nest — an API under its own prefix, with its own fallback
// ============================================================================

/// GET /api/status — inside the nested router.
///
/// What it does: a normal handler, and yet it never mentions `/api` anywhere.
/// Why it matters: `nest("/api", api_routes)` gives every route inside the
/// nested router this prefix for free. Nested handlers never know their
/// prefix: build once, mount under any name (Module 15's `lib.rs` layout
/// builds APIs exactly this way, as plain functions returning Routers).
async fn api_status() -> &'static str {
    "api status: ok\n"
}

async fn api_version() -> &'static str {
    "api version: 0 — these are the nested test routes\n"
}

/// Fallback of the nested /api router.
///
/// What it does: catches anything under `/api/*` that has no row, and leaves
/// everything OUTSIDE `/api/*` to the main app's fallback (Lesson 5).
/// Why it matters: nested fallbacks are subtree-scoped. Run both:
/// `curl -si .../api/unknown` (this handler, JSON) and
/// `curl -si .../unknown` (main fallback, different JSON).
async fn api_unknown() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "unknown /api/* route",
            "hint": "the nested router owns fallbacks under its prefix",
        })),
    )
}

// ============================================================================
// LESSON 4: merge — bolt routers together (and reset_fallback before)
// ============================================================================

/// GET /admin/keys — from the separately-built router that was `merge`d in.
///
/// Why it matters for the routing lesson: `merge` never adds prefixes — this
/// router registered the FULL path `/admin/keys` itself, and the main app
/// just bolted it in. Different mental model from `nest`: nest names the
/// subtree, merge writes its own roads. (The fallback it reset is explained
/// next to `.reset_fallback()` above.)
async fn admin_keys() -> &'static str {
    "admin api keys: none, this module merges routers, not secrets\n"
}

// ============================================================================
// LESSON 4b: versioning — two nested routers, same inner paths, different
// handlers. `nest("/v1", ...)` + `nest("/v2", ...)` is the cheapest version
// flip in Axum; Module 18 revisits versioning as an API-design decision.
// ============================================================================

async fn v1_hello() -> &'static str {
    "v1 hello: plain string responses\n"
}

async fn v2_hello() -> &'static str {
    "v2 hello: same path, whole new handlers — routes are the contract\n"
}

// ============================================================================
// LESSON 5: The app-wide fallbacks (custom 404 and custom 405)
// ============================================================================

/// The main fallback — every no-row path in the whole app lands here.
///
/// What it does: returns a JSON 404 for every unknown path (HEAD has no body;
/// see GET for the shape).
/// Why it matters: unset, Axum answers unknown paths with the hardcoded not
/// found page. A *branded* 404/405 is what real APIs do. It runs even for
/// wildcard misses because matchit knows the tree has no such row at all.
async fn not_found_json() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
            "error": "not found",
            "hint": "the main router owns this fallback; /api/* unknown paths produce a different one",
        })),
    )
}

/// The method-miss fallback: path has a row, but not for this method.
///
/// What it does: replaces Axum's default `405 Method Not Allowed` with a
/// JSON one. Try `curl -si -X PATCH http://.../items/1`.
/// Why it matters: it fires only when the path is ALREADY registered — the
/// true "wrong verb" case; totally-unknown paths still hit `not_found_json`
/// above. Keep this small: the response cannot know every registered method.
async fn method_not_allowed_json() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::METHOD_NOT_ALLOWED,
        Json(serde_json::json!({
            "error": "method not allowed for this path",
            "hint": "see the per-path Allow header Axum already set",
        })),
    )
}

// ============================================================================
// LESSON 6: route_service / nest_service (handlers vs services as routes)
// ============================================================================

/// The handler(s) reachable under /legacy.
///
/// What it does: talks normally — `/{*rest}` proves nest_service strips the
/// `/legacy` prefix before the inner router ever sees the request.
/// Why it matters: `nest`/`route` expect *handlers* (extractor machinery on
/// every request). `nest_service`/`route_service` mount *services* — the
/// things that process whole requests, like tower-http's `ServeDir`
/// (Module 13) or tonic's gRPC server (Module 35). A mini router inside
/// `nest_service` is the smallest legal demo: same behavior, different
/// wiring — and Module 44 hand-writes a Service without any router at all.
/// Note the inner router here never contains the string "legacy".
async fn legacy_hello() -> &'static str {
    "legacy service: mounted with nest_service, no prefix knowledge\n"
}

async fn legacy_rest(Path(rest): Path<String>) -> String {
    format!("legacy service rest path: '{rest}' (prefix /legacy was stripped)\n")
}

// ============================================================================
// LESSON 7: any(), on(MethodFilter), explicit HEAD, and a typed Path<u32>
// ============================================================================

/// ANY /echo-method — `any(handler)` matches EVERY HTTP method.
///
/// What it does: the `Method` extractor (Module 05) tells us which one came.
/// Why it matters: `get/post/put/patch/delete/head/options/trace/connect` are
/// shortcuts for one method each; `any` is the "all of them" selector, handy
/// for proxies, webhooks and catch-all debug routes. Try
/// `curl -s -X PATCH .../echo-method`.
async fn echo_method(method: Method) -> String {
    format!("this row accepts any method; you sent {method}\n")
}

/// GET|POST /read-or-write — `on(filter, handler)` picks a SET of methods.
///
/// What it does: `MethodFilter::GET.or(MethodFilter::POST)` builds the set.
/// Why it matters: when two methods share one handler, `on` avoids writing
/// the handler twice. Any other method on this path still gets the 405.
async fn read_or_write(method: Method) -> String {
    format!("served by one handler for GET and POST; method = {method}\n")
}

/// GET /head-demo — what HEAD does by default, and how to override it.
///
/// What it does: with only `get(..)` registered, Axum answers HEAD itself by
/// running the GET handler and DROPPING the body (same status, same headers).
/// Why it matters: registering `.head(..)` explicitly (as done for this path)
/// lets you skip expensive work — a HEAD handler should not build a body at
/// all. Compare `curl -si /head-demo` with `curl -sI /head-demo`.
async fn head_demo_get() -> (StatusCode, [(&'static str, &'static str); 1], &'static str) {
    (StatusCode::OK, [("x-served-by", "GET handler")], "a body that HEAD never sees\n")
}

/// HEAD /head-demo — the explicit HEAD handler.
async fn head_demo_head() -> (StatusCode, [(&'static str, &'static str); 1]) {
    (StatusCode::OK, [("x-served-by", "explicit HEAD handler")])
}

/// GET /typed-items/{id} — the same row as `/items/{item}` but with a typed
/// parameter.
///
/// What it does: `Path<u32>` parses the segment for you; "abc" never reaches
/// the handler and Axum answers 400. Compare with `read_item`, which parses
/// by hand. Module 05 explains every Path form.
async fn typed_item(Path(id): Path<u32>) -> (StatusCode, String) {
    match item_store().lock().expect("store lock is fine").get(&id) {
        Some(name) => (StatusCode::OK, format!("typed item {id} is '{name}'\n")),
        None => (StatusCode::NOT_FOUND, format!("no item {id}\n")),
    }
}

// ============================================================================
// LESSON 8: trailing slashes, hand-written services, fallback_service
// ============================================================================

/// GET /docs and GET /docs/ — `route_with_tsr` ("trailing slash redirect").
///
/// What it does: `axum_extra::routing::RouterExt::route_with_tsr("/docs", ..)`
/// registers `/docs` AND a redirect from `/docs/` to `/docs`. (Register
/// `"/docs/"` instead and the redirect goes the other way.)
/// Why it matters: plain Axum treats `/docs` and `/docs/` as different paths
/// — `/docs/` would be a 404. Users and crawlers add slashes freely, so
/// public pages usually want the redirect. Try `curl -si .../docs/`.
async fn docs_page() -> Html<&'static str> {
    Html("<h1>Docs</h1><p>reached via /docs (and /docs/ redirects here)</p>")
}

/// A hand-written tower service: `Request -> Response`, never failing.
///
/// What it does: answers every request with the path it saw.
/// Why it matters: this is what `ServeDir`, a gRPC server or a reverse proxy
/// is — one async function from request to response with error type
/// `Infallible`. `route_service` mounts it on ONE path, `nest_service` on a
/// prefix, `fallback_service` on "everything else". Handlers are services
/// with extractors bolted on; here there are none, so you see the bare shape.
async fn hand_written_service(request: Request) -> Result<Response, Infallible> {
    let body = format!("hand-written service saw path '{}'\n", request.uri().path());
    let mut response = (StatusCode::OK, Body::from(body)).into_response();
    response
        .headers_mut()
        .insert("x-served-by", HeaderValue::from_static("service_fn"));
    Ok(response)
}
