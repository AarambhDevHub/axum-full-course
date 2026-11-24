//! Module 06: Responses in depth.
//! Topics: `IntoResponse` (the trait every response type understands),
//! `IntoResponseParts` (the trait for response *add-ons*: headers, cookies),
//! building a `Response` by hand with `Response::builder`, status+headers+
//! body tuples, `AppendHeaders`, custom response types, hand-set cookies,
//! content negotiation by `Accept`, and file-download headers.
//! Run: cargo run -p module-06-responses   (port 3006)
//!
//! ## Lesson Reading Guide
//!
//! Module 03 introduced the built-in response types; this module teaches you
//! to build your own. The two traits sit in `axum::response`:
//! - `IntoResponse` — "I can *become* a full Response" (the body side).
//! - `IntoResponseParts` — "I can *decorate* an existing Response" (status,
//!   headers). Every tuple element before the last one is a part; the last
//!   element is the body.
//!
//! Read top to bottom with `curl -i` open: every lesson is visible in
//! headers, not just bodies.
//!
//! - Lesson 1: a whole `Response` built by hand (`Response::builder`)
//! - Lesson 2: a custom response type implementing `IntoResponse`
//! - Lesson 3: the parts chain — status, header tuples, `AppendHeaders`,
//!   and a hand-written `IntoResponseParts`
//! - Lesson 4: a hand-set cookie (the wire format; the jar comes in Module 21)
//! - Lesson 5: content negotiation by the `Accept` header
//! - Lesson 6: file-download headers (`Content-Disposition`, MIME guess)
//! - Lesson 7: the built-in response kit, explained (`/response-kit`) plus
//!   `201 Created` + `Location`
//! - Lesson 8: BUILD YOUR OWN response types — `Created<T>`, `ApiEnvelope<T>`,
//!   `Csv`
//! - Lesson 9: BUILD YOUR OWN response parts — a typed `CacheControl` and a
//!   *fallible* part (`LinkHeader`)
//! - Lesson 10: a streaming body (`Body::from_stream`)

use axum::{
    body::{Body, Bytes},
    extract::Path,
    http::{header, HeaderName, HeaderValue, StatusCode},
    response::{
        AppendHeaders, Html, IntoResponse, IntoResponseParts, Json, Response, ResponseParts,
    },
    routing::get,
    Router,
};
use futures_util::stream;
use serde::Serialize;
use std::{convert::Infallible, time::Duration};

// ============================================================================
// LESSON 1: A whole Response, built by hand
// ============================================================================

/// The entry point (final version of the familiar skeleton).
///
/// What it does: registers the six lessons, binds port 3006, serves forever.
/// Why it matters: Module 05's extracting peer — here you control what
/// leaves. `IntoResponse` is the contract every route must satisfy; the
/// whole module is different ways of satisfying it.
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(dashboard))
        .route("/built", get(hand_built))
        // /version/{name} proves the custom type works with a route that has params.
        .route("/version/{name}", get(custom_type))
        .route("/stamped", get(stamped_headers))
        .route("/cookies", get(etched_cookie))
        .route("/negotiate", get(negotiate))
        .route("/download/{filename}", get(download))
        // ---- Lesson 7-10 ------------------------------------------------
        .route("/response-kit", get(response_kit))
        .route("/items/{id}", get(create_with_location))
        .route("/envelope", get(envelope_demo))
        .route("/report.csv", get(csv_report))
        .route("/cache/{mode}", get(cache_demo))
        .route("/fallible-part/{value}", get(fallible_part))
        .route("/stream", get(stream_demo));

    let port = std::env::var("AXUM_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(3006);
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|error| panic!("could not bind port {port}: {error}"));

    println!("Module 06 (Responses in depth) listening on http://127.0.0.1:{port}");
    println!("Try it:  curl -si http://127.0.0.1:{port}/built");

    axum::serve(listener, app).await.expect("server stopped unexpectedly");
}

