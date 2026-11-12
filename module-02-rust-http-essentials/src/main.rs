//! Module 02: Rust and HTTP essentials.
//! Topics: async/await and Tokio in plain terms, traits and generics Axum
//! uses, closures, `Arc`/`Mutex`, serde basics, `Result` and `?`; HTTP:
//! URL anatomy, methods, status codes, headers, bodies, JSON, REST.
//! Run: cargo run -p module-02-rust-http-essentials   (port 3002)
//!
//! ## Lesson Reading Guide
//!
//! This module is the bridge from "Rust I know" to "axum server I can
//! write". It assumes no async or HTTP background: every handler below
//! teaches one idea, and every doc comment explains what it does AND why it
//! matters. Read top to bottom; each `LESSON` banner is one step.
//!
//! The working examples, in HTTP terms:
//! - `/async-basics`  — what `async fn` and `.await` really do (Lesson 1)
//! - `/serde-roundtrip` — JSON in a struct and back out (Lesson 2)
//! - `/counter`       — `Arc` + `Mutex` across `tokio::spawn` tasks (Lesson 3)
//! - `/status/*`      — a status-code tour (Lesson 4)
//! - `/only-post` + `/maybe-fail` — methods, 405, and `Result` + `?` (Lessons 5-6)
//! - `/traits-generics` — traits, generics and `impl Trait`, the way Axum uses them (Lesson 7)
//! - `/closures` — closures and `move`, the shape of every `tokio::spawn` (Lesson 8)
//! - `/anatomy` — the request you sent, taken apart: method, URL, headers, body (Lesson 9)
//! - `/rest-map` + `/curl-cheatsheet` — REST and the curl/httpie anatomy (Lesson 10)
//!
//! Nothing here uses path/query/body extractors or shared route state on
//! purpose — those are Modules 05 and 07, and no idea is used before its
//! own lesson in this course. (The one exception is Lesson 9, which takes the
//! raw `Request` as a *preview* of what an extractor is.)

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// The port this module listens on: `AXUM_PORT` if set, else 3000 + module number.
///
/// What it does: reads the environment variable once at startup.
/// Why it matters: every module in the course follows the same rule, so two
/// modules never fight over a port and a learner can always override it.
fn requested_port() -> u16 {
    std::env::var("AXUM_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3002)
}

// ============================================================================
// LESSON 1: Async basics (async fn, .await, and what the runtime does)
// ============================================================================

/// The entry point: same shape every course module uses.
///
/// What it does: registers one route table, binds port 3002, serves forever.
/// Why it matters: `#[tokio::main]` is a macro that rewrites `async fn main`
/// into a normal `fn main` that starts the Tokio runtime and blocks on the
/// future it returns. Without it, `main` cannot `.await` anything — the
/// compiler says so, loudly.
#[tokio::main]
async fn main() {
    let app = axum::Router::new()
        .route("/", get(dashboard))
        .route("/async-basics", get(async_basics))
        .route("/counter", get(counter))
        .route("/serde-roundtrip", get(serde_roundtrip))
        .route("/only-post", post(only_post))
        .route("/maybe-fail", get(maybe_fail))
        .route("/status/created", get(created))
        .route("/status/bad-request", get(bad_request))
        .route("/status/teapot", get(teapot))
        .route("/status/rate-limited", get(rate_limited))
        .route("/status/server-error", get(server_error))
        .route("/status/maintenance", get(maintenance))
        .route("/traits-generics", get(traits_generics))
        .route("/closures", get(closures))
        .route("/anatomy", get(anatomy).post(anatomy))
        .route("/rest-map", get(rest_map))
        .route("/curl-cheatsheet", get(curl_cheatsheet));

    let port = requested_port();
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .unwrap_or_else(|error| panic!("could not bind port {port}: {error}"));

    println!("Module 02 (Rust and HTTP essentials) listening on http://127.0.0.1:{port}");
    println!("Start with:  curl -si http://127.0.0.1:{port}/async-basics");

    axum::serve(listener, app).await.expect("server stopped unexpectedly");
}

/// GET / — the module dashboard.
///
/// What it does: lists every lesson endpoint with one line of theory.
/// Why it matters: an entire Axum handler can be shared, static text —
/// proving that behind every "framework" sits one plain function like this.
/// Note the traits/generics the remaining lessons rely on: a handler is
/// selected by the `Handler` trait; responses come from the `IntoResponse`
/// trait; both are made visible right here in the `.route(...)` calls.
async fn dashboard() -> &'static str {
    "Rust and HTTP essentials (module 02)

GET  /async-basics      async fn, .await, Tokio sleep, elapsed time
GET  /serde-roundtrip   a struct deserialized from JSON and re-serialized
GET  /counter           Arc + Mutex shared across three tokio::spawn tasks
POST /only-post         HTTP methods; try GET on it and get a 405
GET  /maybe-fail        Result and the ? operator
GET  /status/created|bad-request|teapot|rate-limited|server-error|maintenance
GET  /traits-generics   traits, generics, impl Trait (what IntoResponse is made of)
GET  /closures          closures, move, FnMut — the shape of tokio::spawn
ANY  /anatomy           your own request taken apart (method, URL, headers, body)
GET  /rest-map          REST: resources, verbs, status codes
GET  /curl-cheatsheet   curl and httpie, flag by flag

Anatomy tour with curl: curl -si http://127.0.0.1:3002/async-basics
"
}

