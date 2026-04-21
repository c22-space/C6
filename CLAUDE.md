# C12 — Agent Configuration

## Stack

- **Frontend**: Leptos 0.7 (Rust/WASM, CSR), Trunk, Tailwind CDN — in `ui/`
- **Backend**: Tauri v2, Rust, rusqlite (bundled SQLite) — in `src-tauri/`
- **Worker**: Cloudflare Worker (Rust, workers-rs 0.4), D1, R2 — in `worker/`
- **Build**: `cargo tauri dev` / `cargo tauri build`; frontend served by Trunk at :5173

## Engineering Principles

### Correctness before everything
- Every error must be propagated to the caller — never swallowed with `let _ =` unless it is genuinely fire-and-forget (log it if so)
- SQLite queries must be parameterised — no string interpolation in SQL
- All numeric types must be chosen deliberately: use `f64` for emission values, `i64` for IDs and timestamps, `i32` for years
- Tauri commands return `Result<T, AppError>` — never `Result<T, String>` in new code; `AppError` must implement `Serialize`

### Type safety
- No `serde_json::Value` in command return types — define explicit structs
- All Leptos signals must carry the most specific type available (`RwSignal<Vec<EmissionSource>>`, not `RwSignal<Vec<serde_json::Value>>`)
- Avoid `.unwrap()` in production paths; use `?` or pattern-match with meaningful fallbacks
- Tauri args structs must carry `#[serde(rename_all = "camelCase")]` — Tauri v2 expects camelCase on the wire

### Minimal surface area
- No helper functions, abstractions, or utilities that are not called from at least two sites
- No `mod utils` or `mod helpers` — code lives in the module that owns it
- No feature flags or backwards-compat shims; change the code directly
- Dead code must be deleted, not commented out

### Leptos patterns
- Use `Memo::new` for derived state, not recomputing in every closure
- `Effect::new` for side effects; never call `spawn_local` outside an Effect or event handler
- Prefer `on:blur` for text input saves (not `on:input`) to avoid per-keystroke Tauri calls
- `provide_context` / `use_context` for global state — no prop-drilling

### SQLite (rusqlite)
- All connections share the same `Database(Mutex<Connection>)` — hold the lock for the minimum duration
- Use `conn.prepare` + `query_map` for multi-row queries; `conn.query_row` for single-row
- All custom math functions (SQRT, POWER) are registered at connection open time in `db.rs`
- Migrations are append-only numbered SQL files in `src-tauri/migrations/`

### Cloudflare Worker (Rust)
- All route handlers take `(Request, &Env)` — no global state
- JWT signing/verification uses HMAC-SHA256 via the `hmac` + `sha2` crates (no Web Crypto)
- Column names in D1 upserts are validated against a compile-time whitelist — no arbitrary user-controlled SQL
- D1 binds use `JsValue::from_str` / `JsValue::from_f64` / `JsValue::null()` — never raw JS eval

### What not to do
- Do not add comments explaining what code does — name things well instead
- Do not create `README.md`, planning docs, or analysis files unless explicitly asked
- Do not add error handling for paths that cannot happen (Leptos reactive system invariants, etc.)
- Do not use `cargo fmt` style rewrites as part of a bug fix — keep diffs minimal
- Do not install npm, pnpm, or any Node tooling — the entire project is Rust

## Secrets required (GitHub Actions)

| Secret | Purpose |
|---|---|
| `TAURI_SIGNING_PRIVATE_KEY` | Tauri updater signing key (generated with `cargo tauri signer generate`) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Password for the signing key |
| `CLOUDFLARE_ACCOUNT_ID` | For R2 upload in release workflow |
| `CLOUDFLARE_API_TOKEN` | R2 write access token |
