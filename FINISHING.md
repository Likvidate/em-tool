# Finishing Plan 1 — last steps

Plan 1 (Foundation) is **code-complete**. Everything buildable without sudo has been built and tested. What remains requires system libraries to install the Rust side of Tauri.

## What's working right now (no action needed)

- Vite production build: `npm run build` ✓ (100 kB main chunk, 39 kB gzipped)
- TypeScript typecheck: `npx vue-tsc --noEmit` ✓
- Frontend test suite: `npm run test` — 3/3 pass (idle-timer tests)
- Rust toolchain: rustc 1.95.0 (already upgraded via `rustup update stable`)
- SSH auth to GitHub: ✓ (pushed as Likvidate)

## What you need to do

### 1. Install the Linux system libraries Tauri needs

```bash
sudo apt update
sudo apt install -y \
    libwebkit2gtk-4.1-dev \
    libglib2.0-dev \
    libgtk-3-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev \
    libssl-dev \
    libxdo-dev \
    build-essential \
    curl wget file pkg-config
```

### 2. Verify the Rust side compiles and tests pass

```bash
cd ~/em-tool/src-tauri
cargo test --lib
```

Expected output (first run will take 5–10 min as rusqlite with bundled-sqlcipher compiles its C sources):

```
running 12 tests
test kdf::tests::derive_key_is_deterministic ... ok
test kdf::tests::different_passwords_produce_different_keys ... ok
test kdf::tests::different_salts_produce_different_keys ... ok
test kdf::tests::random_salt_has_correct_length ... ok
test migrations::tests::all_tables_exist_after_v1 ... ok
test migrations::tests::color_check_constraint_rejects_invalid ... ok
test migrations::tests::first_run_reaches_current_version ... ok
test migrations::tests::run_is_idempotent ... ok
test migrations::tests::unique_week_rating_per_report_per_week ... ok
test vault::tests::create_rejects_existing_file ... ok
test vault::tests::create_then_open_roundtrip ... ok
test vault::tests::wrong_password_fails ... ok

test result: ok. 12 passed; 0 failed
```

If any test fails, see the troubleshooting section below.

### 3. Run the full app (Tauri shell + Vue)

```bash
cd ~/em-tool
npm run tauri dev
```

A native window should open. First-run flow → set a password → lands on Weekly Capture stub. Close and re-open → Unlock screen → enter password → lands on capture again.

## Plan 1 smoke test — run this manually

From the plan's Task 11, step 3 — click through this exactly:

1. First launch → enter password `testpass1234` in both fields, tick the no-recovery acknowledgement, click "Create vault". → should transition to the Weekly Capture stub with the sidebar on the left.
2. Close the window entirely.
3. Relaunch: `npm run tauri dev` → should land on the Unlock view (vault exists, locked).
4. Enter wrong password `wrong` → inline "Wrong password." message.
5. Try 4 more wrong attempts → on the 5th failure, the button should show "Too many attempts — try again in Ns" and the input should disable.
6. Wait for the countdown to elapse, then enter the correct password (`testpass1234`) → unlocks, routes to Weekly Capture.
7. Click each sidebar item — all 5 stubs should render.

Verify the vault was created:
```bash
ls -la ~/.local/share/em-tool/
# Expected: vault.db and vault.salt both exist
```

If that all works, tag the completion:
```bash
cd ~/em-tool
git tag -a plan1-foundation-complete -m "Plan 1 (Foundation) — scaffold + encrypted vault + shell"
git push origin plan1-foundation-complete
```

## Troubleshooting

**`cargo test` fails with "package 'glib-2.0' was not found"** — the system libs aren't installed. Re-run step 1.

**`cargo test` compiles forever the first time** — this is normal. It's compiling rusqlite with bundled-sqlcipher (significant C code) plus the whole tauri stack. Subsequent builds are fast (incremental).

**`npm run tauri dev` fails with WebView2 errors on Windows** — install the WebView2 runtime from Microsoft (bundled with Windows 11; manual install on Windows 10).

**Wrong password works anyway / vault opens with any password** — this should be impossible, but if you see it, check that `src-tauri/src/vault.rs::open` still uses `Err(rusqlite::Error::SqliteFailure(_, _)) => Err(VaultError::InvalidPassword)` after the `SELECT count(*) FROM sqlite_master` probe. Don't proceed to Plan 2 — that code path is core to the security model.

## What's next

When Plan 1 is tagged as complete, ask the AI to **write Plan 2 (Capture & viewing)** — reports CRUD, weekly capture grid, per-person timeline, team heatmap. That plan will be generated fresh based on what was actually shipped in Plan 1, reflecting any small design tweaks that surfaced during implementation (e.g. the `lib.rs`/`main.rs` split, the sidecar salt file approach).