async fn dashboard() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "module": "06 Responses in depth",
        "lessons": [
            "GET /built             Response::builder, status 202, custom header",
            "GET /version/{name}    custom IntoResponse type",
            "GET /stamped           header tuples + AppendHeaders + custom part",
            "GET /cookies           hand-forged Set-Cookie",
            "GET /negotiate         Accept: negotiation JSON/HTML/text",
            "GET /download/notes.md attachment headers",
            "GET /response-kit      every built-in response type, explained",
            "GET /items/{id}        201 Created + Location (custom Created<T>)",
            "GET /envelope          custom ApiEnvelope<T> response type",
            "GET /report.csv        custom Csv response type",
            "GET /cache/{mode}      custom IntoResponseParts: CacheControl (no-store|public|private)",
            "GET /fallible-part/{v} custom FALLIBLE IntoResponseParts",
            "GET /stream            streaming body, one chunk every 300 ms"
        ]
    }))
}

/// GET /built — the full Response, assembled from raw parts.
///
/// What it does: builds the response piece by piece: status first, then
/// headers, then the `Body::from`, and hands the result back as `Response`.
/// Why it matters: this is what `IntoResponse` types expand into anyway;
/// seeing it once makes all of them feel like convenience. Custom builders
/// matter when you need exact bytes streaming, unusual headers, or something
/// the tuple sugar can't type (Module 26's `Body::from_stream`, Module 44's
/// service-internals). The `.unwrap()` here is on a *header value* — invalid
/// bytes fail at build time, not on the wire; Module 08 maps real errors.
async fn hand_built() -> Response {
    Response::builder()
        .status(StatusCode::ACCEPTED)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .header("x-built-by", "response::builder in an axum handler")
        .body(Body::from(
            "This response never used any helper: raw builder, raw body.\n",
        ))
        .expect("valid status and headers cannot fail before the server runs")
}

// ============================================================================
// LESSON 2: A custom response type (impl IntoResponse for your wrapper)
// ============================================================================

/// Typed payload that goes out with an API-version header on every response.
///
/// What it does: `ApiPayload` turns any handler's data into
/// "(x-api-version header, JSON body)" — one uniform envelope that can
/// carry whatever the handler computed.
/// Why it matters: a module-06-native way to add uniform metadata (API
/// versions, deprecation warnings) without repeating header lines in every
/// handler: the response format is code, not convention.
#[derive(Serialize)]
struct ApiPayload {
    served_to: String,
    payload_len: usize,
}

impl ApiPayload {
    /// The (parts, body) tuple this response type always produces.
    fn served(served_to: &str, payload_len: usize) -> ([(HeaderName, HeaderValue); 1], Json<ApiPayload>) {
        (
            [(
                HeaderName::from_static("x-api-version"),
                HeaderValue::from_static("v1"),
            )],
            Json(ApiPayload {
                served_to: served_to.to_string(),
                payload_len,
            }),
        )
    }
}

/// GET /version/{name} — the custom envelope at work.
///
/// What it does: builds `([(headers; 1)], Json(...))` via the factory above
/// and returns it — the pair is exactly a (parts, body) tuple.
/// Why it matters: every response in the course is just parts + one body;
/// a custom response type simply standardizes ONE bundle. No extra
/// `impl IntoResponse` is needed because the tuple itself is already
/// `IntoResponse` when every element is.
async fn custom_type(Path(name): Path<String>) -> ([(HeaderName, HeaderValue); 1], Json<ApiPayload>) {
    ApiPayload::served(&name, name.len())
}

// ============================================================================
// LESSON 3: The parts chain — status + header tuples + a custom IntoResponseParts
// ============================================================================

/// A hand-written response part: sets `X-Trace-Id` on any response.
///
/// What it does: implements `IntoResponseParts` — the trait for response
/// *decorations* — and adds one header to whatever response it decorates.
/// Why it matters: this is the same toolbox `tower-http`'s header layers
/// (set_header, request id) use under the hood (Module 12); hand-rolling
/// one is a few lines and gives full wire control. Use `AppendHeaders` for
/// one-off headers and a part like this when the decoration is reusable.
/// The error type: `Infallible` — this part can never fail, because an
/// invalid trace id would be replaced, not rejected. (Parts that CAN fail
/// return their own error type, which must itself be `IntoResponse`.)
struct TraceId {
    id: String,
}

