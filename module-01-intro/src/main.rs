//! Module 01: Introduction and setup.
//! Topics: what Axum is, Tokio + Hyper + Tower, `axum::serve`, `#[tokio::main]`,
//! plain text and JSON responses, configuration from the environment, curl,
//! and auto-reload with `bacon`.
//! Run: cargo run -p module-01-intro   (listens on http://127.0.0.1:3001)
//!
//! ## Lesson Reading Guide
//!
//! Like every `src/main.rs` in this course, this file is a linear lesson:
//! read it from top to bottom, and each `LESSON` banner is one step. The doc
//! comment on every function says what it does and why it matters. Keep the
//! server running and try each endpoint with `curl` while you read — the
//! README lists the exact commands.
//!
//! Where Axum sits (one sentence per crate, because the whole stack matters):
//! - **Tokio** owns the async runtime: event loop, threads, timers, sockets.
//! - **Hyper** is the HTTP/1 and HTTP/2 engine: it parses bytes into requests
//!   and writes responses back.
//! - **Tower** contributes "layers": composable wrappers around services
//!   (timeouts, limits, logging). Axum is built on Tower services.
//! - **Axum** sits on top and gives you the pieces you will actually write:
//!   the `Router`, handlers as plain async functions, extractors for reading
//!   requests, and `IntoResponse` for writing them.

use axum::{response::Json, routing::get, Router};
use serde::Serialize;
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

// ============================================================================
// LESSON 1: Hello world (the smallest Axum server that can run)
// ============================================================================

/// The entry point of the server.
///
/// What it does: starts Tokio's async runtime, builds the router, binds a TCP
/// listener, and hands both to `axum::serve` (Axum 0.7+ replaced the old
/// `axum::Server` with this one function).
/// Why it matters: everything in Axum is driven by this single loop — Tokio
/// accepts a connection, Hyper turns the bytes into a request, and Axum finds
/// the matching route and runs your handler.
#[tokio::main]
async fn main() {
    // LESSON 4 covers where this port comes from.
    let port = requested_port();
    let address = format!("127.0.0.1:{port}");

    // `Router` maps "HTTP method + path" -> handler. It starts empty and grows
    // by chaining `.route(...)` calls; each call returns the same router, so
    // the chain reads like a table of contents for the whole module.
    let app = Router::new()
        .route("/", get(root))
        .route("/text", get(text))
        .route("/json", get(json_hello))
        .route("/port", get(port_from_env))
        .route("/now", get(now));

    // A `TcpListener` is plain Tokio, not Axum. That separation (listener vs.
    // framework) is deliberate: later, Modules 41 and 42 swap in TLS listeners
    // and Unix domain sockets without touching any handler.
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .unwrap_or_else(|error| {
            panic!("could not bind {address}: {error} — is another course module already running on this port?")
        });

    // Printed hints double as the "Try It" section of the README.
    println!("Module 01 (Introduction and setup) listening on http://{address}");
    println!("Try it with curl:");
    println!("  curl -s http://{address}/");
    println!("  curl -s http://{address}/json");
    println!("  curl -si http://{address}/text");

    // `axum::serve` keeps accepting connections until the process is killed.
    // (Graceful shutdown, signals, and everything else about serve is Module 41.)
    axum::serve(listener, app)
        .await
        .expect("server stopped unexpectedly");
}

/// GET / — the hello-world handler.
///
/// What it does: returns a `&'static str`. Axum writes it into the response
/// body and labels it `Content-Type: text/plain; charset=utf-8` automatically.
/// Why it matters: an async fn is literally all a handler needs to be — no
/// structs, no macros, no trait implementations. Types such as `&'static str`,
/// `String`, `Json`, `Html`, and `StatusCode` all understand `IntoResponse`,
/// so Axum can turn each of them into an HTTP reply by itself. (The full
/// response lesson is Module 06.)
async fn root() -> &'static str {
    "Hello, Axum! This is your first handler.\nThis module has 4 more endpoints: /text, /json, /port and /now.\n"
}

// ============================================================================
// LESSON 2: Plain text (and what the Content-Type header does)
// ============================================================================

