//! Module 05: Extractors.
//! Topics: how a handler *reads* a request — `Path`, `Query`, `Json`, `Form`,
//! `HeaderMap`, `Bytes`, the full-request extractor, `Option<T>` and
//! `Result<T, Rejection>` forms, extractor order (body extractors must be
//! last), `OptionalFromRequest`, `MatchedPath`, and `ConnectInfo` for the
//! socket address.
//! Run: cargo run -p module-05-extractors   (port 3005)
//!
//! ## Lesson Reading Guide
//!
//! An extractor is any argument of a handler: Axum pulls the argument out of
//! the incoming request before the future even starts, and a *failed*
//! extraction short-circuits into a rejection response the client sees.
//! That second sentence is why extractors are a module of their own: the
//! same trait (`FromRequestParts` for headers-style data, `FromRequest` for
//! anything reading the body) is the whole framework's input contract, and
//! custom extractors in Module 09 implement exactly those two traits.
//!
//! - Lesson 1: `Path` — typed URL params, multi-segment tuples
//! - Lesson 2: `Query` — typed struct with required and optional fields
//! - Lesson 3: `Json` bodies — the request side of Module 03's response side
//! - Lesson 4: extractor ORDER — (Path, Query, ..., Json) with body last
//! - Lesson 5: `Form` — the browser-shaped body (x-www-form-urlencoded)
//! - Lesson 6: `Option<T>` extractors (`OptionalFromRequest` — implemented
//!   for `Path`, `Json`, `MatchedPath`, `Extension`) and
//!   `Result<T, Rejection>`. Honest note: `Option<Query<T>>` is NOT a thing
//!   in 0.8.9 — `Query` has no `OptionalFromRequestParts` impl yet, so an
//!   "optional query" is expressed through the struct's own Option fields
//!   (Lesson 2) or by reading `RawQuery`.
//! - Lesson 7: `HeaderMap`, `Bytes`, `MatchedPath` and `ConnectInfo`
//! - Lesson 8: the rest of the built-in kit — `Path<HashMap>`, `Path<struct>`,
//!   `RawPathParams`, `Query<HashMap>`, `RawQuery`, `Method`/`Uri`/`Version`,
//!   `NestedPath`, `OriginalUri`, `Extension`, `TypedHeader`, `RawForm`,
//!   and the whole `Request`
//! - Lesson 9: BUILD YOUR OWN — a custom `FromRequestParts` (`ApiKey`), its
//!   `Option<ApiKey>` form (`OptionalFromRequestParts`), a custom
//!   `FromRequest` that reads the body (`PlainLines`), a composed extractor
//!   (`ClientInfo`), and custom rejections
//!
//! Every extractor is one of exactly two traits. Know which one and you can
//! build any extractor:
//!
//! | trait                | gets                     | may read body | position       |
//! |----------------------|--------------------------|---------------|----------------|
//! | `FromRequestParts`   | `&mut Parts` + state     | no            | any argument   |
//! | `FromRequest`        | whole `Request` + state  | yes           | LAST argument  |

use axum::{
    body::Bytes,
    extract::{
        rejection::JsonRejection,
        rejection::{BytesRejection},
        ConnectInfo, Extension, Form, FromRequest, FromRequestParts, MatchedPath, NestedPath,
        OptionalFromRequestParts, OriginalUri, Path, Query, RawForm, RawPathParams, RawQuery,
        Request,
    },
    http::{header, request::Parts, HeaderMap, Method, StatusCode, Uri, Version},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use axum_extra::{headers::UserAgent, TypedHeader};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, net::SocketAddr};

/// Typed response payload for the JSON lesson (the `UserInput` round trip).
#[derive(Serialize)]
struct UserCreated {
    created: bool,
    username: String,
    age_years: u16,
}

/// The request body shape for the JSON lesson.
#[derive(Deserialize)]
struct UserInput {
    username: String,
    age: u16,
}

/// Typed query string for the search lesson: required `q`, optional rest.
#[derive(Deserialize)]
struct SearchParams {
    q: String,
    page: Option<u32>,
    sort: Option<String>,
}

