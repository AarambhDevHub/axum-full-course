# Module 04 - Routing

## What You'll Learn

- The route table: `Router::new().route(...)` with one method selector per row
- Method selectors: `get`, `post`, `put`, `delete`, `head`, `options`, `any` — chained on one path
- Path parameters `/{id}` and the wildcard `/{*rest}` (0.8 syntax; `/:id` and `/*rest` panic)
- `nest` — mounting a router under a prefix with its own subtree fallback
- `merge` — bolting independently-built routers together (+ why `reset_fallback` exists)
- `fallback` for a branded 404 and `method_not_allowed_fallback` for a branded 405
- `route_service` / `nest_service` — mounting *services* instead of handlers
- Versioning with `/v1`, `/v2` nested routers
- Automatic `HEAD` handling on registered GET paths

## Prerequisites

- Modules 01–03 (you know handlers, responses, `Json`, `NoContent`)

## How to Run

```bash
cargo run -p module-04-routing
```

This module listens on **http://127.0.0.1:3004** (set `AXUM_PORT` to change it).

## Endpoints

| Method | Path | Status | Purpose |
|--------|------|--------|---------|
| GET | `/` | 200 | The route tree dashboard |
| GET | `/items` | 200 | List items (in-memory store) |
| POST | `/items/{name}` | 201 | Create item (name in path; bodies come in Module 05) |
| GET | `/items/{id}` | 200/404 | Read item (id must be numeric, else 404-style miss) |
| PUT | `/items/{id}/{name}` | 200/404 | Rename item |
| DELETE | `/items/{id}` | 204/404 | Delete item |
| GET | `/files/{*path}` | 200 | Wildcard route demo |
| GET | `/api/status`, `/api/version` | 200 | Routes inside a nested router |
| GET | `/api/anything-else` | 404 JSON | The **nested** router's fallback |
| GET | `/admin/keys` | 200 | Route from a `merge`d router (`reset_fallback` demo) |
| GET | `/v1/hello` · `/v2/hello` | 200 | Versioning via two nested routers |
| GET | `/legacy/anything` | 200 | Mounted as a **service** (`nest_service`) |
| any | unknown path | 404 JSON | Main app `fallback` |
| PATCH | `/items/1` | 405 JSON | `method_not_allowed_fallback` |
| any | `/echo-method` | 200 | `any(handler)` — every method |
| GET, POST | `/read-or-write` | 200 (PUT → 405) | `on(MethodFilter::GET.or(MethodFilter::POST), ..)` |
| GET, HEAD | `/head-demo` | 200 | Automatic HEAD vs an explicit `.head(..)` handler |
| GET | `/typed-items/{id}` | 200 / 400 / 404 | Typed `Path<u32>` (400 on `abc`) |
| GET | `/docs` and `/docs/` | 200 / 308 | `route_with_tsr` — trailing-slash redirect |
| GET | `/service` | 200 | `route_service` with a hand-written `service_fn` |
| GET | `/svc/known`, `/svc/anything` | 200 | Handler row + `fallback_service` inside a nested router |

## Key Concepts

```rust
Router::new()
    .route("/items/{item}", post(create).get(read).delete(remove))
    .route("/files/{*rest}", get(wildcard))
    .nest("/api", api_router)            // subtree + prefix
    .merge(admin_router)                 // bolt-on
    .fallback(not_found_json)            // branded 404
    .method_not_allowed_fallback(not_allowed_json) // branded 405

// Lesson 7-8
    .route("/echo-method", any(echo_method))                       // every method
    .route("/read-or-write", on(MethodFilter::GET.or(MethodFilter::POST), h))
    .route("/head-demo", get(get_h).head(head_h))                  // explicit HEAD
    .route_with_tsr("/docs", get(docs))                            // /docs/ -> /docs
    .route_service("/service", service_fn(my_service))             // a tower Service
    .nest("/svc", Router::new().fallback_service(service_fn(my_service)))
```

- One **MethodRouter per path pattern**; chain method selectors onto it.
- Handler vs **service**: `route`/`nest`/`fallback` take handlers; `route_service`/
  `nest_service`/`fallback_service` take any tower `Service` (`ServeDir`, a proxy, gRPC).
- With only `get(..)` registered, Axum answers `HEAD` for you (same headers, no body).
- `/docs` and `/docs/` are different paths. `route_with_tsr` (axum-extra, feature
  `routing`) registers one and redirects the other.