impl IntoResponseParts for TraceId {
    // IntoResponse + infallible: this decoration never fails.
    type Error = std::convert::Infallible;

    fn into_response_parts(
        self,
        mut res: axum::response::ResponseParts,
    ) -> Result<axum::response::ResponseParts, Self::Error> {
        // headers_mut: ResponseParts exposes the map for mutation; insert
        // returns the PREVIOUS value (like HashMap::insert) and is a
        // statement here — the ResponseParts itself is what we hand back.
        res.headers_mut().insert(
            HeaderName::from_static("x-trace-id"),
            HeaderValue::from_str(&self.id)
                .unwrap_or_else(|_| HeaderValue::from_static("trace-unavailable")),
        );
        Ok(res)
    }
}

/// GET /stamped — four decoration layers in one tuple.
///
/// What it does: returns `(StatusCode, TraceId, AppendHeaders<...>,
/// &[u8])`: status, custom part, standard-append multi-header, body — the
/// largest tuple demos go.
/// Why it matters: the tuple reads like a receipt: each element adds
/// something, last is the body. `AppendHeaders` shows headers when
/// duplicates are wanted (Set-Cookie is the classic multi); the custom part
/// is invisible until you diff curl output.
/// The body: since scratch doc comments referenced it, the helper insists:
/// only the last tuple slot is a body — everything else is parts.
async fn stamped_headers() -> (
    StatusCode,
    TraceId,
    AppendHeaders<[(HeaderName, HeaderValue); 2]>,
    &'static str,
) {
    (
        StatusCode::IM_A_TEAPOT,
        TraceId { id: "trace-42".to_string() },
        AppendHeaders([
            (
                HeaderName::from_static("x-deprecation"),
                HeaderValue::from_static("sunsets in v2"),
            ),
            (
                HeaderName::from_static("x-course-module"),
                HeaderValue::from_static("06"),
            ),
        ]),
        "one body; headers: x-trace-id, 2 appended (x-deprecation, x-course-module), content-type\n",
    )
}

// ============================================================================
// LESSON 4: Hand-forged cookies (the raw wire format)
// ============================================================================

/// GET /cookies — Set-Cookie by hand, the way the jar will do it in Module 21.
///
/// What it does: writes the raw `Set-Cookie: visitor=...; Path=/; Max-Age=..`
/// header; the browser keeps the pair for a day.
/// Why it matters: cookies are just a header until Module 21 teaches how
/// axum-extra's CookieJar wraps them with signing and privacy. Seeing the
/// format once demystifies the domain=, the Path=/, and the Max-Age contract
/// you'll sign away in modules ahead. Multiple Set-Cookie lines stack via
/// AppendHeaders (Try It hints at it).
async fn etched_cookie() -> (AppendHeaders<[(HeaderName, HeaderValue); 1]>, &'static str) {
    (
        AppendHeaders([(
            header::SET_COOKIE,
            // The name=value pair is all a cookie IS; attributes configure it.
            HeaderValue::from_static("visitor=yes; Path=/; Max-Age=86400; HttpOnly"),
        )]),
        "handed you a cookie named 'visitor'. curl -si shows Set-Cookie;\n\
         your browser would store it; curl saves to a jar file with -c.\n",
    )
}

// ============================================================================
// LESSON 5: Content negotiation (the Accept header decides the format)
// ============================================================================

