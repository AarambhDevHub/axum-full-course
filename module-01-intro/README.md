# Module 01 - Introduction and Setup

## What You'll Learn

- What Axum is, and how it relates to Tokio, Hyper and Tower
- Your first server with `axum::serve` and `#[tokio::main]`
- Plain-text and typed JSON responses
- Reading configuration (the port) from the environment
- Auto-reload while editing (with `bacon`)

## Prerequisites

- Rust 1.80+ (install with [rustup](https://rustup.rs))
- `curl` available in your terminal

No HTTP or async background is assumed — Module 02 fills that in.

## How to Run

```bash
cargo run -p module-01-intro
```

This module listens on **http://127.0.0.1:3001** (set `AXUM_PORT` to change it).

Auto-reload while you edit:

```bash
cargo install --locked bacon
bacon run -- -p module-01-intro
```

Everything after `--` is passed straight to `cargo run`, so `-p module-01-intro`
picks the module. Prefer a shorter command? Go into the module folder and run
plain `bacon run`:

```bash
cd module-01-intro
bacon run
```

## Endpoints

| Method | Path | Response | Purpose |
|--------|------|----------|---------|
| GET | `/` | 200 · text/plain | Hello world |
| GET | `/text` | 200 · text/plain | A multi-line plain text demo |
| GET | `/json` | 200 · application/json | Typed JSON from a `#[derive(Serialize)]` struct |
| GET | `/port` | 200 · text/plain | Shows the `AXUM_PORT` configuration |
| GET | `/now` | 200 · text/plain | Server epoch time (probe for the auto-reload workflow) |

## Key Concepts

The whole course fits in these five lines:

```rust
#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

- `#[tokio::main]` — starts the async runtime that drives everything.
- `Router::new().route(...)` — maps `method + path` to a handler.
- `TcpListener::bind` — where the port comes from.
- `axum::serve` — the request loop accepting connections forever.

## Working Examples

- HelloWorld — `/`
- PlainText — `/text`
- JsonHello — `/json`
- PortFromEnv — `/port`
- ReloadCheck — `/now`

## Try It

```bash
curl -s http://127.0.0.1:3001/
curl -si http://127.0.0.1:3001/text
curl -s http://127.0.0.1:3001/json
curl -s http://127.0.0.1:3001/port
AXUM_PORT=4011 cargo run -p module-01-intro &   # then:
curl -s http://127.0.0.1:4011/port
curl -s http://127.0.0.1:3001/now
```

With `bacon run -- -p module-01-intro`, edit the text inside `/now` and save —
the server rebuilds, restarts, and the response changes with no manual step.

## Common Mistakes

- Following pre-0.7 tutorials and calling `axum::Server::bind` — that type was
  removed. Use `axum::serve(listener, app)` (this exact shape).
- Forgetting `#[tokio::main]` — then `main` cannot await the listener, or the
  binary binds and exits instantly.
- `:3001` already in use — every course module has its own port (3000 + module
  number); another module is probably still running.
- Using path syntax `/:id` from old examples: in Axum 0.8 it **panics at
  startup**. The course uses `/{id}` everywhere (Module 04).
- Setting `default-features = false` on axum without a list — `tokio`, `json`
  and friends are default features and the course assumes they are on.
- Writing `bacon -p module-01-intro run` — bacon reads `-p` as its own option,
  so cargo never receives it. Put cargo flags after `--`:
  `bacon run -- -p module-01-intro`.

## Exercises

1. Add `GET /bye` returning a short `&'static str` goodbye message.
2. Add `GET /json-me` returning a JSON struct with three fields about you.
3. Run the module with `AXUM_PORT=4011` and call `/port`.
4. Install `bacon`, edit `/now`'s text, and watch the auto-restart update it.

Previous: Start

Next: Module 02 - Rust and HTTP Essentials (coming soon — published with its video)