- Param names must be consistent for the same position (`{item}`, not `{id}` twice).
- Nested routers keep their own fallback; `merge` forbids two explicit fallbacks,
  which is exactly what `reset_fallback()` is for.

## Working Examples

- CrudRouteTable — `/items`
- ParamAndWildcard — `/items/{item}`, `/files/{*path}`
- NestedApi — `/api/*` with subtree fallback
- MergedRouters — `/admin/keys` + `reset_fallback`
- VersionedApi — `/v1/hello` vs `/v2/hello`
- ServicesAsRoutes — `/legacy/*` via `nest_service`
- MethodSelectors — `/echo-method` (`any`), `/read-or-write` (`on`), `/head-demo`
- TypedParams — `/typed-items/{id}`
- TrailingSlashAndServices — `/docs`, `/service`, `/svc/*`

## Try It

```bash
BASE=http://127.0.0.1:3004

curl -si $BASE/                                  # the route tree
curl -si $BASE/items                             # list
curl -si -X POST $BASE/items/lantern             # 201, "created item 3"
curl -si $BASE/items/3                           # 200 "item 3 is 'lantern'"
curl -si -X PUT $BASE/items/3/lantern-of-light   # 200 rename
curl -si -X DELETE $BASE/items/3                 # 204
curl -si $BASE/items/banana                      # 404-style miss (not an id)
curl -si $BASE/files/assets/css/main.css         # 200, wildcard demo
curl -si $BASE/api/status                        # nested router
curl -si $BASE/api/wat                           # nested fallback (JSON)
curl -si $BASE/wat                               # main fallback (JSON) — different!
curl -si $BASE/admin/keys                        # merged router
curl -si $BASE/v1/hello && curl -si $BASE/v2/hello
curl -si $BASE/legacy/anything                   # nest_service
curl -si -X PATCH $BASE/items/1                  # branded 405
curl -I $BASE/items                              # automatic HEAD (empty body)
curl -si $BASE/items/                            # trailing slash = different path -> 404

curl -s -X PATCH $BASE/echo-method               # any() accepts every method
curl -s -X POST  $BASE/read-or-write             # on(GET|POST)
curl -si -X PUT  $BASE/read-or-write             # 405
curl -si $BASE/head-demo && curl -sI $BASE/head-demo   # note x-served-by differs
curl -si $BASE/typed-items/1                     # typed Path<u32>
curl -si $BASE/typed-items/abc                   # 400, handler never runs
curl -si $BASE/docs/                             # 308 -> /docs (route_with_tsr)
curl -s  $BASE/service                           # hand-written service_fn
curl -s  $BASE/svc/known && curl -s $BASE/svc/anything
```

## Common Mistakes

- Copying `/:id` or `/*rest` from a pre-0.8 tutorial — Axum 0.8 **panics at
  startup** with "invalid route"; the course syntax is `/{id}`, `/{*rest}`.
- Two param names for the same position (`/users/{id}` and `/users/{name}`) —
  routing treats them as one "*" position, so names must match once per position.
- Sending POST to `/items` (no name available yet) — this module carries data
  in paths on purpose; JSON request bodies are Module 05.
- Expecting `/items/` to match `/items` — trailing slash is a different row;
  be consistent (Module 12's NormalizePathLayer can fuse them).
- Merging two routers that both set `fallback()` — construction panics; call
  `.reset_fallback()` on whichever router's fallback you want to disinherit.
- Forgetting that `nest`ed handlers don't see the prefix — write handlers
  prefix-free and let `nest` name them.

- Writing `.route("/docs/", ..)` and expecting `/docs` to work — `route_with_tsr`
  redirects only the *other* form, so register the form you want users to land on.
- Using `fallback` with a `Service` — services go through `fallback_service`.

## Exercises

1. Add `GET /items/count` — path params actually make `/items/count` win over
   `/items/{item}` for leaf paths; try it and see exact-vs-param priority.
2. Nest a `/users` router with `GET /users/{id}` and its own fallback.
3. Add `GET /v0/hello` that 301s (redirect is yours from Module 03) to `/v1/hello`.
4. Make `PATCH /items/1` answer with your *own* status code choice in the
   method fallback (keep it 405 in real services).
5. Mount the same `api_routes` router at a second prefix, e.g. `nest("/v3/api", ...)`,
   and curl both — proving handlers never see their prefix.

Previous: [Module 03](../module-03-handlers-responses)

Next: [Module 05 - Extractors](../module-05-extractors)
