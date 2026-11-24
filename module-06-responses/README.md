# Module 06 - Responses in Depth

## What You'll Learn

- The response grammar: `(StatusCode, parts.., body)` — **parts** decorate, the **body** is last
- `IntoResponse` (the body side) and `IntoResponseParts` (the decoration side)
- Every built-in response thing, explained (`/response-kit`): text, bytes, `Html`, `Json`, `Body`,
  `Response`, `NoContent`, `Redirect`, `()`, `Result`, status codes, header arrays, `HeaderMap`,
  `AppendHeaders`, `Extension`
- Building a whole `Response` by hand with `Response::builder`
- Hand-set cookies — the raw `Set-Cookie` wire format (the jar arrives in Module 21)
- Content negotiation by the `Accept` header
- File-download headers: `Content-Disposition: attachment`, safe filenames
- `201 Created` + `Location`
- **Build your own** response types: `Created<T>`, `ApiEnvelope<T>`, `Csv`
- **Build your own** response parts: a typed `CacheControl` and a *fallible* `LinkHeader`
- Streaming a body with `Body::from_stream`

## Prerequisites

- Module 03 (built-in response types) and Module 05 (extractors, `HeaderMap`)

## How to Run

```bash
cargo run -p module-06-responses
```

This module listens on **http://127.0.0.1:3006** (set `AXUM_PORT` to change it).

## Endpoints

| Method | Path | Status | Lesson |
|--------|------|--------|--------|
| GET | `/` | 200 | Dashboard |
| GET | `/built` | 202 | 1 · `Response::builder` by hand |
| GET | `/version/{name}` | 200 | 2 · header tuple + `Json` (the "custom type" idea) |
| GET | `/stamped` | 418 | 3 · status + custom part + `AppendHeaders` + body |
| GET | `/cookies` | 200 | 4 · hand-forged `Set-Cookie` |
| GET | `/negotiate` | 200 | 5 · JSON / HTML / text by `Accept` |
| GET | `/download/{filename}` | 200 | 6 · `Content-Disposition: attachment` |
| GET | `/response-kit` | 200 | 7 · what every built-in return value becomes |
| GET | `/items/{id}` | 201 | 7/8 · `Created<T>` — status + `Location` + JSON |
| GET | `/envelope` | 200 | 8 · custom `ApiEnvelope<T>` response type |
| GET | `/report.csv` | 200 | 8 · custom `Csv` response type |
| GET | `/cache/{mode}` | 200 | 9 · custom `CacheControl` part (`no-store` / `public` / `private`) |
| GET | `/fallible-part/{value}` | 200 / 500 | 9 · a part whose `Error` is itself a response |
| GET | `/stream` | 200 | 10 · streaming body, a chunk every 300 ms |

## Key Concepts

```rust
// The grammar: [status] [parts...] body
(StatusCode::CREATED, [(header::LOCATION, "/items/9")], Json(item))
(CacheControl::NoStore, AppendHeaders([...]), "body")
```

### Build your own response type — implement `IntoResponse`

```rust
struct Created<T> { location: String, body: T }

impl<T: Serialize> IntoResponse for Created<T> {
    fn into_response(self) -> Response {
        match HeaderValue::from_str(&self.location) {
            Ok(loc) => (StatusCode::CREATED, [(header::LOCATION, loc)], Json(self.body)).into_response(),
            Err(_)  => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
```

Recipe: make a struct → `impl IntoResponse` → build any tuple inside and call `.into_response()` on it.
Any format Axum lacks (CSV, XML, YAML, MessagePack) is this same pattern.

### Build your own response part — implement `IntoResponseParts`

```rust
impl IntoResponseParts for CacheControl {
    type Error = Infallible;                       // or your own IntoResponse error
    fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
        res.headers_mut().insert(header::CACHE_CONTROL, /* value */);
        Ok(res)
    }
}
// usable anywhere before the body:  (CacheControl::NoStore, "hello")
```

- Parts are reusable decorations; when a part's `Error` is returned, **that error's response replaces the whole tuple**.
- `[(K, V); N]` and `HeaderMap` **replace** same-named headers; `AppendHeaders` **keeps duplicates** (`Set-Cookie`).
- Streaming bodies are not buffered: memory stays flat for huge files (Modules 26–30 build on this).
- Never trust a URL-supplied filename in a header — strip it first (see `/download`).

## Working Examples

- ManualResponse — `/built`
- ResponseTuples — `/version/{name}`, `/stamped`
- Cookies — `/cookies`
- Negotiation — `/negotiate`
- Downloads — `/download/{filename}`
- BuiltInKit — `/response-kit`
- CustomResponseTypes — `/items/{id}`, `/envelope`, `/report.csv`
- CustomResponseParts — `/cache/{mode}`, `/fallible-part/{value}`
- StreamingBody — `/stream`

## Try It

```bash
BASE=http://127.0.0.1:3006

curl -si $BASE/built                                  # 202 + x-built-by header
curl -si $BASE/version/axum                           # x-api-version + JSON
curl -si $BASE/stamped                                # 418, x-trace-id, x-deprecation, x-course-module
curl -si $BASE/cookies                                # Set-Cookie: visitor=yes; ...
curl -s  $BASE/negotiate
curl -s  -H 'Accept: application/json' $BASE/negotiate
curl -s  -H 'Accept: text/html' $BASE/negotiate
curl -si $BASE/download/notes.md                      # attachment; filename="notes.md"
curl -s  $BASE/response-kit
curl -si $BASE/items/9                                # 201 + Location: /items/9
curl -si $BASE/envelope                               # {"data":[..],"meta":{..}}
curl -si $BASE/report.csv                             # text/csv attachment
curl -si $BASE/cache/public && curl -si $BASE/cache/no-store
curl -si $BASE/fallible-part/ok                       # 200 + Link header
curl -si $BASE/fallible-part/a%0Ab                    # 500 from the part's own error
curl -N -s $BASE/stream                               # chunks arrive one by one
```

## Common Mistakes

- Putting the body anywhere but **last** in a tuple — the compiler rejects it.
- Expecting `[(SET_COOKIE, a), (SET_COOKIE, b)]` to send two cookies — it keeps one; use `AppendHeaders`.
- `unwrap()` on `HeaderValue::from_str` with user text — control characters make it fail. Handle the error.
- Returning `Json` for a type whose `Serialize` can fail and not expecting a 500 at runtime.
- Using `format!("attachment; filename=\"{user_input}\"")` unsanitised — header injection.
- Buffering a huge file into a `Vec<u8>` instead of streaming it.

## Exercises

1. Add `GET /moved` that returns `(StatusCode::MOVED_PERMANENTLY, [(LOCATION, "/")])` by hand.
2. Create `ApiEnvelope::paged(data, page, total)` and add pagination fields to `Meta`.
3. Add an `Etag(String)` response part (fallible if the value has quotes) and use it on `/envelope`.
4. Write `Xml(String)` — same pattern as `Csv`, content type `application/xml`.
5. Make `/stream` send `data: ..\n\n` frames and open it with `curl -N` (a preview of Module 25's SSE).
6. Send two `Set-Cookie` headers from `/cookies` with `AppendHeaders`.

Previous: [Module 05](../module-05-extractors)

Next: Module 07 - Shared State (coming soon — published with its video)