/// Form payload for the browser-shaped body lesson.
#[derive(Deserialize)]
struct Feedback {
    name: String,
    rating: u8,
    comment: Option<String>,
}

/// Small JSON payload used by the optional-body handler.
#[derive(Deserialize)]
struct Greeting {
    message: String,
}

/// Small JSON payload used by the order lesson (extractor order).
#[derive(Deserialize)]
struct OrderLine {
    sku: String,
    quantity: u32,
}

// ============================================================================
// LESSON 1: Path — URL params, typed, parsed, and checked by the extractor
// ============================================================================

/// Every handler below is a normal async fn whose *arguments* are extractors.
///
/// What it does: registers the whole lesson set, binds port 3005, and serves
/// with `into_make_service_with_connect_info` — the small addition Lesson 7
/// needs so handlers can see the client's socket address, too.
/// Why it matters: this is the line from all the earlier modules: the route
/// table (Module 04) picks a handler; the handler's arguments pick what data
/// it reads. Everything below demonstrates one argument each.
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(dashboard))
        // ---- Lesson 1: Path -------------------------------------------
        .route("/products/{product_id}", get(product_by_id))
        .route("/users/{user_id}/posts/{post_id}", get(nested_params))
        // ---- Lesson 2: Query ------------------------------------------
        .route("/search", get(search))
        // ---- Lesson 3: Json bodies ------------------------------------
        .route("/users", post(create_user))
        // ---- Lesson 4: extractor order (Path + Query + Json) ----------
        .route("/orders/{order_id}", post(add_order_line))
        // ---- Lesson 5: Form -------------------------------------------
        .route("/feedback", post(feedback))
        // ---- Lesson 6: Option and Result extractors -------------------
        .route("/theme", get(theme_prefs))
        .route("/theme/{name}", get(theme_prefs))
        .route("/optional-json", post(optional_json))
        .route("/strict-json", post(strict_json))
        // ---- Lesson 7: headers, bytes, route info, client address ------
        .route("/headers", get(read_headers))
        .route("/echo-raw", post(echo_raw))
        .route("/meta/route", get(route_info))
        .route("/ip", get(client_ip))
        // ---- Lesson 8: the rest of the built-in kit ---------------------
        .route("/all-params/{org}/{repo}", get(all_params_map))
        .route("/repo-struct/{org}/{repo}", get(repo_struct))
        .route("/raw-params/{org}/{repo}", get(raw_params))
        .route("/query-map", get(query_map))
        .route("/raw-query", get(raw_query))
        .route("/uri-parts", get(uri_parts))
        .nest("/inner", Router::new().route("/where", get(nested_where)))
        .route("/extension", get(extension_demo))
        .route("/typed-header", get(typed_header_demo))
        .route("/raw-form", post(raw_form))
        .route("/whole-request", post(whole_request))
        // ---- Lesson 9: custom extractors --------------------------------
        .route("/custom/api-key", get(needs_api_key))
        .route("/custom/api-key-optional", get(maybe_api_key))
        .route("/custom/lines", post(plain_lines))
        .route("/custom/client", get(client_summary))
        // Extension values are attached as a layer (Module 07 compares this
        // with State).
        .layer(Extension(BuildInfo { version: "0.1.0", module: 5 }));

    let port = std::env::var("AXUM_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3005);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|error| panic!("could not bind port {port}: {error}"));

    println!("Module 05 (Extractors) listening on http://127.0.0.1:{port}");
    println!("Try it:  curl -s 'http://127.0.0.1:{port}/products/7'");
    // NOTE: the connect-info make service is required for ConnectInfo (Lesson 7).
    println!("         curl -s http://127.0.0.1:{port}/ip");

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("server stopped unexpectedly");
}