/// GET /async-basics — makes the function pause without blocking the server.
///
/// What it does: sleeps for 200 milli seconds (Tokio timers), measures real
/// elapsed time, then reports how long the await actually took.
/// Why it matters: `.await` is a *suspension point*, not a thread sleep. While
/// this future waits, the runtime thread runs other handlers — if you had
/// 1,000 concurrent requests here, they would all still finish in ~0.2 s.
/// A blocking call (`thread::sleep`) would freeze the worker thread; Module
/// 39 turns that difference into homework: `spawn_blocking` and the fixes.
async fn async_basics() -> String {
    let started = std::time::Instant::now();

    // Tokio's timer, not the OS one: it schedules on the runtime's internal
    // wheel, so no thread is blocked while this future waits.
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let elapsed = started.elapsed();
    format!(
        "Slept 200 ms with .await — actual elapsed: {elapsed:.1?}.\nThe runtime kept accepting requests while this future waited.\n"
    )
}

// ============================================================================
// LESSON 2: serde round-trip (JSON as a strongly typed contract)
// ============================================================================

/// The struct both sides of this lesson: JSON in, typed Rust value; typed
/// Rust value, JSON out.
///
/// What it does: `Deserialize` gives the struct `serde_json::from_str`, and
/// `Serialize` gives it `serde_json::to_string_pretty`.
/// Why it matters: in the course this same struct appears on both sides of an
/// HTTP boundary — later written by clients as request bodies into extractors
/// (Module 05) and returned as responses (Module 06). One derive, no runtime
/// schema gymnastics: if a client sends `stars: "many"`, the parse fails with
/// a typed error, not a runtime surprise.
#[derive(Serialize, Deserialize)]
struct RepoLookup {
    name: String,
    stars: u32,
    topics: Vec<String>,
}