/// GET /negotiate — same data, three encodings, decided by Accept.
///
/// What it does: reads `Accept` from the request, chooses Json (applications)
/// / Html (browsers) / plain text (curl default), and returns accordingly.
/// Why it matters: REST thinkers call this "content negotiation": one
/// endpoint, multiple representations. It's a HAND-ROLLED negotiation (the
/// cfg/branch parse is manual string matching, deliberately) — real apps use
/// middleware or utoipa-style empowerment (Module 33's docs). The response
/// side you know already; accepting `Accept` is the extractor review here.
async fn negotiate(headers: axum::http::HeaderMap) -> Response {
    let accept = headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();

    let data = serde_json::json!({ "course": "Axum", "module": 6 });

    if accept.contains("application/json") {
        Json(data).into_response()
    } else if accept.contains("text/html") {
        Html("<h1>Axum module 6</h1><p>you asked for HTML, so here it is</p>").into_response()
    } else {
        format!("negotiation says: plain text (Accept was '{accept}')\n").into_response()
    }
}

// ============================================================================
// LESSON 6: File downloads (MIME + Content-Disposition, the safe filename)
// ============================================================================

/// GET /download/{filename} — attachment headers that force "save as".
///
/// What it does: streams a small "file" (in-memory bytes here; Module 26/29
/// stream from disk) as `application/octet-stream` with a Content-Disposition
/// of `attachment; filename="..."`.
/// Why it matters: `attachment` is what makes browsers offer "Save as"
/// instead of rendering; the explicit MIME type prevents sniffing. The
/// filename is echoed from the URL — in production, validate or guess it
/// with the `mime_guess` crate (Module 27) and NEVER trust it wholesale
/// (path-as-header injection: quotes are real-world effort, Module 23).
async fn download(Path(filename): Path<String>) -> Response {
    let file_bytes = "These are the course notes for module 06.
Responses are code too: into_response() for the body,
into_response_parts() for the decorations.\n";

    // Header values cannot contain newlines or control chars. This handler
    // defends the CONTACT surface: strip anything outside classic alphanum,
    // dot or dash before the name goes on the wire.
    let safe_filename: String = filename
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
        .collect();

    let disposition = format!("attachment; filename=\"{safe_filename}\"");
    let ok_disposition: Result<HeaderValue, _> = disposition.parse();

    match ok_disposition {
        Ok(value) => (
            StatusCode::OK,
            [
                (header::CONTENT_DISPOSITION, value),
                (header::CONTENT_TYPE, HeaderValue::from_static("application/octet-stream")),
            ],
            file_bytes,
        )
            .into_response(),
        // Fallback if the browser sent something header-hostile (the parse
        // above would fail): answer plain, no disposition, same status.
        Err(_) => (StatusCode::OK, Body::from(file_bytes)).into_response(),
    }
}

// ============================================================================
// LESSON 7: The built-in response kit, explained
// ============================================================================

/// One built-in thing a handler can return (or put in a response tuple).
#[derive(Serialize)]
struct KitRow {
    kind: &'static str,
    value: &'static str,
    becomes: &'static str,
}

/// GET /response-kit — the "explain every built-in" reference.
///
/// What it does: lists what each return value turns into.
/// Why it matters: a handler's return type IS the response definition. Two
/// roles exist in a tuple: PARTS (status, headers, extensions — anything that
/// implements `IntoResponseParts`) and the BODY (anything `IntoResponse`,
/// always last). `(StatusCode, parts.., body)` is the full grammar.
async fn response_kit() -> Json<Vec<KitRow>> {
    let row = |kind, value, becomes| KitRow { kind, value, becomes };
    Json(vec![
        row("body", "&'static str, String, Cow<str>, Box<str>", "200, text/plain; charset=utf-8"),
        row("body", "Vec<u8>, &'static [u8], Bytes, Cow<[u8]>", "200, application/octet-stream"),
        row("body", "Html<T>", "200, text/html; charset=utf-8"),
        row("body", "Json<T: Serialize>", "200, application/json (500 if serialisation fails)"),
        row("body", "Body (Body::from_stream / from(..))", "200, you choose headers; streams chunk by chunk"),
        row("body", "Response / Response<B>", "sent as-is"),
        row("body", "NoContent", "204, no body"),
        row("body", "Redirect", "303 / 307 / 308 + location"),
        row("body", "()", "200, empty body"),
        row("body", "Result<T, E>", "T or E, both IntoResponse"),
        row("part", "StatusCode (first in a tuple)", "replaces the status"),
        row("part", "[(HeaderName, HeaderValue); N], [(&str, &str); N]", "inserts headers (replaces same-name)"),
        row("part", "HeaderMap", "inserts every header in the map"),
        row("part", "AppendHeaders<I>", "APPENDS headers — repeated names kept (Set-Cookie)"),
        row("part", "Extension<T>", "stores a value in response extensions (for middleware)"),
        row("part", "your own type: impl IntoResponseParts", "whatever you write (Lesson 9)"),
    ])
}