async fn dashboard() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "module": "05 Extractors",
        "lessons": [
            "GET /products/7                 Path<u32>",
            "GET /users/7/posts/3            Path<(u32, u32)>",
            "GET /search?q=axum              Query<SearchParams>",
            "POST /users {json}              Json<UserInput>",
            "POST /orders/5?q=..&.. {json}   order: Path, Query, Json-last",
            "POST /feedback {form}           Form<Feedback>",
            "GET /theme  GET /theme/dark     Option<Path>",
            "POST /optional-json [json]      Option<Json>",
            "POST /strict-json {json}        Result<Json, JsonRejection>",
            "GET /headers                    HeaderMap echo",
            "POST /echo-raw {any body}       Bytes extractor",
            "GET /meta/route                 MatchedPath",
            "GET /ip                         ConnectInfo<SocketAddr>",
            "GET /all-params/{org}/{repo}    Path<HashMap<String,String>>",
            "GET /repo-struct/{org}/{repo}   Path<struct>",
            "GET /raw-params/{org}/{repo}    RawPathParams (no parsing)",
            "GET /query-map?a=1&b=2          Query<HashMap>",
            "GET /raw-query?anything         RawQuery",
            "GET /uri-parts                  Method + Uri + Version",
            "GET /inner/where                NestedPath + OriginalUri",
            "GET /extension                  Extension + Option<Extension>",
            "GET /typed-header               TypedHeader<UserAgent>",
            "POST /raw-form                  RawForm",
            "POST /whole-request             Request (everything)",
            "GET /custom/api-key             CUSTOM FromRequestParts (x-api-key)",
            "GET /custom/api-key-optional    CUSTOM Option<ApiKey>",
            "POST /custom/lines              CUSTOM FromRequest (reads body)",
            "GET /custom/client              CUSTOM composed extractor"
        ]
    }))
}

// ---------------------------------------------------------------------------
// Lesson 1: Path
// ---------------------------------------------------------------------------

/// GET /products/{product_id} — one param, one typed argument.
///
/// What it does: binds the `{product_id}` segment (Module 04) into a `u32`.
/// A non-numeric segment never reaches the handler: `Path<u32>`'s rejection
/// answers 400 with the parse message.
/// Why it matters: the extractor contract turns URL text into *validated
/// types in the signature* — the whole point of the framework. Watch it fail
/// closed with `curl -i .../products/abc` and succeed with `.../products/7`.
async fn product_by_id(Path(product_id): Path<u32>) -> String {
    format!("Path<u32> extracted: product_id = {product_id}\n")
}

/// GET /users/{user_id}/posts/{post_id} — two segments, one tuple.
///
/// What it does: `Path<(u32, u32)>` maps segment-by-segment into the tuple;
/// order decides which parameter is which.
/// Why it matters: multi-param routes are ordinary REST (developers do
/// read `/users/7/posts/3`); the tuple form scales to any number of args a
/// route row actually has. Mixed types work too: `(u32, String)`.
async fn nested_params(Path((user_id, post_id)): Path<(u32, u32)>) -> String {
    format!("Path<(u32, u32)> extracted: user_id = {user_id}, post_id = {post_id}\n")
}

// ---------------------------------------------------------------------------
// Lesson 2: Query — typed query strings with real optionality
// ---------------------------------------------------------------------------

/// GET /search?q=..&page=..&sort=.. — one optional, one required field.
///
/// What it does: deserializes the query string into `SearchParams`; `q` is
/// required (missing -> 400 rejection), `page`/`sort` are `Option` fields
/// that simply stay `None` when the client omits them.
/// Why it matters: this is the professional way to take 2..20 query params:
/// one struct, serde does the coercing, missing keys surface as None, and
/// unknown keys are ignored. Next module's `garde`/validator turns the same
/// struct into one with rules (max length and such).
async fn search(Query(params): Query<SearchParams>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "q": params.q,
        "page": params.page.unwrap_or(1),
        "sort": params.sort.unwrap_or_else(|| "relevance".to_string()),
        "note": "page/sort were optional q was required",
    }))
}

// ---------------------------------------------------------------------------
// Lesson 3: Json — the request side of the body
// ---------------------------------------------------------------------------

/// POST /users — read a JSON body as a typed struct.
///
/// What it does: Json<UserInput> requires `content-type: application/json`,
/// buffers the body, and deserializes into the struct; then the handler
/// answers 201 with the derived view (Module 03's tuple style).
/// Why it matters: this is `Json` from the response lessons, coming back in
/// as an argument. Two realistic failure cases for `curl` practice:
/// content-type missing/wrong -> 415 Unsupported Media Type; text that is not
/// valid JSON (or trailing garbage after valid JSON — a real 0.8 change) -> 400
/// Bad Request; valid JSON of the wrong shape (missing field, wrong type) ->
/// 422 Unprocessable Entity. The rejections happen before any of your code runs.
async fn create_user(Json(input): Json<UserInput>) -> (StatusCode, Json<UserCreated>) {
    (
        StatusCode::CREATED,
        Json(UserCreated {
            created: true,
            username: input.username,
            age_years: input.age,
        }),
    )
}