/// GET /serde-roundtrip — runs the round trip on a fixed payload.
///
/// What it does: parses a hardcoded JSON string (the "request"), mutates the
/// parsed value like a real handler would, and pretty-prints it back out
/// (the "response").
/// Why it matters: it makes ser/deserialization visible *before* HTTP wraps
/// it. Error mapping for malformed bodies is learned properly in Module 08
/// with a real `AppError` — here the parsed data is trusted because it is
/// ours.
async fn serde_roundtrip() -> String {
    // Pretend this arrived from a client.
    let incoming = r#"{"name":"tokio-rs/axum","stars":23000,"topics":["web","http","rust"]}"#;

    // Deserialize: JSON text -> typed struct. from_str returns
    // Result<RepoLookup, serde_json::Error> we would map in Module 08.
    let mut repo: RepoLookup = serde_json::from_str(incoming).expect("hardcoded JSON is valid");

    // A "real" mutation on the typed value: bump the star count.
    repo.stars += 1;

    // Serialize: typed struct -> JSON text again. The round trip is done.
    let outgoing = serde_json::to_string_pretty(&repo).expect("our own struct serializes");

    format!("Parsed:     {incoming}\n\nRe-served: {outgoing}\n")
}

// ============================================================================
// LESSON 3: Arc and Mutex across tokio::spawn (shared mutable data)
// ============================================================================

/// GET /counter — one request, three spawned tasks, one shared counter.
///
/// What it does: creates `Arc<Mutex<u64>>`, clones the pointer into three
/// `tokio::spawn` tasks, lets each increment 1,000 times, joins the tasks,
/// and reports the total.
/// Why it matters: this exact trio is how a server keeps mutable state:
/// - `Arc` = **A**tomic **R**eference **C**ounted pointer: many owners, one
///   allocation, freed when the last owner drops.
/// - `Mutex` = lock that guards mutation: only one task inside at a time.
/// - `tokio::spawn` = run a future on the runtime *concurrently*, not just
///   interleaved like `.await`.
///
/// Counters must end at exactly 3,000 — run it repeatedly to see it never
/// race. Module 07 plugs this pattern into the router properly with
/// `State<Arc<AppState>>`; today it stays inside one handler on purpose.
async fn counter() -> String {
    let count = std::sync::Arc::new(std::sync::Mutex::new(0_u64));
    let mut handles = Vec::new();

    for worker in 0..3_u64 {
        // Arc::clone shares the *pointer*, increments a counter, and never
        // copies the guarded value — the idiom for task-shared data.
        let count = std::sync::Arc::clone(&count);
        handles.push(tokio::spawn(async move {
            for _ in 0..1_000 {
                // lock() blocks for nanoseconds; that is fine in async code,
                // holding a lock ACROSS .await is the classic deadlock
                // (Module 39 teaches the prevention).
                let mut guard = count.lock().expect("mutex was never poisoned");
                *guard += 1;
            }
            worker
        }));
    }

    // Join every handle: await returns what the task returned. If the task
    // had panicked, JoinError would arrive here instead.
    let mut finished = Vec::new();
    for handle in handles {
        finished.push(handle.await.expect("counter task did not panic").to_string());
    }

    let total = *count.lock().expect("mutex was still fine");
    format!(
        "3 workers x 1000 increments in one request = {total} (exactly 3000).\nWorkers finished in order: {finished:?}.\nArc = shared owner, Mutex = safety, spawn = concurrency.\n"
    )
}

// ============================================================================
// LESSON 4: The status-code tour (bare StatusCode responses)
// ============================================================================

// Every handler below is `async fn f() -> StatusCode`.
//
// What it does: returns ONLY a status code — no body bytes at all.
// Why it matters: `StatusCode` implements `IntoResponse` by itself, which
// makes the whole family of tiny codes a two-line handler. Combined with
// `curl -i` (the HTTP anatomy tour in the README) you see precisely what a
// status line is: metadata at the top of a response with an optional body.
// Status + body combos are Module 03's lesson.

/// 201 Created — exhaustion for "the POST did create something somewhere".
async fn created() -> StatusCode {
    StatusCode::CREATED
}

/// 400 Bad Request — the client sent something undecodable.
async fn bad_request() -> StatusCode {
    StatusCode::BAD_REQUEST
}

/// 418 I'm a teapot — the RFC 2324 joke, real servers really use it for fun.
async fn teapot() -> StatusCode {
    StatusCode::IM_A_TEAPOT
}

