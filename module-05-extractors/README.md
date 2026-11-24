# Module 05 - Extractors

## What You'll Learn

- What an extractor is: **any handler argument** that Axum fills from the request
- The two extractor traits and which one to use:

  | Trait | Gets | Reads the body? | Where it may sit |
  |-------|------|-----------------|------------------|
  | `FromRequestParts` | `&mut Parts` + state | no | any argument |
  | `FromRequest` | the whole `Request` + state | yes | **last** argument only |

- Every built-in extractor, explained one by one:
  `Path` (single, tuple, struct, `HashMap`), `RawPathParams`, `Query` (struct, `HashMap`),
  `RawQuery`, `Json`, `Form`, `RawForm`, `HeaderMap`, `TypedHeader`, `Bytes`, `Method`/`Uri`/`Version`,
  `MatchedPath`, `NestedPath`, `OriginalUri`, `ConnectInfo`, `Extension`, `Request`
- `Option<T>` extractors (`OptionalFromRequest[Parts]`) and `Result<T, Rejection>`
- Extractor order and why the body extractor is last
- **Build your own** extractors: `FromRequestParts` (`ApiKey`), its `Option` form,
  `FromRequest` that reads the body (`PlainLines`), a composed extractor (`ClientInfo`),
  and custom rejection types

## Prerequisites

- Module 04 completed (routes, `/{param}` rows)

## How to Run

```bash
cargo run -p module-05-extractors
```

This module listens on **http://127.0.0.1:3005** (set `AXUM_PORT` to change it).

## Endpoints

### Lessons 1-7 — the core extractors

| Method | Path | Status | Extractor |
|--------|------|--------|-----------|
| GET | `/` | 200 | JSON dashboard of the lessons |
| GET | `/products/{product_id}` | 200 / 400 | `Path<u32>` |
| GET | `/users/{user_id}/posts/{post_id}` | 200 / 400 | `Path<(u32, u32)>` |
| GET | `/search?q=..&page=..&sort=..` | 200 / 400 | `Query<SearchParams>` (required + `Option` fields) |
| POST | `/users` | 201 / 400 / 415 / 422 | `Json<UserInput>` |
| POST | `/orders/{order_id}?priority=..` | 200 / 400 / 422 | Order: Path → Query → Json (body last) |
| POST | `/feedback` | 200 / 415 / 422 | `Form<Feedback>` |
| GET | `/theme` · `/theme/{name}` | 200 | `Option<Path<String>>` |
| POST | `/optional-json` | 200 / 400 | `Option<Json<Greeting>>` |
| POST | `/strict-json` | 200 / 422 | `Result<Json<Greeting>, JsonRejection>` |
| GET | `/headers` | 200 | `HeaderMap` |
| POST | `/echo-raw` | 200 | `Bytes` |
| GET | `/meta/route` | 200 | `MatchedPath` |
| GET | `/ip` | 200 | `ConnectInfo<SocketAddr>` |

### Lesson 8 — the rest of the built-in kit

| Method | Path | Extractor |
|--------|------|-----------|
| GET | `/all-params/{org}/{repo}` | `Path<HashMap<String, String>>` |
| GET | `/repo-struct/{org}/{repo}` | `Path<struct>` (names map to fields) |
| GET | `/raw-params/{org}/{repo}` | `RawPathParams` (no parsing) |
| GET | `/query-map?a=1&b=2` | `Query<HashMap<String, String>>` |
| GET | `/raw-query?anything` | `RawQuery` |
| GET | `/uri-parts` | `Method`, `Uri`, `Version` |
| GET | `/inner/where` | `NestedPath`, `OriginalUri`, `Uri` inside a nested router |
| GET | `/extension` | `Extension<T>` and `Option<Extension<T>>` |
| GET | `/typed-header` | `TypedHeader<UserAgent>` (axum-extra) |
| POST | `/raw-form` | `RawForm` |
| POST | `/whole-request` | `Request` |

### Lesson 9 — build your own