// ---------------------------------------------------------------------------
// Lesson 4: Extractor order — Path, Query, Json(body) — body LAST
// ---------------------------------------------------------------------------

/// POST /orders/{order_id}?priority=.. — three extractors, one signature.
///
/// What it does: reads the path param, then the query, then the JSON body —
/// in that declared order, and the compiler will refuse any body extractor
/// that is not last.
/// Why it matters: extractors run in argument order; anything reading the
/// body needs the whole request, so Axum's `FromRequest` is only allowed
/// once and only as the final argument. A tuple of a body extractor must
/// own the last slot: `(Path(..), Query(..), Json(..))`, never reordered.
/// All three rejections can be observed in Try It.
async fn add_order_line(
    Path(order_id): Path<u32>,
    Query(priority): Query<OrderPriority>,
    Json(line): Json<OrderLine>,
) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "order_id": order_id,
        "priority": priority.priority.unwrap_or_else(|| "normal".to_string()),
        "sku": line.sku,
        "quantity": line.quantity,
        "note": "three extractors: Path then Query then Json (body)",
    }))
}

/// The query struct for the order lesson: single optional field.
#[derive(Deserialize)]
struct OrderPriority {
    priority: Option<String>,
}

// ---------------------------------------------------------------------------
// Lesson 5: Form — the browser body (application/x-www-form-urlencoded)
// ---------------------------------------------------------------------------

/// POST /feedback — a classic form endpoint.
///
/// What it does: `Form<Feedback>` accepts `name=A&rating=5&comment=...`
/// (exactly what `curl -d name=A -d rating=5` sends, and what HTML `<form>`
/// posts to servers as `application/x-www-form-urlencoded`). Same 415/422
/// rejection family as `Json`.
/// Why it matters: before HTMX arrives in Module 13, this is THE shape
/// browser forms post — one extractor, zero extra dependencies. Optional
/// field handling is identical to the query structs from Lesson 2.
async fn feedback(form: Form<Feedback>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "received_from": form.name,
        "rating": form.rating,
        "comment": form.comment,
        "note": "Form extractor: application/x-www-form-urlencoded",
    }))
}

// ---------------------------------------------------------------------------
// Lesson 6: Option<T> extractors and Result<T, Rejection> (tolerant reading)
// ---------------------------------------------------------------------------

/// GET /theme (no path) and GET /theme/{name} — one handler, two rows.
///
/// What it does: `Option<Path<String>>` becomes `None` when the route row
/// had **zero** path parameters to extract (the `/theme` row) and `Some`
/// when the segment exists (`/theme/{name}`).
/// Why it matters: this is the 0.8 way to write "the param may be absent":
/// the handler is mounted on both rows and stays readable. `Query` has no
/// optional extractor yet (see the header note), so this lesson builds the
/// pattern on `Path`, where `OptionalFromRequestParts` genuinely exists in
/// 0.8.9. Wrong-typed segments (`/theme/4x`? still String here) stay fine —
/// optionality is about *absence*, not parse errors.
async fn theme_prefs(path: Option<Path<String>>) -> Json<serde_json::Value> {
    match path {
        Some(Path(name)) => Json(serde_json::json!({
            "chosen_theme": name,
            "note": "the path segment existed — Some",
        })),
        None => Json(serde_json::json!({
            "chosen_theme": "default",
            "note": "route row had no params — Option<Path> was None",
        })),
    }
}

