# Axum beyond 0.8.9 — features that are NOT released yet

This course teaches **axum 0.8.9** (verified 27 Oct 2026). The features below
exist only on the Axum `main` branch, are described in the changelog as
"Unreleased", and several are marked **breaking**, so they will land in the
next breaking release (no date announced).

**Do not teach them as working code, and do not ship code that uses them.**
Snippets here were written only so you can recognize them when they arrive.

## Listener and connection management

| Unreleased API | What it will do |
|---|---|
| `ListenerExt::limit_connections` | Cap concurrent TCP connections at the listener, instead of using `tower::limit::ConcurrencyLimitLayer` inside the router. |
| `serve::ConnectionLifetimeLimits` | Set an idle connection lifetime / max lifetime on accepted connections. |
| Hyper's default `header_read_timeout` in `axum::serve` | Headers must arrive within a deadline by default, closing slowloris-style half-open requests. |

Today's replacement is still: `tower-http`'s `RequestBodyLimitLayer`, a
`ConcurrencyLimitLayer`, and a `TimeoutLayer` (Modules 12 and 23).

## Executor hooks (breaking)

- `serve::Executor` and `Serve::with_executor` (breaking): let you choose how the
  server runs per-connection tasks (for custom runtimes, not Tokio).

## Routing and responses (breaking)

- `MethodFilter::QUERY` and `routing::query` (breaking): a dedicated `.query(...)`
  route prefix for RFC 9110 query-safe GET-like endpoints.
- `sse::Event::raw`: send SSE events with non-UTF-8 (raw bytes) payload data.
- `Redirect` as a response *part*: combine `Redirect` with other response parts
  instead of it owning the whole response.
- Nested-router fallback merging: an inner router's fallback survives when the
  outer router rewrites the address part.

## axum-extra deprecations

- The deprecated `Host` and `Scheme` extractors in `axum-extra` are removed
  (use `ConnectInfo` / typed headers instead).
- `tower-log` is no longer a default feature of `axum-extra` in the next release.

## Rule for this course

If a new Axum minor/major version ships, verify each item here against its
published changelog, then move what is now published into the module that
covers that feature (and update the version table in the root README).