| Method | Path | What you build |
|--------|------|----------------|
| GET | `/custom/api-key` | `ApiKey`: custom `FromRequestParts` (header `x-api-key: ak_xxxxxxxx`) |
| GET | `/custom/api-key-optional` | `Option<ApiKey>` via `OptionalFromRequestParts` |
| POST | `/custom/lines` | `PlainLines`: custom `FromRequest` reading a `text/plain` body |
| GET | `/custom/client` | `ClientInfo`: composes `ConnectInfo` + a header |

## Key Concepts

```rust
// Body extractors last, everything else before.
async fn add_order_line(
    Path(order_id): Path<u32>,          // 1. path segments
    Query(priority): Query<Priority>,   // 2. query string
    Json(line): Json<OrderLine>,        // 3. the body — always last
) -> Json<Value> { /* .. */ }
```

### Build a `FromRequestParts` extractor (no body)

```rust
struct ApiKey(String);

impl<S: Send + Sync> FromRequestParts<S> for ApiKey {
    type Rejection = ApiKeyRejection;            // any type that is IntoResponse

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let value = parts.headers.get("x-api-key").ok_or(ApiKeyRejection::Missing)?;
        /* validate, then */ Ok(ApiKey(value.to_str().unwrap().to_string()))
    }
}
```

### Build a `FromRequest` extractor (reads the body)

```rust
impl<S: Send + Sync> FromRequest<S> for PlainLines {
    type Rejection = PlainLinesRejection;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        /* check content-type, then delegate the reading (and the size limit) to Bytes: */
        let bytes = Bytes::from_request(request, state).await.map_err(PlainLinesRejection::BadBody)?;
        /* parse bytes into your type */
    }
}
```

- **No `#[async_trait]` in 0.8** — write plain `async fn` in the impl.
- A failed extractor never reaches your handler: its `Rejection` becomes the response.
- `Option<T>` compiles only when `T: OptionalFromRequest[Parts]`. In 0.8.9 that is true for
  `Path`, `Json`, `MatchedPath`, `Extension`, `TypedHeader`… and for your own types if you write the impl —
  **not** for `Query` (model optionality with `Option` fields or read `RawQuery`).
- `Result<T, T::Rejection>` turns a rejection into an ordinary value you can log or re-emit.
- Extractors compose: call other extractors *inside* yours (see `ClientInfo`).

## Working Examples

- PathParams — `/products/{id}`, `/users/{a}/posts/{b}`, `/repo-struct`, `/all-params`
- QueryParams — `/search`, `/query-map`, `/raw-query`
- JsonBody — `POST /users`
- ExtractorOrder — `POST /orders/{id}`
- TolerantReads — `/theme`, `/optional-json`, `/strict-json`
- RequestMetadata — `/headers`, `/uri-parts`, `/meta/route`, `/inner/where`, `/ip`, `/typed-header`
- Extensions — `/extension`
- RawBodies — `/echo-raw`, `/raw-form`, `/whole-request`
- CustomExtractors — `/custom/api-key`, `/custom/api-key-optional`, `/custom/lines`, `/custom/client`

## Try It