/// POST /optional-json — body present or absent, one handler.
///
/// What it does: `Option<Json<Greeting>>` is None when the JSON content type is
/// MISSING; a JSON content type with a broken or empty body is still a 400
/// rejection ("optional" means absent, not forgiven).
/// Why it matters: the price of optional bodies used to be unreadable
/// extractors; 0.8's `OptionalFromRequest` makes `Option<Json<T>>` plain
/// Rust again. Useful for "UI may or may not send the field" endpoints and
/// PATCH-style partial updates (Module 18).
async fn optional_json(body: Option<Json<Greeting>>) -> String {
    match body {
        Some(Json(greeting)) => format!("you said: {}\n", greeting.message),
        None => "no body sent — Option<Json> was None, so no rejection\n".to_string(),
    }
}

/// POST /strict-json — the *tolerant* way to handle bad bodies in place.
///
/// What it does: `Result<Json<Greeting>, JsonRejection>` lets the handler
/// see the exact rejection (unsupported media type vs syntax error vs data
/// error) and answer a coherent 422 every time.
/// Why it matters: `Result` turns rejection failures into values; decide, log
/// or re-emit. The general shape is `Result<T, E> where T: FromRequest and
/// E: From<T::Rejection>` — the concrete `JsonRejection` here comes from
/// axum::extract::rejection (Module 08 keeps the habit but with one custom
/// `AppError` on every handler instead).
async fn strict_json(body: Result<Json<Greeting>, JsonRejection>) -> (StatusCode, String) {
    match body {
        Ok(Json(greeting)) => (StatusCode::OK, format!("strict ok: {}\n", greeting.message)),
        Err(rejection) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("rejection made into a value: {rejection}\n"),
        ),
    }
}

// ---------------------------------------------------------------------------
// Lesson 7: HeaderMap, Bytes, MatchedPath, ConnectInfo — the rest of the kit
// ---------------------------------------------------------------------------

/// GET /headers — echoes every header the client actually sent.
///
/// What it does: takes the whole `HeaderMap` and formats it.
/// Why it matters: `HeaderMap` is the raw set (auth tokens and user agents
/// arrive here until Module 09's typed `TypedHeader` from axum-extra). Any
/// extractor can coexist with others: add `Path`/`Query` args and the map
/// still comes whole — headers never consume the body.
async fn read_headers(headers: HeaderMap) -> String {
    let mut listing = String::from("headers the client sent:\n");
    for (name, value) in headers.iter() {
        if let Ok(value) = value.to_str() {
            listing.push_str(&format!("  {name}: {value}\n"));
        }
    }
    listing
}

/// POST /echo-raw — the raw bytes extractor.
///
/// What it does: binds the body directly as `Bytes` and echoes it back.
/// Why it matters: the rawest body access without parsing anything — proof
/// that extractors are just typed views over request parts. Modules 26-27
/// build on this mental model with real streaming (`Body::from_stream`,
/// `Multipart`); today the point is the signature: `async fn(body: Bytes)`.
async fn echo_raw(body: Bytes) -> Bytes {
    body
}

/// GET /meta/route — which row of the route tree matched.
///
/// What it does: `MatchedPath` reports the *pattern* that matched,
/// independent of the concrete URL.
/// Why it matters: metrics, rate limits, and per-route config want "which
/// endpoint" — not "which concrete path" (or cardinality explodes across
/// customers, since Browse `/products/42` and `/products/43` collapse to one
/// row name here). Module 37's tracing spans read the same value.
async fn route_info(path: MatchedPath) -> String {
    format!("matched route pattern: {}\n", path.as_str())
}

/// GET /ip — the client's socket address.
///
/// What it does: `ConnectInfo<SocketAddr>` returns the peer address; see
/// `main` for the required `.into_make_service_with_connect_info` — without
/// that plumbing, this extractor rejects at startup with a clear error.
/// Why it matters: behind real deployments the direct IP is the *proxy's*
/// (`X-Forwarded-For` rules, Module 32); but local services, admin APIs and
/// rate limit keys still use it daily (Module 17).
async fn client_ip(ConnectInfo(info): ConnectInfo<SocketAddr>) -> String {
    format!(
        "client socket: {info}\n(behind a proxy this is the proxy — Module 32 covers headers)\n"
    )
}

// ============================================================================
// LESSON 8: The rest of the built-in kit (one handler per extractor)
// ============================================================================