/// 429 Too Many Requests — rate limiters answer with this (Module 17).
async fn rate_limited() -> StatusCode {
    StatusCode::TOO_MANY_REQUESTS
}

/// 500 Internal Server Error — don't trust the server's promised guarantees.
async fn server_error() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}

/// 503 Service Unavailable — your app is up but overloaded or disabled.
async fn maintenance() -> StatusCode {
    StatusCode::SERVICE_UNAVAILABLE
}

// ============================================================================
// LESSON 5: HTTP methods (POST-only route, 405 from behind)
// ============================================================================

/// POST /only-post — registered with `.route("/only-post", post(...))`.
///
/// What it does: accepts POST bodies; GET/DELETE/... on the same path is
/// answered by Axum itself with 405 Method Not Allowed. Try:
/// `curl -si http://127.0.0.1:3002/only-post` (as GET) and see.
/// Why it matters: the `post(...)` selector is one of `get/post/put/patch/
/// delete/head/options/any` — the route is registered *per method*, and Axum
/// generates the 405 and `Allow` header bookkeeping for the rest. Methods
/// carry the REST verbs: read (GET), create (POST), replace (PUT), update
/// (PATCH), remove (DELETE) — REST thinking is spread across Modules 18 and
/// 27.
async fn only_post() -> String {
    "Thank you for POSTing. Try the same URL with GET to watch Axum answer 405.\n"
        .to_string()
}

// ============================================================================
// LESSON 6: Result and the ? operator (errors as values)
// ============================================================================

/// GET /maybe-fail — a handler whose signature carries failure too.
///
/// What it does: flips a coin from system nanos: even -> `Ok(String)` and a
/// 200; odd -> `Err(StatusCode::SERVICE_UNAVAILABLE)` returned early by `?`.
/// Why it matters: `Result<T, E>` is how Rust moves errors from panics to
/// ordinary returns, and `?` is how one line bubbles an error up instead of
/// writing `match` chains. Axum makes the whole thing HTTP-shaped:
/// `Result<T, E>` is itself `IntoResponse` when both sides are — the Ok arm
/// builds the 200 response (with this body), the Err arm builds the response
/// too (here a bare 503 with no body). Handlers you already trust become
/// two-line functions that never re-check `unwrap`. Real `AppError` types
/// with bodies and logging are Module 08.
async fn maybe_fail() -> Result<String, StatusCode> {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.subsec_nanos())
        .unwrap_or(0);

    // A helper that can fail, so `?` has something real to propagate.
    let verdict = coin_flip(nanos)?;
    Ok(format!("Coin came up Ok: 200 with this body ({verdict}). Run again to try for a 503.\n"))
}