```bash
BASE=http://127.0.0.1:3005
JSON='content-type: application/json'

# --- the core ---
curl -s  $BASE/products/7                                   # 200 Path<u32>
curl -si $BASE/products/abc                                 # 400 parse rejection
curl -s  $BASE/users/7/posts/3                              # 200 tuple
curl -s  "$BASE/search?q=axum&page=2&sort=stars"            # 200 typed query
curl -si $BASE/search                                       # 400 missing q
curl -si -X POST -H "$JSON" -d '{"username":"darshan","age":25}' $BASE/users   # 201
curl -si -X POST -d 'nope' $BASE/users                      # 415
curl -si -X POST -H "$JSON" -d '{"username":"x"}' $BASE/users                  # 422 wrong shape
curl -si -X POST -H "$JSON" -d '{bad' $BASE/users                              # 400 not JSON
curl -si -X POST -H "$JSON" -d '{"sku":"D1","quantity":3}' "$BASE/orders/88?priority=high"
curl -si -X POST -d 'name=A&rating=5&comment=ok' $BASE/feedback               # Form
curl -s  $BASE/theme && curl -s $BASE/theme/dark            # Option<Path>
curl -si -X POST $BASE/optional-json                        # Option None
curl -si -X POST -H "$JSON" -d '{"message":"hi"}' $BASE/optional-json
curl -si -X POST -H "$JSON" -d '{bad' $BASE/optional-json                    # 400: optional means absent, not forgiven
curl -si -X POST -H "$JSON" -d '{bad' $BASE/strict-json     # Result arm
curl -s  -H 'x-my-header: hi' $BASE/headers
curl -si -X POST -d 'raw!!!' $BASE/echo-raw
curl -s  $BASE/meta/route && curl -s $BASE/ip

# --- the rest of the kit ---
curl -s  $BASE/all-params/tokio-rs/axum
curl -s  $BASE/repo-struct/tokio-rs/axum
curl -s  $BASE/raw-params/tokio-rs/axum
curl -s  "$BASE/query-map?a=1&b=2"
curl -s  "$BASE/raw-query?anything=goes"  && curl -s $BASE/raw-query     # Some / None
curl -s  $BASE/uri-parts
curl -s  $BASE/inner/where                                  # NestedPath vs OriginalUri vs Uri
curl -s  $BASE/extension                                    # Some(BuildInfo), None(NeverAttached)
curl -s  -A 'my-client/1.0' $BASE/typed-header
curl -si $BASE/typed-header -H 'user-agent:'                # empty/missing header -> rejection
curl -s  -X POST -d 'a=1&b=2' $BASE/raw-form
curl -s  -X POST -d 'anything' $BASE/whole-request

# --- build your own ---
curl -si $BASE/custom/api-key                               # 401 missing
curl -si -H 'x-api-key: nope' $BASE/custom/api-key          # 400 malformed
curl -si -H 'x-api-key: ak_12345678' $BASE/custom/api-key   # 200
curl -s  $BASE/custom/api-key-optional                      # anonymous
curl -s  -H 'x-api-key: ak_12345678' $BASE/custom/api-key-optional
curl -si -X POST -H 'content-type: text/plain' --data-binary $'one\ntwo\n\nthree' $BASE/custom/lines
curl -si -X POST -H "$JSON" -d '{}' $BASE/custom/lines      # 415 from our own rejection
curl -s  $BASE/custom/client
```

## Common Mistakes

- Putting a body extractor first (`Json` before `Path`) — compile error: the last
  extractor owns the consumable body.
- Expecting `Option<Query<T>>` to work because older tutorials say so — 0.8.9 has no
  `OptionalFromRequestParts` for `Query`; use `Option` fields or `RawQuery`.
- Believing a missing JSON `content-type` is a 400 — missing/wrong type is **415**;
  text that is not JSON (or trailing garbage) is **400**; valid JSON of the wrong shape or with a wrong field type is **422**.
- Writing `#[async_trait]` on a custom extractor — remove it; 0.8 uses native `async fn` in traits.
- Implementing only `FromRequestParts` for `ApiKey` and then writing `Option<ApiKey>` —
  you also need `OptionalFromRequestParts`.
- Forgetting `into_make_service_with_connect_info::<SocketAddr>()` — `ConnectInfo` then fails at runtime.
- Treating `Query<HashMap>` as validation — it accepts anything (Module 09 adds rules).

## Exercises

1. Add `GET /multiply/{a}/{b}` with `Path<(u16, u16)>` and try `a=65536`.
2. Add `Option<TypedHeader<UserAgent>>` to `/typed-header` so a missing header is tolerated.
3. Build `RequestId`, a `FromRequestParts` extractor that reads `x-request-id` or generates one.
4. Build `CsvBody`, a `FromRequest` extractor that parses `text/csv` into `Vec<Vec<String>>`.
5. Echo `X-Forwarded-For` in `/ip` when present, the direct socket otherwise (Module 32 prologue).
6. Make `ApiKey` also accept `Authorization: Bearer ak_...` and return the same rejection types.

Previous: [Module 04](../module-04-routing)

Next: [Module 06 - Responses in Depth](../module-06-responses)