/// The struct for `Path<RepoRef>`: field names must match the `{param}` names.
#[derive(Deserialize)]
struct RepoRef {
    org: String,
    repo: String,
}

/// GET /all-params/{org}/{repo} — `Path<HashMap<String, String>>`.
///
/// What it does: collects EVERY path parameter by name, as strings.
/// Why it matters: when a route's parameters are dynamic or numerous you do
/// not need a tuple or struct. No parsing happens — values stay strings.
async fn all_params_map(Path(params): Path<HashMap<String, String>>) -> Json<HashMap<String, String>> {
    Json(params)
}

/// GET /repo-struct/{org}/{repo} — `Path<struct>`.
///
/// What it does: serde maps parameter NAMES onto struct fields, so the order
/// in the signature no longer matters (unlike the tuple form).
/// Why it matters: structs are self-documenting and survive route edits.
async fn repo_struct(Path(repo): Path<RepoRef>) -> String {
    format!("org = {}, repo = {}\n", repo.org, repo.repo)
}

/// GET /raw-params/{org}/{repo} — `RawPathParams`.
///
/// What it does: iterates `(name, value)` pairs exactly as they appeared in
/// the URL — percent-decoded, never parsed into types.
/// Why it matters: use it in middleware or generic code that must work for
/// routes whose parameter names it does not know.
async fn raw_params(params: RawPathParams) -> String {
    let mut out = String::from("raw path params:\n");
    for (name, value) in &params {
        out.push_str(&format!("  {name} = {value}\n"));
    }
    out
}

/// GET /query-map?a=1&b=2 — `Query<HashMap<String, String>>`.
///
/// What it does: accepts ANY query keys. Why it matters: perfect for
/// pass-through filters; remember there is no validation at all. (Repeated
/// keys keep only the last value — for `?tag=a&tag=b` use
/// `axum_extra::extract::Query` with a `Vec`.)
async fn query_map(Query(params): Query<HashMap<String, String>>) -> Json<HashMap<String, String>> {
    Json(params)
}

/// GET /raw-query?anything — `RawQuery`.
///
/// What it does: hands over the un-parsed query string as `Option<String>`
/// (`None` when there is no `?` at all). Why it matters: the escape hatch for
/// custom parsing, and the honest answer to "optional query" in 0.8.9.
async fn raw_query(RawQuery(query): RawQuery) -> String {
    match query {
        Some(text) => format!("raw query string: '{text}'\n"),
        None => "no query string at all (RawQuery was None)\n".to_string(),
    }
}

/// GET /uri-parts — `Method`, `Uri`, `Version`.
///
/// What it does: three tiny extractors for the request line itself.
/// Why it matters: `Uri` inside a nested router shows the path WITHOUT the
/// nest prefix; `OriginalUri` (below) shows the full one.
async fn uri_parts(method: Method, uri: Uri, version: Version) -> String {
    format!("method = {method}\npath   = {}\nquery  = {:?}\nhttp   = {version:?}\n", uri.path(), uri.query())
}

/// GET /inner/where — `NestedPath` and `OriginalUri` inside a nested router.
///
/// What it does: `Uri` sees `/where`; `OriginalUri` sees `/inner/where`;
/// `NestedPath` reports the prefix the router was nested under (`/inner`).
/// Why it matters: redirects and link building inside a reusable sub-router
/// need the prefix without hard-coding it.
async fn nested_where(nested: NestedPath, OriginalUri(original): OriginalUri, uri: Uri) -> String {
    format!(
        "NestedPath  = {}\nOriginalUri = {original}\nUri         = {uri}\n",
        nested.as_str()
    )
}

/// Data attached by `.layer(Extension(..))` in `main`.
#[derive(Clone)]
struct BuildInfo {
    version: &'static str,
    module: u8,
}

/// A type that is NEVER attached, so the optional form always sees `None`.
#[derive(Clone)]
struct NeverAttached;

/// GET /extension — `Extension<T>` and `Option<Extension<T>>`.
///
/// What it does: reads a value another layer put into the request. The
/// optional form turns "nobody attached it" into `None` instead of a 500.
/// Why it matters: middleware (Module 10) produces Extensions — the current
/// user, a request id — for handlers further in. Module 07 explains when to
/// prefer `State`.
async fn extension_demo(
    Extension(info): Extension<BuildInfo>,
    missing: Option<Extension<NeverAttached>>,
) -> String {
    format!(
        "BuildInfo: version {}, module {}\nNeverAttached present: {}\n",
        info.version,
        info.module,
        missing.is_some()
    )
}