/// GET /text — a plain-text route, deliberately multi-line.
///
/// What it does: same idea as `/`, but this handler exists so you can inspect
/// a full response with `curl -i` and see the headers for yourself.
/// Why it matters: HTTP is just text with rules — a status line, headers, a
/// blank line, then the body. Watching `curl -i` print `HTTP/1.1 200 OK` and
/// `content-type: text/plain; charset=utf-8` demystifies the whole stack at
/// once, before Module 02 dissects the anatomy in detail.
async fn text() -> &'static str {
    "Everything on the web is a request, then a response.\nThis text is a body; curl -i shows it with its headers.\n"
}

// ============================================================================
// LESSON 3: Typed JSON (structs as API contracts)
// ============================================================================

/// GET /json — a typed JSON response.
///
/// What it does: serializes a plain Rust struct with serde and wraps it in
/// `Json`, which sets `Content-Type: application/json` for you.
/// Why it matters: in a real API the struct IS the contract. Change a field
/// type and every client's compiler breaks loudly at build time — JSON in
/// Rust is strongly typed end to end. (Reading JSON bodies from requests is
/// the extractor lesson, Module 05.)
#[derive(Serialize)]
struct HelloJson {
    message: String,
    course: String,
    module: u8,
}

async fn json_hello() -> Json<HelloJson> {
    Json(HelloJson {
        message: "Hello Axum, now with a real JSON body".to_string(),
        course: "Axum Full Course".to_string(),
        module: 1,
    })
}

// ============================================================================
// LESSON 4: Configuration from the environment (the port)
// ============================================================================

/// Reads the port the server should use.
///
/// What it does: checks the `AXUM_PORT` environment variable, falls back to
/// this module's default (3001 = 3000 + module number).
/// Why it matters: servers must never hardcode operational values. Container
/// platforms assign ports at runtime, and every one of the 44 course modules
/// gets its own port so several can run side by side while you study.
fn requested_port() -> u16 {
    env::var("AXUM_PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(3001)
}

/// GET /port — proves configuration reaches the handlers.
///
/// What it does: reports which port this server is configured for, and how
/// you could have chosen a different one.
/// Why it matters: it demystifies `main`: the address you `bind` is decided
/// once, at startup, from configuration. Assignments (Modules 38's config
/// lesson goes deeper: layered config files, dotenvy, clap) all end at the
/// same place — values that are read before `axum::serve` starts.
async fn port_from_env() -> String {
    match env::var("AXUM_PORT") {
        Ok(value) => format!(
            "AXUM_PORT={value}, so this server listens on 127.0.0.1:{value}.\nUnset it and the module default (3001) is used.\n"
        ),
        Err(_) => "AXUM_PORT is not set, so this server used the module default 3001.\nTry: AXUM_PORT=4011 cargo run -p module-01-intro\n".to_string(),
    }
}

// ============================================================================
// LESSON 5: Auto-reload workflow (edit, save, restart — with bacon)
// ============================================================================

/// GET /now — a probe for testing the auto-reload workflow by hand.
///
/// What it does: returns the server's current epoch seconds. It is the same
/// static-typed handler style as the other lessons, so it is a safe spot to
/// edit while the feedback loop is running.
/// Why it matters: Rust has no built-in server auto-reload, but the workflow
/// is a single command with `bacon` (or `cargo watch -x run`):
///
/// ```text
/// cargo install --locked bacon
/// bacon -p module-01-intro run
/// ```
///
/// Then change the string in this function, save, and reload your curl
/// request: bacon rebuilds and restarts the server, and the new text plus the
/// new timestamp prove the restart worked. A per-module `bacon.toml` is not
/// required — the default `run` job is exactly `cargo run` in the background.
async fn now() -> String {
    let seconds_since_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock is before 1970")
        .as_secs();

    format!(
        "Server epoch time: {seconds_since_epoch} seconds.\nChange this text in module-01-intro/src/main.rs and save; \
         with `bacon -p module-01-intro run` the server restarts by itself and this line changes.\n"
    )
}
