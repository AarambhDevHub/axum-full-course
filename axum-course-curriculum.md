# Axum Full Course (Beginner to Advanced): Curriculum

Prepared 27 Oct 2026 for Aarambh Dev Hub.
Goal: a new Axum course that takes a total beginner from "hello world" to production, and covers **everything** a backend developer needs around Axum. Same module format as your Leptos course and the Dioxus curriculum.

---

## 0. What I found in your previous Axum course

I cloned `aarambh-darshan/axum-full-course` and read all 12 module READMEs and sources.

**What it is today**
- 12 modules: Intro, Routing, Extractors, Responses, State, Middleware, Errors, Database, Auth, Advanced, Testing, Production.
- Each module is one crate with a single `src/main.rs` (131 to 394 lines) and a README (What You'll Learn, Running, Endpoints table, patterns, Try It with curl, Next).
- Root: workspace `Cargo.toml` with shared deps, `Dockerfile`, `docker-compose.yml`, `.env.example`, `LICENSE`.
- Targets Axum 0.8.8, Rust 1.78+.

**What is already good (keep):** one crate per module, workspace-pinned dependencies, the *Endpoints* table, curl-based *Try It*, a clear "what changed in 0.8" section.

**Gaps and issues to fix in the new course**
1. **Too much per module.** Module 10 teaches WebSockets, SSE and file uploads in 175 lines. Auth (JWT, Argon2, middleware) is 229 lines. Each of these needs its own module.
2. **A wrong claim.** The README lists `ListenerExt::limit_connections` as an Axum 0.8 feature. I checked the published `axum 0.8.9` source: it is **not there**. It exists only on the unreleased main branch (changelog "Unreleased"). Same for `serve::Executor`, `MethodFilter::QUERY` and `connection_lifetime_limits`. The new course must teach only what is in the published crate and keep unreleased items in a clearly labelled box.
3. **Outdated dependencies.** The old workspace pins `axum-extra 0.10`, `tower-http 0.6`, `sqlx 0.8`, `jsonwebtoken 9.3`, `argon2 0.5`. Current: `axum-extra 0.12.6`, `tower-http 0.7.1`, `sqlx 0.9.0`, `jsonwebtoken 11.1.0`, `argon2 0.6.0`. Minimum Rust is **1.80** for axum 0.8.9 (not 1.78).
4. **Missing topics.** No chunked/resumable upload or download, no Range requests, no video or audio streaming, no Tower fundamentals, validation, sessions/cookies, rate limiting, caching, Redis, ORMs, OpenAPI, GraphQL, gRPC, templates/HTMX, background jobs, OpenTelemetry/metrics, configuration, TLS, HTTP/2, deployment/CI, performance, or Axum internals.
5. **No beginner on-ramp.** It assumes Rust async and HTTP knowledge. A short prerequisites module fixes this.
6. **No capstone.** The Leptos course ends with a production app; this one ends with a 143-line demo.

---

## 1. Version baseline (verified 27 Oct 2026)

| Crate | Version | Notes |
|---|---|---|
| **axum** | **0.8.9** (14 Apr 2026) | Latest stable. MSRV 1.80. |
| axum-core | 0.5.6 | Home of `IntoResponse`, `FromRequest`, `FromRef`, `DefaultBodyLimit` (re-exported by axum). |
| axum-extra | 0.12.6 | Cookies, typed headers, typed routing, `Resource`, `JsonLines`, `Protobuf`, `FileStream`, `Attachment`, etc. |
| axum-macros | 0.5.1 | `debug_handler`, `debug_middleware`, `FromRequest`, `FromRequestParts`, `FromRef`, `TypedPath`. |
| tower / tower-http | 0.5.3 / **0.7.1** | tower-http 0.7.0 added a **CSRF protection layer**. |
| hyper / tokio | 1.12.0 / 1.54.1 | |
| Database | sqlx **0.9.0**, sea-orm 2.0.4, diesel 2.3.14 (+ diesel-async 0.9.2), redis 1.8.0 | |
| Auth | argon2 0.6.0, jsonwebtoken 11.1.0, tower-sessions 0.15.0, axum-login 0.18.0 | **Version trap:** `axum-login` 0.18 requires `tower-sessions` 0.14, so it cannot be combined with 0.15. `tower-sessions-sqlx-store` 0.15.0 needs `tower-sessions-core` 0.14 and sqlx 0.8, and `tower-sessions-redis-store` 0.16.0 needs core 0.14. The course therefore uses tower-sessions 0.15 with a hand-written store. |
| API docs | utoipa 6.0.0, utoipa-axum 0.3.0, utoipa-swagger-ui 10.0.2, utoipa-scalar 0.4.0, aide 0.15.1 | |
| Other APIs | async-graphql-axum 7.2.1, tonic 0.14.7 | |
| Files and media | headers 0.4.2, tokio-util 0.7.20, mime_guess 2.0.5, lofty 0.25.5, rusty-s3 0.10.2 / aws-sdk-s3 1.155.0 (presigned multipart), ffmpeg-sidecar 2.7.0 and m3u8-rs 6.0.1 (optional, HLS) | HLS needs the `ffmpeg` binary installed. |
| Observability | tracing 0.1.44, tracing-subscriber 0.3.23, opentelemetry 0.33.1, tracing-opentelemetry 0.34.0, metrics-exporter-prometheus 0.18.3 | |
| Templates | askama 0.16.1, minijinja 3.0.4, maud 0.27.0 | |
| Testing | testcontainers 0.28.0, mockall 0.15.0, wiremock 0.6.5, insta 1.49.0, rstest 0.28.0 | |

**Unreleased on the main branch (NOT in 0.8.9; do not teach as working code).** Taken from the changelog at commit 86aa615 (19 Oct 2026): `ListenerExt::limit_connections`, `serve::Executor` / `Serve::with_executor`, `serve::ConnectionLifetimeLimits`, `MethodFilter::QUERY` and `routing::query`, `sse::Event::raw`, `Redirect` as a response part, nested-router fallback merging, hyper's default `header_read_timeout` in `axum::serve`, removal of axum-extra's deprecated `Host`/`Scheme`, and `tower-log` no longer a default feature. Several are marked **breaking**, so they will ship in the next breaking release (no date announced; I found no official roadmap). Put them in one `UPDATE-axum-0.8.9.md` note, not in modules.

**Changes in 0.8 that every module must respect:** path syntax is `/{id}` and `/{*rest}` (old `/:id` panics), `#[async_trait]` is gone from custom extractors, `Option<T>` extractors need `OptionalFromRequest(Parts)`, handlers and services must be `Sync`, `ws::Message` uses `Bytes`/`Utf8Bytes`, WebSockets over HTTP/2 use `any(...)`, and `Json` rejects trailing characters.

---

## 2. Course design rules

1. **Beginner first.** Every module opens with the smallest working example, then builds five demos. No concept is used before it is taught.
2. **One idea per module.** No module packs three topics (the old module 10 problem).
3. **Every module is a runnable crate** with `cargo run -p module-NN-name`, an *Endpoints* table, and curl commands.
4. **Published APIs only.** Each code snippet is compiled against the pinned versions before it goes in a README.
5. **Level tag:** **B** basics, **A** applied, **X** advanced.
6. **Template per module README:** What You'll Learn, Prerequisites, How to Run (port), Endpoints table, Key Concepts (a snippet specific to that module), Working Examples (5), Try It (curl), Common Mistakes, Exercises, Previous/Next.

---

## 3. Modules (44) in 8 parts

### Part 1: Foundations (beginner)
| # | Module | Level | What you'll learn | 5 working examples |
|---|---|---|---|---|
| 01 | Introduction and setup | B | What Axum is, how it relates to Tokio, Hyper and Tower, install Rust, `cargo new`, first server with `axum::serve`, `#[tokio::main]`, `curl`, auto-reload (`cargo watch`/`bacon`), project layout | Hello world, Plain-text route, JSON hello, Port from env, Auto-reload loop |
| 02 | Rust and HTTP essentials | B | Async/await and Tokio in plain terms, traits and generics used by Axum, closures, `Arc`/`Mutex`, serde basics, `Result` and `?`; HTTP: URL anatomy, methods, status codes, headers, bodies, JSON, REST, using curl/httpie | Async basics, Serde round-trip, Arc/Mutex counter, HTTP anatomy with curl, Status-code tour |
| 03 | Handlers and first responses | B | Handlers are async functions, return `&'static str`/`String`/`Html`/`Json`/`StatusCode`/tuples, `impl IntoResponse`, `NoContent`, `Redirect` | Text and HTML pages, JSON endpoint, Status codes, Redirect, Health check |
| 04 | Routing | B | `Router::new().route`, `get/post/put/patch/delete/head/options/any`, chaining methods, path params `{id}` and wildcards `{*rest}`, `nest`, `merge`, `fallback`, `method_not_allowed_fallback`, `reset_fallback`, trailing slashes, `route_service`/`nest_service`, HEAD handling | CRUD route table, Nested API, Wildcard files route, Custom 404 and 405, API versioning `/v1` |
| 05 | Extractors | B | `Path`, `Query`, `Json`, `Form`, `HeaderMap`, `String`/`Bytes`, `Request`, `Option<T>` and `Result<T, Rejection>`, extractor order (body last), `OptionalFromRequest`, `MatchedPath`, `OriginalUri`, `NestedPath`, `RawPathParams`, `RawQuery`, `ConnectInfo`, `Extension` | Path and query demo, JSON body, Form post, Header reader, Client-IP echo |
| 06 | Responses in depth | B | `IntoResponse`, `IntoResponseParts`, building `Response`, `AppendHeaders`, status+headers+body tuples, `Html`, `Json`, custom response types, content type negotiation | Custom response type, Header tuples, Cookie header, Content negotiation, File download header |
| 07 | Shared state | B | `State`, `with_state`, `Arc<Mutex>` vs `Arc<RwLock>` and tokio vs std locks, `FromRef` and sub-states, `Extension` vs `State`, trait objects for dependency injection | Counter, In-memory todo store, Config state, Sub-state with `FromRef`, Injected service trait |
| 08 | Error handling | B | Handler errors, `Result<T, AppError>` with `IntoResponse`, `thiserror`, `anyhow`, `?` in handlers, `HandleErrorLayer`, customizing extractor and path rejections, `WithRejection`, global 404, panic catching, JSON problem responses | Error enum, anyhow error response, Custom rejection, Global 404, Problem-details JSON |
| 09 | Validation and custom extractors | A | `validator` 0.21 and `garde`, a `ValidatedJson` extractor, `#[derive(FromRequest)]`, `#[derive(FromRequestParts)]`, `#[derive(FromRef)]`, body size with `DefaultBodyLimit`, parsing by content type, reading the body twice | Validated signup, Custom API-key extractor, Derived extractor, Content-type switch, Body-limit demo |

### Part 2: Middleware and Tower
| # | Module | Level | What you'll learn | 5 working examples |
|---|---|---|---|---|
| 10 | Tower fundamentals | A | `Service` and `Layer`, `ServiceBuilder`, why `Router` is a `Service`, `layer` vs `route_layer`, layer order (last added runs first), `ServiceExt::oneshot` | Logging layer, Layer order lab, ServiceBuilder stack, Route vs router layer, Router as service |
| 11 | Axum middleware | A | `from_fn`, `from_fn_with_state`, `from_extractor`, `map_request`, `map_response`, `option_layer`, `debug_middleware`, modifying requests and responses, reading bodies in middleware | Timing header, Auth guard, Request logger, Response rewriter, Optional layer |
| 12 | The tower-http toolbox | A | `TraceLayer`, `CorsLayer`, `CompressionLayer` and `DecompressionLayer`, `TimeoutLayer`, `RequestBodyLimitLayer`, set/propagate headers, request IDs, sensitive headers, `CatchPanicLayer`, `NormalizePath`, `ValidateRequest`, CSRF layer (new in 0.7) | CORS setup, Compression, Request IDs, Timeouts and limits, CSRF protection |
| 13 | Static files and server-rendered HTML | A | `ServeDir`/`ServeFile`, SPA fallback, `rust-embed`, Askama 0.16, Minijinja 3, Maud, forms and the post-redirect-get pattern, HTMX basics | Static site, SPA fallback, Askama page, Minijinja page, HTMX todo |

### Part 3: Data
| # | Module | Level | What you'll learn | 5 working examples |
|---|---|---|---|---|
| 14 | SQLx and Postgres | A | Pool in state, migrations, `query`/`query_as`/`FromRow`, transactions, compile-time checked queries with offline mode, error mapping | Connect and ping, Notes CRUD, Transaction transfer, Migrations, Checked query |
| 15 | Architecture and the repository pattern | A | Handlers, services and repositories, traits and generics vs `dyn`, `AppState` design, `lib.rs` vs `main.rs`, module layout | Layered CRUD, Repo trait, In-memory vs Postgres repo, AppState builder, Error layering |
| 16 | ORMs and other databases | A | SQLite with SQLx, SeaORM 2, Diesel and diesel-async, tokio-postgres with pools, MongoDB, choosing between them | SQLite notes, SeaORM CRUD, Diesel CRUD, Pooled tokio-postgres, Mongo CRUD |
| 17 | Caching and rate limiting | A | Redis (bb8/deadpool), in-process cache with `moka`, HTTP caching (ETag, Cache-Control, conditional requests), `tower-governor` rate limiting, Tower `ConcurrencyLimit`/`LoadShed` | Redis cache, Moka cache, ETag endpoint, Rate-limited API, Load shedding |
| 18 | REST API design | A | Resource design, pagination (offset and cursor), filtering and sorting, PATCH semantics, idempotency keys, versioning, consistent error format, `axum-extra` `Resource` and typed routing | Paginated list, Filter and sort, Cursor pagination, Idempotent POST, Typed routes |

### Part 4: Authentication and security
| # | Module | Level | What you'll learn | 5 working examples |
|---|---|---|---|---|
| 19 | Passwords and users | A | Why hashing, argon2 0.6 parameters, registration and login, user table, reset-token flow | Register, Login, Hash tuning, Reset token, Change password |
| 20 | JWT authentication | A | `jsonwebtoken` 11, claims, access and refresh tokens, an `AuthUser` extractor, optional auth with `OptionalFromRequestParts`, logout and revocation | Issue token, Protected route, Refresh flow, Optional auth, Token blacklist |
| 21 | Cookies and sessions | A | `CookieJar`, `SignedCookieJar`, `PrivateCookieJar`, `tower-sessions` 0.15 with its in-memory store and a custom `SessionStore` backed by SQLx 0.9 (the published SQLx and Redis stores still depend on `tower-sessions-core` 0.14, see section 1), flash messages, `axum-login` as a note only | Cookie counter, Signed cookie, Session login, Flash message, Custom SQLx session store |
| 22 | Authorization | A | Roles and permissions, guard layers, API keys, resource ownership checks, multi-tenant basics | Role guard, Permission extractor, API-key service, Owner-only edit, Tenant scoping |
| 23 | Security hardening | A | CORS in depth, CSRF, security headers, request limits and timeouts, SQL injection, secrets handling, SSRF when proxying, dependency auditing | Header hardening, CORS policy, Limits demo, Safe queries, Audit report |

### Part 5: Realtime, streaming, large files, media and background work
| # | Module | Level | What you'll learn | 5 working examples |
|---|---|---|---|---|
| 24 | WebSockets | A | `WebSocketUpgrade`, `Message` (`Utf8Bytes`/`Bytes`), split sink and stream, broadcast chat, ping/pong, subprotocols, WebSockets over HTTP/2 with `any`, auth and limits | Echo, Chat room, Broadcast, Presence list, Auth'd socket |
| 25 | Server-Sent Events | A | `Sse`, `Event`, `KeepAlive`, streams with `tokio-stream`, broadcast to SSE, reconnect and `Last-Event-ID` | Live clock, Ticker, Progress stream, Notifications, Broadcast feed |
| 26 | Streaming bodies | A | `Body::from_stream`, `BodyDataStream`, streaming large responses, `FileStream`, `AsyncReadBody`, `JsonLines`, `Protobuf` | Stream a file, JSON lines export, Streamed upload to disk, Proxy stream, Protobuf endpoint |
| 27 | Uploads and downloads | A | `Multipart`, size limits, streaming to disk, validation and MIME checks, `Attachment` and safe filenames, object storage | Image upload, Multi-file upload, Streamed big file, Download endpoint, Storage backend |
| 28 | Chunked and resumable uploads | X | Why one big request fails (memory, timeouts, `DefaultBodyLimit`, proxy limits), slicing a file in the browser with `Blob.slice`, upload sessions (id, chunk size, offsets), sending chunks with `Content-Range`-style headers, writing chunks to disk with `tokio::fs` and `AsyncSeekExt`, per-chunk SHA-256 checks, resuming after a dropped connection by asking the server for the offset, parallel chunks and final assembly, cleaning up abandoned sessions, overview of the tus protocol and S3-style multipart uploads | Start an upload session, Upload and verify one chunk, Resume after a dropped connection, Assemble and checksum the file, Browser progress bar with parallel chunks |
| 29 | Range requests and chunked downloads | A | `Range`, `Content-Range`, `Accept-Ranges` and `If-Range`, when to answer `200`, `206` or `416`, streaming big files without loading them into memory (`tokio::fs::File` with `ReaderStream`, `FileStream`), `Attachment` downloads, ETag checks so a resume is safe, resuming with `curl -C -`, parallel ranged downloads with `reqwest`, backpressure | Stream a 1 GB file, Custom Range handler (`headers::Range`), Resume with `curl -C -`, Parallel chunk downloader, Throttled download |
| 30 | Streaming video and audio | A | Why `<video>` and `<audio>` do not download the whole file (the browser sends `Range: bytes=...` and the server answers `206`), seeking, MIME types with `mime_guess`, the simplest path with `ServeDir`/`ServeFile`, a custom Range handler with `headers::Range` and `FileStream::try_range_response`, open-ended and clamped ranges, `ETag`/`Last-Modified`/`If-Range`, why not to compress media, protecting media with cookies or signed expiring URLs, reading audio tags with `lofty`, overview of adaptive streaming with HLS (ffmpeg segments, `.m3u8` playlist, `hls.js` in the frontend) | Video page with `<video>`, Audio player with seeking, Custom 206/416 handler, Signed expiring stream URL, HLS playlist and segments |
| 31 | Background jobs and messaging | X | `tokio::spawn`, mpsc channels, graceful task shutdown, scheduled tasks (`tokio-cron-scheduler`), job queues (`apalis` 1.0), AMQP with `lapin`, outbox pattern | Spawn a job, Channel worker, Cron task, Job queue, Message consumer |
| 32 | Calling other services and proxying | A | `reqwest` 0.13, timeouts and retries, webhooks with signature checks, reverse proxy and HTTP proxy examples, forwarded headers | Call an API, Retry client, Webhook receiver, Reverse proxy, Forwarded headers |

### Part 6: API styles
| # | Module | Level | What you'll learn | 5 working examples |
|---|---|---|---|---|
| 33 | OpenAPI documentation | A | `utoipa` 6 with `utoipa-axum` `OpenApiRouter`, schemas, Swagger UI and Scalar, `aide` as a mention only (its optional axum-extra support is pinned to 0.10, not 0.12) | Documented CRUD, Schemas and examples, Swagger UI, Scalar UI, Client generation |
| 34 | GraphQL | X | `async-graphql` with Axum, queries, mutations, subscriptions, state and auth in resolvers | Hello schema, CRUD graph, Subscriptions, DataLoader, Auth'd resolver |
| 35 | gRPC with tonic | X | `tonic` 0.14, protobuf, serving gRPC and HTTP together, shared state, when to choose which | Unary call, Server stream, Shared state, Mixed REST and gRPC, Client |

### Part 7: Quality and operations
| # | Module | Level | What you'll learn | 5 working examples |
|---|---|---|---|---|
| 36 | Testing | A | Handler tests, `Router` with `oneshot`, `http-body-util`, tests over a real TCP listener, WebSocket tests, database tests (`sqlx::test`, `testcontainers`), mocks (`mockall`, `wiremock`), snapshots (`insta`), `rstest` | Route test, JSON test, Auth test, DB test, WebSocket test |
| 37 | Logging, tracing and metrics | A | `tracing`, `tracing-subscriber`, JSON logs, `TraceLayer`, spans and request IDs, OpenTelemetry export, Prometheus metrics, `tokio-console`, health and readiness | Structured logs, Request spans, OTLP trace, Metrics endpoint, Health probes |
| 38 | Configuration and project structure | A | Environment variables, `dotenvy`, `config` crate, CLI with `clap`, secrets, workspace layout, feature flags | Env config, Layered config, CLI flags, Secrets handling, Workspace layout |
| 39 | Async and debugging handlers | X | Blocking vs async and `spawn_blocking`, cancellation safety, `select!` and timeouts, backpressure, `Send` and `Sync` errors, `#[debug_handler]`, fixing "handler trait not satisfied" | Blocking bug, Cancellation demo, Timeout with select, debug_handler fix, Send error fix |
| 40 | Performance | X | Release profile, allocator choices, connection pools, compression cost, HTTP/2, load testing, profiling, benchmarking with `criterion` | Baseline benchmark, Pool sizing, Allocator compare, Load test, Flamegraph |

### Part 8: Servers and packaging
| # | Module | Level | What you'll learn | 5 working examples |
|---|---|---|---|---|
| 41 | `axum::serve` in depth | X | `Listener` and `ListenerExt::tap_io`, TCP options, `ConnectInfo` and `into_make_service_with_connect_info`, graceful shutdown with signals, Unix domain sockets, serving with Hyper directly, HTTP/2, socket handoff for reloads | Graceful shutdown, Unix socket, Custom listener, Hyper directly, Socket handoff |
| 42 | TLS and HTTPS | A | `axum-server` 0.8 with rustls, low-level rustls/OpenSSL/native-tls, graceful TLS shutdown, HTTP to HTTPS redirect, HSTS, reverse-proxy TLS | HTTPS server, Redirect to HTTPS, Low-level rustls, HSTS header, TLS shutdown |
| 43 | Docker and containers | A | Multi-stage builds, dependency caching, small runtime images, health checks, `docker-compose` with Postgres and Redis | Basic image, Cached build, Compose stack, Healthcheck, Non-root image |
| 44 | Axum internals and custom Services | X | How `Router` works (`matchit`), the `Handler` trait, `IntoResponse`, bodies, hand-written `Service` and `Layer`, building on Tower, Axum on wasm | Hand-written Service, Custom Layer, Handler trait tour, Router internals, Wasm router |

### Capstone projects (one folder each)
1. **Todo REST API**: in-memory, then Postgres with SQLx (after Part 3).
2. **Auth service**: users, JWT and refresh tokens, roles (after Part 4).
3. **Realtime chat**: WebSockets plus SSE, Redis pub/sub across instances (after Part 5).
4. **Video and music streaming site**: upload big videos in resumable chunks (28), download with Range (29), stream to a `<video>`/`<audio>` player with seeking (30), optional HLS, thumbnails (after Part 5).
5. **Link shortener or blog**: server-rendered HTML with HTMX, sessions, caching.
6. **Production API**: Postgres, auth, uploads, OpenAPI, tracing and metrics, Docker (the counterpart of your Leptos module 25).

---

## 4. Mapping from your old 12 modules

| Old module | New module(s) |
|---|---|
| 01 Intro | 01, 02, 03 |
| 02 Routing | 04 |
| 03 Extractors | 05, 09 |
| 04 Responses | 03, 06 |
| 05 State | 07 |
| 06 Middleware | 10, 11, 12 |
| 07 Errors | 08 |
| 08 Database | 14, 15, 16 |
| 09 Auth | 19, 20, 21, 22 |
| 10 Advanced (WS, SSE, uploads) | 24, 25, 26, 27 (+ new 28, 29, 30 for big files and media) |
| 11 Testing | 36 |
| 12 Production | 37, 38, 41, 43 + capstone 6 |

**New topics (no old equivalent):** 02, 09, 13, 17, 18, 23, 28, 29, 30, 31, 32, 33, 34, 35, 39, 40, 42, 44.

---

## 5. Root repo files

`Cargo.toml` (workspace with pinned versions), `README.md` (requirements: Rust 1.80+, quick start, module table with port and run command, tracks, "What changed in 0.8"), `LICENSE`, `.gitignore`, `.env.example`, `docker-compose.yml` (Postgres 16, Redis 7), `Dockerfile` (capstone 6), `UPDATE-axum-0.8.9.md` (unreleased items), `migrations/` per database module, and `cargo test --workspace` in CI.

---

## 6. What I verified and what to check

**Verified on this machine against the published crates:** versions on crates.io; the 0.8.9 changelog; that these exist in axum 0.8.9 / axum-core 0.5.6 / axum-extra 0.12.6: `route_layer`, `nest_service`, `fallback_service`, `method_not_allowed_fallback`, `reset_fallback`, `without_v07_checks`, `into_make_service_with_connect_info`, `ListenerExt::tap_io`, `DefaultBodyLimit`, `FromRef`, `MatchedPath`, `NestedPath`, `OriginalUri`, `RawPathParams`, `RawQuery`, `RawForm`, `ConnectInfo`, `WebSocketUpgrade`, `Multipart`, `Sse`, `Redirect`, `NoContent`, `AppendHeaders`, `IntoResponseParts`, `ErrorResponse`, `from_fn_with_state`, `from_extractor`, `map_request_with_state`, `map_response_with_state`, `HandleErrorLayer`, `OptionalFromRequest(Parts)`, `CookieJar`, `SignedCookieJar`, `PrivateCookieJar`, `TypedHeader`, `WithRejection`, `Cached`, `JsonLines`, `Protobuf`, `Resource`, `TypedPath`, `RouterExt::route_with_tsr`, `Attachment`, `FileStream`, `ErasedJson`, `JsonDeserializer`, `AsyncReadBody`, `option_layer`; that `ServeDir`/`ServeFile` (tower-http 0.7.1) answer a single byte range with `206 Partial Content`, `Accept-Ranges: bytes` and `Content-Range`, and answer unsatisfiable or multi-range requests with `416` (multi-range can be ignored with `ignore_multi_range_requests`); that axum-extra `FileStream` has `into_range_response` and `try_range_response`; that the `headers` 0.4.2 crate has typed `Range`, `ContentRange`, `AcceptRanges` and `IfRange`; the 55 official examples in the Axum repo; the tower-http 0.7.1 feature list.

**Check before recording (I did not compile a demo for each):** exact APIs of `utoipa-axum` 0.3, `tower-sessions` 0.15, `axum-login` 0.18, `tower-governor` 0.8, `apalis` 1.0, `async-graphql-axum` 7.2, `tonic` 0.14 with Axum 0.8, `sqlx` 0.9 (new major version), `jsonwebtoken` 11 and `argon2` 0.6 (both changed major versions since your old course), and the OpenTelemetry 0.33 setup. Several crates are on new majors, so snippets from older tutorials may not compile.
- **Modules 28-30 (big files and media):** I verified the Range building blocks above, but I did not write or run a chunked-upload server or an HLS pipeline. HLS generation needs `ffmpeg` installed. For resumable uploads I found no maintained Rust server crate for the tus protocol (the `tus` crate's last release is Dec 2023), so the course should teach the protocol and build the chunk logic by hand.
- **Dependency compatibility with Axum 0.8, checked on crates.io:** compatible: `tower-governor` 0.8 (axum ^0.8), `utoipa-axum` 0.3 / `utoipa-swagger-ui` 10 / `utoipa-scalar` 0.4 (axum ^0.8.4), `async-graphql-axum` 7.2.1 (axum ^0.8.1), `tonic` 0.14 (optional axum ^0.8), `axum-htmx` 0.8.1 (axum-core ^0.5), `axum-server` 0.8 (no axum dependency), `tracing-opentelemetry` 0.34 with `opentelemetry` 0.33, `sea-orm` 2.0.4 with sqlx 0.9. **Not compatible as a set:** `axum-login` 0.18 with `tower-sessions` 0.15, the published tower-sessions SQLx and Redis stores with tower-sessions 0.15, and `aide` 0.15 with axum-extra 0.12. This only checks dependency versions; I have not compiled demos with these crates.
- **Whole-workspace resolution test:** because your repo is one Cargo workspace (one shared lockfile), I put every pinned crate (without `axum-login` and the SQLx/Redis session stores) into one test crate and ran `cargo generate-lockfile`. It resolved **749 packages with no conflicts**: one `axum` 0.8.9, one `sqlx` 0.9.0, one `tokio` 1.54.1, one `hyper` 1.12.0. The only duplicate among the key crates is `tower-http`: 0.7.1 (yours) plus 0.6.11 pulled in by `reqwest` 0.13 through `opentelemetry-otlp`. That is harmless, but do not mix the two types. Diesel was resolved with the `postgres` feature only; combining Diesel's SQLite support with SQLx's SQLite support in one workspace can hit a native-library `links` conflict, so teach SQLite through SQLx only. This proves the versions can coexist; it does not prove every API example compiles.
- `dotenvy` has not been released since 2023; the `config` crate is the safer teaching default.
- I found no official Axum 0.9 roadmap. The web search for it returned nothing useful.

## Sources
- Your old course: https://github.com/aarambh-darshan/axum-full-course
- Axum repository, changelogs and 55 examples: https://github.com/tokio-rs/axum
- Announcing axum 0.8.0: https://tokio.rs/blog/2025-01-01-announcing-axum-0-8-0
- axum crate docs: https://docs.rs/axum
- tower-http changelog (0.7.x): https://github.com/tower-rs/tower-http
- Crate versions: https://crates.io