/// GET /typed-header — `TypedHeader<UserAgent>` from axum-extra.
///
/// What it does: parses the header into a real type (the `headers` crate).
/// Why it matters: no string handling and the header-specific parse rules
/// (e.g. `Authorization<Bearer>` in Module 19) come free. A missing header is
/// a 400 rejection; use `Option<TypedHeader<..>>` to tolerate absence.
async fn typed_header_demo(TypedHeader(agent): TypedHeader<UserAgent>) -> String {
    format!("you are: {}\n", agent.as_str())
}

/// POST /raw-form — `RawForm`.
///
/// What it does: gives the form body as `Bytes` without deserialising.
/// Why it matters: signature checks (webhooks) need the exact bytes sent.
async fn raw_form(RawForm(bytes): RawForm) -> String {
    format!("raw form body ({} bytes): {}\n", bytes.len(), String::from_utf8_lossy(&bytes))
}

/// POST /whole-request — the entire `Request`.
///
/// What it does: takes method, headers and the unread body at once.
/// Why it matters: `Request` is itself a `FromRequest` extractor — the
/// "give me everything" end of the spectrum, used by middleware and proxies.
async fn whole_request(request: Request) -> String {
    let (parts, body) = request.into_parts();
    let size = axum::body::to_bytes(body, 1024 * 1024).await.map(|b| b.len()).unwrap_or(0);
    format!("{} {} with {} header(s) and {} body byte(s)\n", parts.method, parts.uri, parts.headers.len(), size)
}

// ============================================================================
// LESSON 9: BUILD YOUR OWN extractors
// ============================================================================
//
// Recipe: (1) make a type, (2) make a rejection type that implements
// `IntoResponse`, (3) implement ONE of the two traits. In 0.8 there is no
// `#[async_trait]` — write `async fn` directly in the impl.

// ---------------------------------------------------------------------------
// 9a. Custom FromRequestParts — read a header, never touch the body
// ---------------------------------------------------------------------------

/// A validated API key taken from the `x-api-key` header.
///
/// What it does: after extraction the handler holds an `ApiKey` that is known
/// to be well-formed (starts with `ak_`, at least 8 characters).
/// Why it matters: "parse, don't validate" — the check happens once, in the
/// extractor, so every handler that takes `ApiKey` is safe by its signature.
struct ApiKey(String);

/// Why an API key was refused. Implements `IntoResponse` so it can be a
/// `Rejection`.
///
/// What it does: each variant maps to a status + JSON body.
/// Why it matters: a rejection is just an `IntoResponse` type; Module 08
/// shows how to funnel these into one app-wide error.
enum ApiKeyRejection {
    Missing,
    Malformed,
}

impl IntoResponse for ApiKeyRejection {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiKeyRejection::Missing => (StatusCode::UNAUTHORIZED, "missing x-api-key header"),
            ApiKeyRejection::Malformed => (StatusCode::BAD_REQUEST, "x-api-key must look like ak_xxxxxxxx"),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// The extractor itself.
///
/// What it does: reads `parts.headers`, checks the shape, returns the key.
/// Why it matters: `S` is the app state; `Send + Sync` is the only bound when
/// you do not use it. `&mut Parts` is why this trait cannot read the body.
impl<S> FromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = ApiKeyRejection;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts
            .headers
            .get("x-api-key")
            .ok_or(ApiKeyRejection::Missing)?
            .to_str()
            .map_err(|_| ApiKeyRejection::Malformed)?;
        if value.starts_with("ak_") && value.len() >= 8 {
            Ok(ApiKey(value.to_string()))
        } else {
            Err(ApiKeyRejection::Malformed)
        }
    }
}