/// GET /items/{id} — a `201 Created` that points at the new resource.
///
/// What it does: returns the custom `Created<T>` response type (Lesson 8).
/// Why it matters: REST says a create answers 201 and a `Location` header
/// naming the new resource. Wrapping that in one type means no handler can
/// forget either half.
async fn create_with_location(Path(id): Path<u32>) -> Created<ApiPayload> {
    Created {
        location: format!("/items/{id}"),
        body: ApiPayload { served_to: format!("item-{id}"), payload_len: 0 },
    }
}

// ============================================================================
// LESSON 8: BUILD YOUR OWN response types (implement IntoResponse)
// ============================================================================
//
// Recipe: struct -> `impl IntoResponse` -> build any tuple/Response inside
// `into_response` and call `.into_response()` on it. The tuple grammar from
// Lesson 7 does the heavy lifting.

/// `201 Created` + `Location` + JSON body, as a type.
///
/// What it does: generic over any serialisable body `T`.
/// Why it matters: the trait bound `T: Serialize` is all `Json<T>` needs, so
/// the wrapper stays generic.
struct Created<T> {
    location: String,
    body: T,
}

impl<T: Serialize> IntoResponse for Created<T> {
    fn into_response(self) -> Response {
        // `HeaderValue::from_str` can fail (control characters), so handle it
        // instead of unwrapping: a bad location is OUR bug -> 500.
        match HeaderValue::from_str(&self.location) {
            Ok(location) => {
                (StatusCode::CREATED, [(header::LOCATION, location)], Json(self.body)).into_response()
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "invalid Location header").into_response(),
        }
    }
}

/// A uniform `{ "data": .., "meta": .. }` envelope for every API answer.
///
/// What it does: wraps any payload, adds the API version header.
/// Why it matters: clients parse one shape; Module 18 builds pagination
/// on top of exactly this idea.
struct ApiEnvelope<T> {
    data: T,
}

#[derive(Serialize)]
struct Meta {
    module: u8,
    api_version: &'static str,
}

#[derive(Serialize)]
struct EnvelopeBody<T> {
    data: T,
    meta: Meta,
}

impl<T: Serialize> IntoResponse for ApiEnvelope<T> {
    fn into_response(self) -> Response {
        let body = EnvelopeBody { data: self.data, meta: Meta { module: 6, api_version: "v1" } };
        ([("x-api-version", "v1")], Json(body)).into_response()
    }
}

/// GET /envelope — the envelope in use.
async fn envelope_demo() -> ApiEnvelope<Vec<&'static str>> {
    ApiEnvelope { data: vec!["extractors", "responses", "state"] }
}

/// A CSV document as a response type (Axum has no built-in CSV).
///
/// What it does: joins rows with commas and sets `text/csv`.
/// Why it matters: any format Axum lacks — XML, YAML, MessagePack — is this
/// same 10-line pattern: serialise, set content-type, done. (Real CSV needs
/// quoting; use the `csv` crate in production.)
struct Csv(Vec<Vec<String>>);

