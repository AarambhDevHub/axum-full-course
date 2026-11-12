# Module 02 - Rust and HTTP Essentials

## What You'll Learn

- `async fn`, `.await` and the Tokio runtime — what actually happens when a handler waits
- Traits and generics — the two ideas `IntoResponse`, `FromRequest` and `Handler` are built from
- Closures and `move` — the shape of every `tokio::spawn` and every `.layer(from_fn(..))`
- `Arc` + `Mutex` for data shared between tasks
- `serde` — JSON in a struct and back out
- `Result` and the `?` operator — errors as values
- HTTP anatomy: request line, URL, headers, body (take a real request apart)
- Methods, status codes and a REST map (resources + verbs + statuses)
- `curl` flag by flag (with the httpie twin)

## Prerequisites

- Module 01 completed
- Basic Rust syntax (structs, enums, `match`)

## How to Run

```bash
cargo run -p module-02-rust-http-essentials
```

This module listens on **http://127.0.0.1:3002** (set `AXUM_PORT` to change it).

## Endpoints

| Method | Path | Status | Lesson |
|--------|------|--------|--------|
| GET | `/` | 200 | Dashboard of every lesson |
| GET | `/async-basics` | 200 | 1 · `async`/`.await`, Tokio sleep, elapsed time |
| GET | `/serde-roundtrip` | 200 | 2 · JSON → struct → JSON |
| GET | `/counter` | 200 | 3 · `Arc<Mutex>` across three `tokio::spawn` tasks |
| GET | `/status/created` · `bad-request` · `teapot` · `rate-limited` · `server-error` · `maintenance` | 201 / 400 / 418 / 429 / 500 / 503 | 4 · status-code tour |
| POST | `/only-post` | 200 (GET → 405) | 5 · methods and 405 |
| GET | `/maybe-fail` | 200 / 503 | 6 · `Result` and `?` |
| GET | `/traits-generics` | 200 | 7 · traits, generics, `impl Trait`, `dyn Trait` |
| GET | `/closures` | 200 | 8 · `Fn`, `FnMut`, `move` |
| ANY | `/anatomy` | 200 | 9 · your request taken apart |
| GET | `/rest-map` | 200 | 10 · REST table as JSON |
| GET | `/curl-cheatsheet` | 200 | 10 · curl/httpie flags |

## Key Concepts

```rust
// A trait is a promise; a generic function accepts anything that keeps it.
trait Describe { fn describe(&self) -> String; }
fn describe_all<T: Describe>(items: &[T]) -> Vec<String> { /* .. */ }

// `?` returns early with the error, converting it with `From`.
fn coin_flip(nanos: u32) -> Result<&'static str, StatusCode> { /* .. */ }
async fn maybe_fail() -> Result<String, StatusCode> {
    let verdict = coin_flip(nanos)?;
    Ok(format!("..{verdict}"))
}
```

- `.await` is a **suspension point**, not a sleep: the thread serves other requests meanwhile.
- A spawned task outlives your handler, so it must **own** what it uses → `move`.
- Axum's `Handler`, `IntoResponse` and `FromRequest` are ordinary traits; once traits
  and generics feel normal, the framework stops being magic.
- A request is just text: `METHOD /path?query HTTP/1.1`, header lines, a blank line, an optional body.
- *Safe* methods never change data; *idempotent* methods can be retried safely.

## Working Examples

- AsyncBasics — `/async-basics`
- SerdeRoundtrip — `/serde-roundtrip`
- ArcMutexCounter — `/counter`
- StatusTour — `/status/*`
- MethodsAndResult — `/only-post`, `/maybe-fail`
- TraitsGenericsClosures — `/traits-generics`, `/closures`
- HttpAnatomy — `/anatomy`, `/rest-map`, `/curl-cheatsheet`

## Try It

```bash
BASE=http://127.0.0.1:3002

curl -s  $BASE/async-basics               # ~200 ms, runtime stays free
curl -s  $BASE/serde-roundtrip            # parsed + re-served JSON
curl -s  $BASE/counter                    # always exactly 3000
curl -si $BASE/status/teapot              # 418
curl -si $BASE/only-post                  # 405 + Allow header
curl -si -X POST $BASE/only-post          # 200
curl -s  $BASE/maybe-fail                 # run it a few times: 200 or 503
curl -s  $BASE/traits-generics
curl -s  $BASE/closures

# Anatomy: compare what you SENT (curl -v) with what the server SAW.
curl -s -X POST -H 'x-demo: 1' -d 'hello' "$BASE/anatomy?lesson=9"
curl -v -X POST -d 'hello' $BASE/anatomy 2>&1 | head -20

curl -s $BASE/rest-map
curl -s $BASE/curl-cheatsheet
```

## Common Mistakes

- Using `std::thread::sleep` in a handler — it blocks a worker thread. Use `tokio::time::sleep(..).await`.
- Holding a `std::sync::Mutex` guard across an `.await` — fine for nanoseconds, a hazard for I/O (Module 07 shows the tokio lock).
- Forgetting `move` on a spawned task and getting "borrowed value does not live long enough".
- Thinking `POST` is idempotent — retrying a POST can create two resources.
- Putting the verb in the URL (`/createBook`) instead of the method (`POST /books`).

## Exercises

1. Add a `Describe` impl for a third type and print it in `/traits-generics`.
2. Change `/counter` to 8 workers × 5,000 increments. Does it still end exact?
3. Add `GET /status/gone` returning `410 Gone`.
4. Send a request with three custom headers to `/anatomy` and find them in the output.
5. Replace the `Mutex<u64>` in `/counter` with `AtomicU64` (you will do this properly in Module 07).

Previous: [Module 01](../module-01-intro)

Next: Module 03 - Handlers and First Responses (coming soon — published with its video)