/// The OPTIONAL form: makes `Option<ApiKey>` work in 0.8.
///
/// What it does: header absent -> `Ok(None)`; header present but bad ->
/// still an error. Why it matters: without this impl, `Option<ApiKey>` does
/// not compile in 0.8 (the old "swallow every rejection" behaviour was
/// removed on purpose).
impl<S> OptionalFromRequestParts<S> for ApiKey
where
    S: Send + Sync,
{
    type Rejection = ApiKeyRejection;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Option<Self>, Self::Rejection> {
        if parts.headers.contains_key("x-api-key") {
            <ApiKey as FromRequestParts<S>>::from_request_parts(parts, state).await.map(Some)
        } else {
            Ok(None)
        }
    }
}

/// GET /custom/api-key — requires a valid key.
async fn needs_api_key(ApiKey(key): ApiKey) -> String {
    format!("welcome, key {key}\n")
}

/// GET /custom/api-key-optional — works with or without a key.
async fn maybe_api_key(key: Option<ApiKey>) -> String {
    match key {
        Some(ApiKey(key)) => format!("hello key holder {key}\n"),
        None => "hello anonymous visitor\n".to_string(),
    }
}

// ---------------------------------------------------------------------------
// 9b. Custom FromRequest — read the BODY (must be the last argument)
// ---------------------------------------------------------------------------

/// A `text/plain` body split into non-empty lines.
///
/// What it does: checks the content-type, reads the bytes, decodes UTF-8 and
/// splits. Why it matters: the pattern for any custom body format — CSV,
/// XML, MessagePack, protobuf — buffer with `Bytes`, then parse.
struct PlainLines(Vec<String>);

/// Everything that can go wrong while building `PlainLines`.
enum PlainLinesRejection {
    WrongContentType,
    BadBody(BytesRejection),
    NotUtf8,
}

impl IntoResponse for PlainLinesRejection {
    fn into_response(self) -> Response {
        match self {
            PlainLinesRejection::WrongContentType => (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "expected content-type: text/plain\n",
            )
                .into_response(),
            // Reuse the built-in rejection's own status and text.
            PlainLinesRejection::BadBody(rejection) => rejection.into_response(),
            PlainLinesRejection::NotUtf8 => (StatusCode::BAD_REQUEST, "body is not valid UTF-8\n").into_response(),
        }
    }
}

impl<S> FromRequest<S> for PlainLines
where
    S: Send + Sync,
{
    type Rejection = PlainLinesRejection;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        let is_text = request
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.starts_with("text/plain"));
        if !is_text {
            return Err(PlainLinesRejection::WrongContentType);
        }
        // Delegate the actual reading (and the body-size limit) to Bytes.
        let bytes = Bytes::from_request(request, state)
            .await
            .map_err(PlainLinesRejection::BadBody)?;
        let text = std::str::from_utf8(&bytes).map_err(|_| PlainLinesRejection::NotUtf8)?;
        Ok(PlainLines(
            text.lines().map(str::trim).filter(|line| !line.is_empty()).map(String::from).collect(),
        ))
    }
}

/// POST /custom/lines — counts the lines of a text/plain body.
async fn plain_lines(PlainLines(lines): PlainLines) -> Json<serde_json::Value> {
    Json(serde_json::json!({ "line_count": lines.len(), "lines": lines }))
}

// ---------------------------------------------------------------------------
// 9c. A composed extractor — build big extractors out of small ones
// ---------------------------------------------------------------------------

/// Who is calling: the socket address plus the user-agent header.
///
/// What it does: calls two existing extractors from inside our own.
/// Why it matters: extractors compose. Handlers stay one argument long, and
/// the combination lives in one place (Module 09 replaces this hand-written
/// version with `#[derive(FromRequestParts)]`).
struct ClientInfo {
    addr: SocketAddr,
    user_agent: String,
}

impl<S> FromRequestParts<S> for ClientInfo
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ConnectInfo(addr) = ConnectInfo::<SocketAddr>::from_request_parts(parts, state)
            .await
            .map_err(IntoResponse::into_response)?;
        let user_agent = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown")
            .to_string();
        Ok(ClientInfo { addr, user_agent })
    }
}

/// GET /custom/client — one argument instead of two.
async fn client_summary(client: ClientInfo) -> String {
    format!("{} using {}\n", client.addr, client.user_agent)
}
