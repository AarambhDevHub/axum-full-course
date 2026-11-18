# Module 03 - Handlers and First Responses

## What You'll Learn

- Handlers are plain async functions — no structs, macros or traits needed
- Every built-in response type: `&'static str`, `String`, `Html<T>`, `Json<T>`, `StatusCode`
- Status codes **with** bodies via `(StatusCode, T)` response tuples
- Empty-body success with `NoContent` (204)
- `Redirect::to` (303), `Redirect::temporary` (307), `Redirect::permanent` (308)
- The health-check endpoint pattern
- `impl IntoResponse` returns and `.into_response()` for handlers with several branches
- A reference table of every built-in return type (`/types`)
- `Json` with a non-200 status, and raw bytes with an explicit content type

## Prerequisites

- Module 02 completed (you know `curl -i` and status codes)

## How to Run

```bash
cargo run -p module-03-handlers-responses
```

This module listens on **http://127.0.0.1:3003** (set `AXUM_PORT` to change it).

## Endpoints

| Method | Path | Status | Content type | Purpose |
|--------|------|--------|--------------|---------|
| GET | `/` | 200 | text/html | Dashboard (an `Html` handler itself) |
| GET | `/greeting` | 200 | text/plain | `String` built at request time |
| GET | `/page` | 200 | text/html | Static HTML page |
| GET | `/user` | 200 | application/json | Typed JSON view struct |
| GET | `/created` | 201 | text/plain | `(StatusCode::CREATED, body)` tuple |
| GET | `/teapot` | 418 | text/plain | 418 with a body |
| GET | `/no-content` | 204 | — | `NoContent` — done, nothing to say |
| GET | `/old-page` | 303 → `/page` | — | `Redirect::to` (See Other) |
| GET | `/moved` | 308 → `/` | — | `Redirect::permanent` |
| GET | `/health` | 200 | application/json | The probe load balancers call first |
| GET | `/maybe?mode=json\|html\|text` | 200 | varies | Three types, one handler, via `.into_response()` |
| GET | `/types` | 200 | application/json | Every built-in return type in one table |
| GET | `/temporary` | 307 → `/page` | — | `Redirect::temporary` |
| GET | `/json-created` | 201 | application/json | `(StatusCode::CREATED, Json<T>)` |
| GET | `/plain-bytes` | 200 | text/plain | `impl IntoResponse`: header array + byte slice |

## Key Concepts

```rust
async fn created_with_body() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "created")
}

async fn no_content() -> NoContent {
    NoContent
}

async fn redirect_permanent() -> Redirect {
    Redirect::permanent("/")
}
```

The rule that runs every response in this course: **whatever the handler
returns must implement `IntoResponse`.** Tuples, `Html`, `Json`,
`StatusCode`, `Redirect`, `NoContent` — and `Result<T, E>` when both sides do.

## Working Examples

- TextVsAlloc — `/` + `/greeting` (`&'static str` vs `String`)
- HtmlPage — `/page`
- JsonUser — `/user`
- StatusWithBody — `/created`, `/teapot`, `/no-content`
- RedirectFlow — `/old-page` → `/page`, `/moved` → `/`
- HealthCheck — `/health`
- IntoResponseKit — `/maybe`, `/types`, `/plain-bytes`
- MoreStatusAndRedirects — `/temporary`, `/json-created`

## Try It

```bash
BASE=http://127.0.0.1:3003

curl -si $BASE/greeting                          # dynamic String, plain text
curl -si $BASE/page                              # text/html — view in a browser too
curl -s  $BASE/user                              # JSON
curl -si $BASE/created                           # 201 + body in one response
curl -si $BASE/teapot                            # 418 + body
curl -si $BASE/no-content                        # 204, zero-length body
curl -si $BASE/old-page                          # 303 + location: /page
curl -siL $BASE/old-page                         # -L follows it (200 from /page)
curl -si $BASE/moved                             # 308
curl -s  $BASE/health                            # {"status":"ok",...}
curl -si $BASE/temporary                         # 307 (method preserved)
curl -si $BASE/json-created                      # 201 + JSON body
curl -s  "$BASE/maybe?mode=json"                 # three types, one handler
curl -s  $BASE/types                             # reference table of return types
curl -si $BASE/plain-bytes                       # bytes + explicit content-type
```

## Common Mistakes

- Returning `String` for a forever-constant body — use `&'static str`; the
  difference is an allocation per request vs free bytes (Module 40 measures it).
- Building status + body as *two* responses — one tuple does it in one.
- Replying 200 with an empty body where "nothing happened" was meant — use
  `NoContent` (204) so clients can tell success-with-no-data from success.
- Calling `Location` yourself with a `String` body — `Redirect` handles the
  header, status and URI encoding for you.
- Returning 303 after a URL was permanently changed — that's a job for 308.

- Returning different types from `if`/`else` branches — the compiler rejects it;
  call `.into_response()` on each branch (see `/maybe`) or return `Response`.
- Forgetting a content type on raw bytes — they default to `application/octet-stream`.

## Exercises

1. Add `GET /bye` returning a `(StatusCode::OK, &'static str)` tuple, then
   `HEAD /bye` (compare `curl -I` vs `curl -s`).
2. Make `/health` include a `module` field with the number 3.
3. Chain two redirects: `/a` → `/b` → `/page`, and follow with `curl -sL`.
4. Add `GET /created-no-body` using only `StatusCode::CREATED`, and compare
   `curl -si` output of both created endpoints.
5. Return an HTML `<table>` of the endpoints in `/page` (add a row per route).

Previous: [Module 02](../module-02-rust-http-essentials)

Next: [Module 04 - Routing](../module-04-routing)