/// A tiny fallible function: even -> Ok, odd -> Err(503).
///
/// What it does: returns `Result<&'static str, StatusCode>`.
/// Why it matters: `?` in `maybe_fail` calls `From::from` on the error, so a
/// `StatusCode` error flows straight out of the handler. This is the exact
/// mechanism Module 08 scales up to a full `AppError` enum.
fn coin_flip(nanos: u32) -> Result<&'static str, StatusCode> {
    if nanos & 1 == 0 {
        Ok("heads")
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

// ============================================================================
// LESSON 7: Traits and generics — the two ideas Axum is built from
// ============================================================================

/// Anything that can describe itself as a line of text.
///
/// What it does: one method; every type that implements it can be used by
/// `describe_all` below without that function knowing the concrete type.
/// Why it matters: Axum's `IntoResponse`, `FromRequest` and `Handler` are
/// ordinary traits exactly like this one. "Handler returns anything that
/// implements IntoResponse" is just "function takes anything that implements
/// Describe".
trait Describe {
    fn describe(&self) -> String;
}

struct Student {
    name: &'static str,
}

struct Course {
    title: &'static str,
    modules: u32,
}

impl Describe for Student {
    fn describe(&self) -> String {
        format!("student {}", self.name)
    }
}

impl Describe for Course {
    fn describe(&self) -> String {
        format!("course '{}' with {} modules", self.title, self.modules)
    }
}

/// A *generic* function: `T` is any type that implements `Describe`.
///
/// What it does: works for `Student`, `Course` or anything added later.
/// Why it matters: the compiler generates one specialised copy per type
/// (monomorphisation) — zero runtime cost. `Router::route` is generic in the
/// same way: `route<H: Handler<T, S>>(path, handler)` accepts any async fn
/// that satisfies the trait.
fn describe_all<T: Describe>(items: &[T]) -> Vec<String> {
    items.iter().map(Describe::describe).collect()
}

/// `impl Trait` in argument position — shorthand for the generic above.
///
/// Why it matters: handlers *return* `impl IntoResponse` the same way (Module
/// 03): the caller only knows "some type that can become a response".
fn shout(value: impl Display) -> String {
    value.to_string().to_uppercase()
}

/// GET /traits-generics — runs the three ideas above and reports.
///
/// What it does: calls a generic function on two types, uses `impl Trait`,
/// and shows a trait object (`Box<dyn Describe>`) for runtime choice.
/// Why it matters: static dispatch (generics) is fast and used for handlers;
/// dynamic dispatch (`dyn`) is flexible and used for dependency injection
/// (Module 07). You now know both words.
async fn traits_generics() -> String {
    let students = [Student { name: "Asha" }, Student { name: "Ravi" }];
    let courses = [Course { title: "Axum", modules: 44 }];

    // Trait object: the concrete type is chosen at runtime, behind a pointer.
    let mixed: Vec<Box<dyn Describe>> = vec![
        Box::new(Student { name: "Meera" }),
        Box::new(Course { title: "Rust", modules: 30 }),
    ];

    format!(
        "generic over Student: {:?}\ngeneric over Course:  {:?}\nimpl Display arg:      {}\ntrait objects:         {:?}\n",
        describe_all(&students),
        describe_all(&courses),
        shout("impl trait"),
        mixed.iter().map(|item| item.describe()).collect::<Vec<_>>(),
    )
}

// ============================================================================
// LESSON 8: Closures — small functions that capture their surroundings
// ============================================================================

/// GET /closures — three closure shapes you will meet in every Axum app.
///
/// What it does: shows `Fn` (read-only capture), `FnMut` (mutating capture)
/// and `move` (taking ownership) closures, then spawns a task with `move`.
/// Why it matters: `tokio::spawn(async move { .. })`, `.map(|x| ..)`,
/// `.layer(from_fn(..))` and `.on_response(|..| ..)` are all closures.
/// Rule of thumb: a spawned task outlives the handler's stack, so it must OWN
/// what it uses — that is what `move` means.
async fn closures() -> String {
    // Fn: borrows `prefix`, only reads it.
    let prefix = String::from("axum");
    let label = |name: &str| format!("{prefix}-{name}");

    // FnMut: mutates captured `calls`, so the binding itself must be `mut`.
    let mut calls = 0;
    let mut counter = || {
        calls += 1;
        calls
    };
    let first = counter();
    let second = counter();

    // move: the closure takes ownership, so it can be sent to another task.
    let owned = String::from("owned by the task");
    let handle = tokio::spawn(async move { owned.len() });
    let owned_len = handle.await.expect("task did not panic");

    format!(
        "Fn:     {}\nFnMut:  counter gave {first} then {second}\nmove:   spawned task measured {owned_len} bytes it owned\n",
        label("closures"),
    )
}

// ============================================================================
// LESSON 9: HTTP anatomy — a request taken apart
// ============================================================================

/// ANY /anatomy — echoes the request line, headers and body you sent.
///
/// What it does: takes the whole `axum::extract::Request` (a preview of
/// extractors, Module 05) and prints method, URL pieces, version, every
/// header and the body byte count.
/// Why it matters: an HTTP request is just text: a request line
/// (`METHOD /path?query HTTP/1.1`), header lines (`name: value`), a blank
/// line, then an optional body. Run `curl -si -X POST -d 'hello' -H 'x-demo: 1'
/// http://127.0.0.1:3002/anatomy` and compare the echo with `curl -v`.
async fn anatomy(request: Request) -> Response {
    let (parts, body) = request.into_parts();

    // Body bytes counted with a hard cap so a huge upload cannot exhaust memory
    // (Module 09 teaches DefaultBodyLimit properly).
    let bytes = match axum::body::to_bytes(body, 64 * 1024).await {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::PAYLOAD_TOO_LARGE, "body larger than 64 KiB\n").into_response(),
    };

    let mut report = String::new();
    report.push_str(&format!("method:   {}\n", parts.method));
    report.push_str(&format!("path:     {}\n", parts.uri.path()));
    report.push_str(&format!("query:    {}\n", parts.uri.query().unwrap_or("(none)")));
    report.push_str(&format!("version:  {:?}\n", parts.version));
    report.push_str("headers:\n");
    for (name, value) in &parts.headers {
        report.push_str(&format!("  {name}: {}\n", value.to_str().unwrap_or("<binary>")));
    }
    report.push_str(&format!("body:     {} bytes\n", bytes.len()));
    if !bytes.is_empty() {
        report.push_str(&format!("          {:?}\n", String::from_utf8_lossy(&bytes)));
    }
    report.into_response()
}

