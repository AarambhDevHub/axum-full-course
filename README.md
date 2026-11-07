# Axum 0.8 Full Course — Build Production APIs in Rust

[![Rust](https://img.shields.io/badge/Rust-1.80+-orange.svg)](https://www.rust-lang.org)
[![Axum](https://img.shields.io/badge/Axum-0.8.9-blue.svg)](https://docs.rs/axum)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![YouTube](https://img.shields.io/badge/YouTube-Aarambh_Dev_Hub-red.svg)](https://youtube.com/@AarambhDevHub)

A beginner-to-advanced **Axum 0.8** course: 44 modules and 6 capstone projects. Every module is one
small crate you can run, break and fix, with a README (endpoints, `curl` commands, common mistakes,
exercises) and a heavily commented `src/main.rs`.

> **Status — Module 01 is available now.**
> The course is released one video at a time. A module is added to this repository when its YouTube
> video goes live, so this repository contains exactly the modules you can watch today.
> Modules 02–44 and the six capstone projects are **coming soon**.

## Available Now

| # | Module | Level | You learn | Port | Run |
|---|--------|-------|-----------|------|-----|
| 01 | [Introduction and Setup](./module-01-intro) | B | `axum::serve`, `#[tokio::main]`, first routes, port from env, auto-reload | 3001 | `cargo run -p module-01-intro` |

```bash
# From the repository root:
cargo run -p module-01-intro

# In another terminal:
curl http://127.0.0.1:3001/
curl -s http://127.0.0.1:3001/json
```

Every module honours `AXUM_PORT`:

```bash
AXUM_PORT=4000 cargo run -p module-01-intro
```

## What makes this course different

Every topic is taught in the same two steps:

1. **Explain every built-in** — each extractor, response type, state tool and error type is
   defined, demonstrated with a live endpoint and shown with its failure modes.
2. **Then build your own** — each module ends with the custom version: your own extractor, response
   type, response part, state-aware extractor, error type and validating extractor.

The table below maps the full course. Only the modules listed under **Available Now** exist in this
repository today; the rest are added as their videos are published.

| You want to build… | Trait / tool | Module | Example in the code |
|---|---|---|---|
| a custom **extractor** (no body) | `FromRequestParts` (+ `OptionalFromRequestParts` for `Option<T>`) | 05 | `ApiKey`, `ClientInfo` |
| a custom **body extractor** | `FromRequest` (delegate reading to `Bytes`/`Json`) | 05, 09 | `PlainLines`, `ValidatedJson<T>`, `JsonOrForm<T>` |
| a custom **response type** | `IntoResponse` | 06 | `Created<T>`, `ApiEnvelope<T>`, `Csv` |
| a custom **response decoration** | `IntoResponseParts` | 06 | `CacheControl`, fallible `LinkHeader`, `TraceId` |
| custom **state / DI** | `FromRef`, `Arc<dyn Trait>`, `Router<S>` | 07 | `Clock`, `Greeter` registry, `CurrentUser` |
| a custom **error** | `IntoResponse` + `From<..>` + `WithRejection` | 08 | `AppError` (enum), `ApiError` (struct), `ApiJson<T>` |
| a custom **rejection** | any `IntoResponse` type as `type Rejection` | 05, 08, 09 | `ApiKeyRejection`, `PlainLinesRejection` |
| less boilerplate | `#[derive(FromRequest / FromRequestParts / FromRef)]` | 09 | `AppJson<T>`, `ListRequest`, `AppState` |
| a custom **Tower layer** (reusable library) | `Layer` + `Service` (clone-and-swap) | 10 | `LogLayer` / `LogService<S>`, order-lab `TagLayer` |
| a custom **middleware function** | `middleware::from_fn(_with_state)`, `map_request`, `map_response` | 11 | `require_bearer`, `rate_limit`, `json_errors` |
| a middleware that is **only sometimes there** | `option_layer` | 11 | `maintenance_mode` |
| a custom **request-id scheme** | `MakeRequestId` | 12 | `CounterRequestId` |
| a custom **header validator** | `ValidateRequest` | 12 | `ApiKeyValidator` |
| a custom **CSRF rejection** | `CsrfLayer::with_rejection_response` | 12 | JSON 403 |
| a custom **HTML response** | `IntoResponse` over a template engine | 13 | `HtmlTemplate<T>`, `Asset` (ETag + 304) |
| a custom **repository** (swap the database) | `trait TodoRepository` + `Arc<dyn ..>` + a contract test | 15 | `MemoryRepository`, `PostgresRepository` |
| a custom **error layer per library** | one `From<LibraryError> for ApiError` each | 14, 16 | sqlx, SeaORM, Diesel, tokio-postgres, MongoDB |
| a custom **cache / rate limiter** | cache-aside, `GovernorConfig`, `GlobalConcurrencyLimitLayer` | 17 | `Cache` trait, per-key limiter |
| a custom **idempotency extractor + guard** | `FromRequestParts` + RAII `Drop` guard | 18 | `IdempotencyKey`, `PendingGuard` |
| a custom **PATCH / cursor / sort whitelist** | JSON Merge Patch on `serde_json::Value`, opaque cursor | 18 | `merge_patch`, `encode_cursor`, `parse_sort` |
| one **error format** for every failure | `rejection(ApiError)` derives + `fallback` | 18 | `ApiJson`, `ApiQuery`, `ApiPath`, problem+json |
| a custom **WebSocket hub** (rooms, presence, limits) | `WebSocketUpgrade` + `split` + `broadcast` + `Drop` guard | 24 | `Hub`, origin check, rate limit, idle timeout |
| a custom **SSE event log** with replay | `Sse` + `Last-Event-ID` + `watch` | 25 | `EventLog`, `OpenStream` guard |
| a custom **streaming body / JSON Lines endpoint** | `Body::from_stream`, `JsonLines`, `Protobuf` | 26 | streaming proxy, atomic `TempFile` |
| a custom **upload pipeline** (storage trait, sniffing) | `Multipart` + `trait Storage` + contract test | 27 | `sanitize_filename`, magic-number sniffing |
| a custom **resumable upload protocol** | upload sessions, `Content-Range`, sha256 | 28 | `ChunkTicket`, sweeper |
| a custom **Range handler and parallel downloader** | `Range`/`If-Range`/`ETag` by hand | 29 | `parse_range`, `parallel_download` |
| a custom **signed-URL scheme** and **HLS playlists** | HMAC-SHA256 + `ServeDir` + ffmpeg | 30 | `Signer`, `master_playlist` |
| a custom **job runner, worker pool and outbox** | `TaskTracker`, `mpsc` + `Semaphore`, apalis, lapin | 31 | `Jobs`, `Mailer`, `Outbox` |
| a custom **retrying client, webhook verifier and reverse proxy** | `reqwest`, HMAC, hop-by-hop rules | 32 | `send_with_retry`, `webhook::verify`, `proxy::handler` |
| a custom **OpenAPI document** (schemas, security, contract test) | `utoipa` + `OpenApiRouter` + `Modify` | 33 | `ApiDoc`, `SecurityAddon`, `committed_spec_matches_the_code` |
| a custom **GraphQL schema** (loaders, guards, limits) | `async-graphql` `Object`/`Guard`/`DataLoader` | 34 | `RoleGuard`, `AuthorLoader`, depth and complexity limits |
| a custom **gRPC service** (interceptor, deadlines) alongside REST | `tonic` `Interceptor` + `route_service` | 35 | `authenticate`, `enforce_deadline`, shared `Store` |
| a **test harness** around `oneshot`, fakes, a throwaway-database helper | `tower::ServiceExt`, `sqlx::test` | 36 | `TestApp`, `FixedClock`, `BrokenRepo`, `TestDb` |
| **request ids, request spans, a metrics middleware, health probes** | `tower-http` `TraceLayer`, `metrics`, OpenTelemetry | 37 | `CounterRequestIds`, per-app Prometheus recorder, `/readyz` |
| **layered config, a secret wrapper, `_FILE` secrets** | `config`, `clap`, `secrecy`, `dotenvy` | 38 | `parse_bool`, `Redacted<T>`, `resolve_file_secrets`, `secrets_equal` |
| **observable cancellation, a semaphore guard, a hedged request** | `tokio::select!`, `ConcurrencyLimitLayer` | 39 | `CompletionGuard`, `/guarded/work`, `/race` |
| **a load tester, a bounded pool, a connection counter** | `oha`/`wrk`, `sqlx` pool, `axum::serve::Listener` | 40 | `loadtest::run`, `SimPool`, `CountingListener` |
| a **drain-with-deadline shutdown, limiting listener, Unix-socket server, Hyper accept loop, socket handoff** | `axum::serve`, `Listener`, `hyper-util` | 41 | `serve_with_drain`, `LimitedListener`, `hyper_direct::serve` |
| a **TLS server, cert reload, HTTP→HTTPS redirect, HSTS** | `axum-server`, `rustls` | 42 | `tls::server_config`, `serve_https`, `redirect` |
| a **container-ready service** (self health check, SIGTERM, Dockerfiles, compose) | Docker, `cargo-chef`, distroless | 43 | `Settings::from_lookup`, `healthcheck`, `Dockerfile` |
| a **hand-written Service, three Layers, a mini Handler trait and a mini Router** | `tower::Service`, `matchit` | 44 | `Counted`, `RetryOnce`, `MiniHandler`, `MiniRouter` |

## Course Roadmap

The full curriculum is 44 modules in 8 parts. The detailed lesson plan lives in
[`axum-course-curriculum.md`](./axum-course-curriculum.md). Each row below becomes a runnable crate
in this repository when its video is published; until then it is marked **Coming soon**.

### Part 1 — Foundations

| # | Module | Level | You learn | Port | Status |
|---|--------|-------|-----------|------|--------|
| 01 | [Introduction and Setup](./module-01-intro) | B | `axum::serve`, `#[tokio::main]`, first routes, port from env, auto-reload | 3001 | **Available now** |
| 02 | Rust and HTTP Essentials | B | async/Tokio, traits and generics, closures, `Arc`/`Mutex`, serde, `Result`/`?`, HTTP anatomy, REST, curl | 3002 | Coming soon |
| 03 | Handlers and First Responses | B | handlers, every built-in return type, `impl IntoResponse`, tuples, `NoContent`, redirects, health check | 3003 | Coming soon |
| 04 | Routing | B | `route`, `{id}`, `{*rest}`, `nest`, `merge`, fallbacks, 404/405, `any`/`on`, HEAD, trailing slashes, services | 3004 | Coming soon |
| 05 | Extractors | B | every built-in extractor, `Option`/`Result`, order, **custom extractors and rejections** | 3005 | Coming soon |
| 06 | Responses in Depth | B | the response grammar, `IntoResponse`, `IntoResponseParts`, streaming, **custom response types and parts** | 3006 | Coming soon |
| 07 | Shared State | B | `State`, locks vs atomics, `FromRef`, DI with trait objects, **state-aware extractor**, testing with fakes | 3007 | Coming soon |
| 08 | Error Handling | B | `AppError`, `thiserror`, `anyhow`, `WithRejection`, problem JSON, panics, logging, **custom error types** | 3008 | Coming soon |
| 09 | Validation and Custom Extractors | A | `validator`, `garde`, `ValidatedJson`, derive macros, body limits, `JsonOrForm`, `Cached`, reading a body twice | 3009 | Coming soon |

### Part 2 — Middleware and Tower

| # | Module | Level | You learn | Port | Status |
|---|--------|-------|-----------|------|--------|
| 10 | Tower Fundamentals | A | `Service` and `Layer` by hand, `ServiceBuilder`, the layer-order rule, `layer` vs `route_layer`, timeout / limit / load-shed, `oneshot` | 3010 | Coming soon |
| 11 | Axum Middleware | A | `from_fn(_with_state)`, `from_extractor`, `map_request`/`map_response`, `option_layer`, reading bodies, **custom rate limiter** | 3011 | Coming soon |
| 12 | The tower-http Toolbox | A | Trace, CORS, compression, request ids, timeout, body limit, CSRF, header validation, CatchPanic, NormalizePath | 3012 | Coming soon |
| 13 | Static Files and Server-Rendered HTML | A | `ServeDir`/`ServeFile`, SPA fallback, rust-embed + ETag, Askama, Minijinja, Maud, post-redirect-get, HTMX | 3013 | Coming soon |

### Part 3 — Data

| # | Module | Level | You learn | Port | Status |
|---|--------|-------|-----------|------|--------|
| 14 | SQLx and Postgres | A | Pool, embedded migrations, `query_as`, `QueryBuilder`, ENUMs, transactions (`FOR UPDATE`), compile-time checked `query!`, error mapping | 3014 | Coming soon |
| 15 | Architecture and the Repository Pattern | A | Layers, repository trait, `dyn` vs generics, contract test, error layering, `AppState` builder, `lib.rs` vs `main.rs` | 3015 | Coming soon |
| 16 | ORMs and Other Databases | A | The same API on SQLx-SQLite, SeaORM, Diesel-async, tokio-postgres and MongoDB; how to choose | 3016 | Coming soon |
| 17 | Caching and Rate Limiting | A | Redis and moka caches, HTTP caching/ETag, `tower_governor`, concurrency limits, graceful degradation | 3017 | Coming soon |
| 18 | REST API Design | A | Offset and cursor pagination, filter/sort, JSON Merge Patch, idempotency keys, versioning, problem+json, `TypedPath` and `Resource` | 3018 | Coming soon |

### Part 4 — Authentication and Security

| # | Module | Level | You learn | Port | Status |
|---|--------|-------|-----------|------|--------|
| 19 | Passwords and Users | A | argon2id and tuning, register/login, timing-safe failures, lockout, reset tokens, change password, `Secret` | 3019 | Coming soon |
| 20 | JWT Authentication | A | jsonwebtoken 11, pinned algorithm, `kid` rotation, refresh rotation with reuse detection, blacklist, `AuthUser` and optional auth | 3020 | Coming soon |
| 21 | Cookies and Sessions | A | CookieJar, signed and private cookies, tower-sessions 0.15, session-fixation defence, flash messages, **custom SQLx session store** | 3021 | Coming soon |
| 22 | Authorization | A | 401/403/404, roles and permissions, guard layers, `Require<P>` extractor, API keys, ownership, tenant scoping | 3022 | Coming soon |
| 23 | Security Hardening | A | security headers and CSP nonce, CORS allow-list, CSRF, limits, SQL injection, SSRF, secrets, self-audit, `cargo audit` | 3023 | Coming soon |

### Part 5 — Realtime, Streaming, Large Files, Media and Background Work

| # | Module | Level | You learn | Port | Status |
|---|--------|-------|-----------|------|--------|
| 24 | WebSockets | A | `WebSocketUpgrade`, `Message`, split send/recv tasks, broadcast rooms, presence, subprotocols, origin check, limits, ping and idle timeout | 3024 | Coming soon |
| 25 | Server-Sent Events | A | `Sse`, `Event`, `KeepAlive`, broadcast fan-out, `Last-Event-ID` replay, finite streams, disconnect detection, proxy buffering | 3025 | Coming soon |
| 26 | Streaming Bodies | A | `Body::from_stream`, `into_data_stream` backpressure, `FileStream`, `AsyncReadBody`, `JsonLines`, `Protobuf`, streaming proxy | 3026 | Coming soon |
| 27 | Uploads and Downloads | A | `Multipart`, size limits, sniffing, filename sanitising, `Storage` trait, safe download headers, batch uploads | 3027 | Coming soon |
| 28 | Chunked and Resumable Uploads | X | upload sessions, per-chunk and whole-file checksums, resume, expiry sweeper, comparison with tus and S3 multipart | 3028 | Coming soon |
| 29 | Range Requests and Chunked Downloads | A | `Range` parser, `206`/`416`, `If-Range`, `ETag`, a virtual 1 GB file, throttling, a parallel downloader | 3029 | Coming soon |
| 30 | Streaming Video and Audio | A | what `<video>` sends, `ServeDir`, signed expiring URLs, audio tags with `lofty`, HLS with ffmpeg | 3030 | Coming soon |
| 31 | Background Jobs and Messaging | X | tracked `tokio::spawn`, `mpsc` worker and backpressure, graceful shutdown, cron, `apalis`, AMQP with `lapin`, outbox | 3031 | Coming soon |
| 32 | Calling Other Services and Proxying | A | `reqwest` timeouts, retries and `Retry-After`, signed webhooks, reverse proxy, forwarded headers | 3032 | Coming soon |

### Part 6 — API Styles

| # | Module | Level | You learn | Port | Status |
|---|--------|-------|-----------|------|--------|
| 33 | OpenAPI Documentation | A | `utoipa` 6, `OpenApiRouter`, schemas and examples, security schemes, Swagger UI, Scalar, a committed contract and client generation | 3033 | Coming soon |
| 34 | GraphQL | X | `async-graphql`, queries, mutations, subscriptions, DataLoader (N+1), guards, depth and complexity limits | 3034 | Coming soon |
| 35 | gRPC with Tonic | X | protobuf, the four call types, interceptors, status codes, deadlines, gRPC and REST on one port, a client | 3035 | Coming soon |

### Part 7 — Quality and Operations

| # | Module | Level | You learn | Port | Status |
|---|--------|-------|-----------|------|--------|
| 36 | Testing | A | `oneshot` route tests, JSON and auth tests, real-listener WebSocket tests, per-test Postgres databases, `mockall`, `wiremock`, `rstest`, `insta` snapshots | 3036 | Coming soon |
| 37 | Logging, Tracing and Metrics | A | `tracing` and JSON logs, `TraceLayer`, request ids, OpenTelemetry (OTLP, `traceparent`), Prometheus metrics, liveness vs readiness | 3037 | Coming soon |
| 38 | Configuration and Project Structure | A | env vars, `dotenvy`, layered `config`, `clap`, secrets (`secrecy`, `*_FILE`), startup validation, feature flags, workspace layout | 3038 | Coming soon |
| 39 | Async and Debugging Handlers | X | blocking vs async, `spawn_blocking`, cancellation, `select!`, backpressure, `Send` errors, `#[debug_handler]` | 3039 | Coming soon |
| 40 | Performance | X | release and `perf` profiles, allocators, pool sizing, compression cost, HTTP/2, load testing, `criterion`, profiling | 3040 | Coming soon |

### Part 8 — Operations and Internals

| # | Module | Level | You learn | Port | Status |
|---|--------|-------|-----------|------|--------|
| 41 | `axum::serve` in Depth | X | `Listener`/`tap_io`, TCP options, `ConnectInfo`, graceful shutdown with a drain deadline, Unix sockets, Hyper directly, HTTP/2, socket handoff | 3041 | Coming soon |
| 42 | TLS and HTTPS | A | `axum-server` + rustls, low-level rustls, ALPN, cert reload, HSTS, HTTP→HTTPS redirect, TLS behind a proxy | 3042 (HTTP redirect 3142) | Coming soon |
| 43 | Docker and Containers | A | multi-stage builds, dependency caching, small images, health checks, compose with Postgres/Redis | 3043 | Coming soon |
| 44 | Axum Internals and Custom Services | X | `Router` and `matchit`, the `Handler` trait, `IntoResponse`, bodies, hand-written `Service` and `Layer`, runtime-agnostic Router | 3044 | Coming soon |

## Requirements

- Rust **1.80 or newer** for Module 01 (install with [rustup](https://rustup.rs)). Later parts use
  `sqlx 0.9`, which needs Rust **1.94+**; each module's README states its requirement.
- [curl](https://curl.se) for trying every endpoint (every module README gives the exact commands).
- An editor with `rust-analyzer` for the exercises.

Optional, only for the database and capstone modules (14–17 and later):

- Docker (or a local PostgreSQL 16 and Redis 7): `docker compose up -d postgres redis`; then `cp .env.example .env`.
- Modules 14–17 skip their database tests (and say so) when `DATABASE_URL` / `REDIS_URL` are not set. Module 16's MongoDB lesson needs a MongoDB server and is otherwise reported as "not running".

## Check your work

```bash
cargo check --workspace --all-targets      # everything currently published compiles
cargo clippy --workspace --all-targets     # no warnings
cargo test --workspace                     # Module 01 needs nothing external
```

As more modules are published, some of their tests will use Postgres/Redis or external tools; the
module READMEs explain exactly what is required and what is skipped.

## Capstone Projects (coming soon)

Each capstone will be one crate, published after the modules it depends on.

| Capstone | What you build | Port | After |
|----------|----------------|------|-------|
| 01 Todo REST API | In-memory, then Postgres + SQLx | 3101 | Part 3 |
| 02 Auth Service | Users, JWT + refresh tokens, roles | 3102 | Part 4 |
| 03 Realtime Chat | WebSockets + SSE, Redis pub/sub | 3103 | Part 5 |
| 04 Video & Music Streaming Site | Resumable uploads, Range downloads, `<video>` player | 3104 | Part 5 |
| 05 Blog / Link Shortener | Server-rendered HTML with HTMX, sessions, caching | 3105 | Part 5 |
| 06 Production API | Postgres, auth, uploads, OpenAPI, tracing, metrics, Docker | 3106 | Part 8 |

## Learning Tracks

- **Foundations (01–09):** the whole request/response cycle as plain Rust — no magic anywhere.
- **Applied (10–18):** middleware, Tower, static files, real databases, real API design.
- **Security (19–23):** passwords, JWT, sessions, authorization, and hardening.
- **Realtime and files (24–30):** WebSockets, SSE, streaming, resumable uploads, media.
- **API styles (33–35):** documentation, GraphQL, gRPC.
- **Production (31, 32, 36–44):** jobs, proxying, testing, observability, config, performance, serving, TLS, Docker, internals.

## What Changed in Axum 0.8 (every module respects this)

- Path syntax is `/{id}` and `/{*rest}`; the old `/:id` and `/*rest` **panic at startup**.
- `#[async_trait]` is gone from custom extractors (they are plain `async fn` in the trait impl).
- `Option<T>` extractors need `OptionalFromRequest` / `OptionalFromRequestParts`.
- Handlers and services must be `Sync` (no `Rc`, `RefCell`, or non-Sync state).
- `ws::Message` payloads are `Bytes` / `Utf8Bytes`, not `String`.
- WebSockets over HTTP/2 use `any(...)`, not just `get(...)`.
- `Json` rejects trailing garbage after a JSON body.

## Repository Layout

```text
.
├── Cargo.toml            # workspace; only published modules are active members
├── Cargo.lock            # committed on purpose: everyone builds the verified versions
├── module-01-intro/      # one crate per published module
│   ├── Cargo.toml
│   ├── README.md         # endpoints, key concepts, Try It, mistakes, exercises
│   └── src/main.rs       # LESSON banners + "What it does / Why it matters" comments
├── .env.example
├── docker-compose.yml    # Postgres + Redis for the later parts
├── axum-course-curriculum.md
└── UPDATE-axum-0.8.9.md  # features that exist only on Axum's main branch (not taught)
```

When a new video is published, uncomment that module's line in the root `Cargo.toml` `members` list
and add its crate directory. This keeps the repository exactly in sync with YouTube.

## Docker and Databases (later parts)

```bash
cp .env.example .env
docker compose up -d postgres redis
```

Module 01 needs neither. Database modules include their own `migrations/` folder and create their
tables at startup, so a fresh Postgres is enough once those modules are published.

## Troubleshooting

- **`Address already in use`** — another module (or process) holds the port; use `AXUM_PORT=XXXX`.
- **A panic at startup mentioning an invalid route** — you used the pre-0.8 `/:id` syntax; write `/{id}`.
- **Huge handler error messages** — add `#[axum::debug_handler]` (needs axum's `macros` feature; Module 09 enables it).
- **`ConnectInfo` fails at runtime** — serve with `app.into_make_service_with_connect_info::<SocketAddr>()` (Module 05).
- **Old compiler** — `rustc --version` must be 1.80 or newer for Module 01.

## About this Axum Version

The course targets **axum 0.8.9** with all crate versions pinned and verified
against each other (see `Cargo.toml`). APIs that exist only on Axum's main
branch are collected in [`UPDATE-axum-0.8.9.md`](./UPDATE-axum-0.8.9.md) and
are never taught as working code.

## Contributing

Found a mistake or have an idea? Open an issue or a pull request. Keep the module style:
one crate, `LESSON` banners, a "What it does / Why it matters" comment on every handler, a README
with a **Try It** block that really works.

## Community

- YouTube: https://youtube.com/@AarambhDevHub
- GitHub: https://github.com/aarambh-darshan
- Discord: https://discord.gg/HDth6PfCnp
- Razorpay: https://razorpay.me/@aarambhdevhub
- Buy Me a Coffee: https://buymeacoffee.com/aarambhdevhub

Made with care by Aarambh Dev Hub for the Rust community.