impl IntoResponse for Csv {
    fn into_response(self) -> Response {
        let text: String = self
            .0
            .iter()
            .map(|row| row.join(","))
            .collect::<Vec<_>>()
            .join("\n");
        (
            [
                (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                (header::CONTENT_DISPOSITION, "attachment; filename=\"report.csv\""),
            ],
            text + "\n",
        )
            .into_response()
    }
}

/// GET /report.csv — returns the `Csv` type.
async fn csv_report() -> Csv {
    Csv(vec![
        vec!["module".into(), "topic".into()],
        vec!["5".into(), "extractors".into()],
        vec!["6".into(), "responses".into()],
    ])
}

// ============================================================================
// LESSON 9: BUILD YOUR OWN response parts (implement IntoResponseParts)
// ============================================================================

/// A typed `Cache-Control` header.
///
/// What it does: an enum instead of a magic string; implements
/// `IntoResponseParts`, so it goes anywhere in a tuple BEFORE the body.
/// Why it matters: parts are reusable decorations. `Error = Infallible`
/// because this part cannot fail.
enum CacheControl {
    NoStore,
    Public { max_age: u32 },
    Private { max_age: u32 },
}

impl IntoResponseParts for CacheControl {
    type Error = Infallible;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let text = match self {
            CacheControl::NoStore => "no-store".to_string(),
            CacheControl::Public { max_age } => format!("public, max-age={max_age}"),
            CacheControl::Private { max_age } => format!("private, max-age={max_age}"),
        };
        // Our own strings are always valid, but stay panic-free anyway.
        let value = HeaderValue::from_str(&text).unwrap_or_else(|_| HeaderValue::from_static("no-store"));
        res.headers_mut().insert(header::CACHE_CONTROL, value);
        Ok(res)
    }
}

/// GET /cache/{mode} — pick `no-store`, `public` or `private`.
async fn cache_demo(Path(mode): Path<String>) -> Response {
    let cache = match mode.as_str() {
        "public" => CacheControl::Public { max_age: 3600 },
        "private" => CacheControl::Private { max_age: 60 },
        _ => CacheControl::NoStore,
    };
    (cache, format!("cache mode: {mode}\n")).into_response()
}

/// A part that CAN fail: it sets a `Link` header from user-provided text.
///
/// What it does: validates the value when the response is assembled.
/// Why it matters: `IntoResponseParts::Error` must itself be `IntoResponse`;
/// when the part fails, the error's response is sent INSTEAD of the tuple.
struct LinkHeader(String);

/// The error of `LinkHeader`.
struct InvalidHeader(String);

impl IntoResponse for InvalidHeader {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("invalid header value: {}\n", self.0)).into_response()
    }
}

impl IntoResponseParts for LinkHeader {
    type Error = InvalidHeader;

    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        let value = HeaderValue::from_str(&self.0).map_err(|_| InvalidHeader(format!("{:?}", self.0)))?;
        res.headers_mut().insert(header::LINK, value);
        Ok(res)
    }
}

/// GET /fallible-part/{value} — try `ok`, then `a%0Ab` (an encoded newline).
async fn fallible_part(Path(value): Path<String>) -> (LinkHeader, &'static str) {
    (LinkHeader(value), "the Link header was accepted\n")
}

// ============================================================================
// LESSON 10: A streaming body
// ============================================================================

/// GET /stream — five chunks, one every 300 ms, sent as they are produced.
///
/// What it does: `Body::from_stream` takes any `Stream` of `Result<Bytes, _>`;
/// `stream::unfold` produces one item per step with a sleep in between.
/// Why it matters: nothing is buffered — memory stays flat for a 10 GB file
/// or a live feed. Try `curl -N -s .../stream` and watch lines arrive one by
/// one. Modules 28-30 use this for big downloads and video.
async fn stream_demo() -> Response {
    let chunks = stream::unfold(1u32, |step| async move {
        if step > 5 {
            return None;
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
        let chunk: Result<Bytes, Infallible> = Ok(Bytes::from(format!("chunk {step} of 5\n")));
        Some((chunk, step + 1))
    });
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        Body::from_stream(chunks),
    )
        .into_response()
}