// ============================================================================
// LESSON 10: REST and curl — the vocabulary of every later module
// ============================================================================

/// One row of the REST table.
#[derive(Serialize)]
struct RestRow {
    method: &'static str,
    path: &'static str,
    meaning: &'static str,
    success: u16,
    idempotent: bool,
    safe: bool,
}

/// GET /rest-map — a REST API for "books", described as data.
///
/// What it does: returns the standard resource/verb/status table as JSON.
/// Why it matters: REST = nouns in the URL (`/books/7`), verbs in the method,
/// outcome in the status code. *Safe* methods never change data; *idempotent*
/// methods give the same result when repeated (retries are harmless). Modules
/// 14-18 implement exactly this table.
async fn rest_map() -> axum::Json<Vec<RestRow>> {
    axum::Json(vec![
        RestRow { method: "GET", path: "/books", meaning: "list books", success: 200, idempotent: true, safe: true },
        RestRow { method: "GET", path: "/books/7", meaning: "read one book", success: 200, idempotent: true, safe: true },
        RestRow { method: "POST", path: "/books", meaning: "create a book", success: 201, idempotent: false, safe: false },
        RestRow { method: "PUT", path: "/books/7", meaning: "replace book 7", success: 200, idempotent: true, safe: false },
        RestRow { method: "PATCH", path: "/books/7", meaning: "change some fields", success: 200, idempotent: false, safe: false },
        RestRow { method: "DELETE", path: "/books/7", meaning: "remove book 7", success: 204, idempotent: true, safe: false },
    ])
}

/// GET /curl-cheatsheet — the flags this course uses, in plain text.
///
/// What it does: prints each curl flag with its httpie twin.
/// Why it matters: every README "Try It" block is curl; knowing `-s -i -v -X
/// -H -d` means you can reproduce any lesson from the terminal.
async fn curl_cheatsheet() -> &'static str {
    "curl flag        meaning                               httpie twin
-s               silent (no progress bar)              (default)
-i               include response headers              --print=hb (default)
-v               verbose: request AND response         -v
-X POST          choose the method                     http POST url
-H 'k: v'        add a header                          k:v
-d 'a=1'         send a body (implies POST)            a=1 (form) / a:=1 (json)
--json '{...}'   JSON body + content-type headers      http url key=value
-o file          save the body to a file               --download
-L               follow redirects                      --follow
"
}
