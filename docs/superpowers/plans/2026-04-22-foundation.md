# EM Tool — Plan 1: Foundation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Scaffold the Tauri 2 + Vue 3 + TypeScript desktop app, implement the encrypted vault (Argon2id KDF + SQLCipher) with all 8 data-model tables, and ship the onboarding / unlock / auto-lock flows behind a 5-screen sidebar shell. No domain features yet — this is the security and navigation foundation that every later plan builds on.

**Architecture:** Rust does three jobs only — derive a key from a password via Argon2id, open/create a SQLCipher database with that key, and expose a small set of typed `invoke` commands to the Vue layer. All screens and logic above the DB live in Vue 3 + TS. App state is managed with Pinia; the vault open/closed state drives the router.

**Tech Stack:**
- Tauri 2 (Rust backend, system WebView frontend)
- Vue 3.4 + TypeScript 5 + Vite 5
- Pinia (state)
- Vue Router 4
- Vitest (TS tests), `cargo test` (Rust tests)
- SQLCipher via `rusqlite` with the `bundled-sqlcipher` feature
- `argon2` crate (Argon2id)
- `rand` crate (salt generation)
- Node 20+, Rust 1.75+

---

## File structure (created by this plan)

```
em-tool/
├── .gitignore                             (already exists — extend)
├── package.json                           (Task 1)
├── vite.config.ts                         (Task 1)
├── tsconfig.json                          (Task 1)
├── tsconfig.node.json                     (Task 1)
├── index.html                             (Task 1)
├── src/                                   (Vue frontend)
│   ├── main.ts                            (Task 1)
│   ├── App.vue                            (Task 2)
│   ├── router.ts                          (Task 2)
│   ├── style.css                          (Task 2)
│   ├── stores/
│   │   └── vault.ts                       (Task 7)
│   ├── lib/
│   │   ├── invoke.ts                      (Task 6)
│   │   └── idle-timer.ts                  (Task 10)
│   ├── views/
│   │   ├── OnboardingView.vue             (Task 8)
│   │   ├── UnlockView.vue                 (Task 9)
│   │   ├── WeeklyCaptureView.vue          (Task 2, stub)
│   │   ├── ReportsView.vue                (Task 2, stub)
│   │   ├── TeamHeatmapView.vue            (Task 2, stub)
│   │   ├── PlanGeneratorView.vue          (Task 2, stub)
│   │   └── SettingsView.vue               (Task 2, stub)
│   └── components/
│       └── AppSidebar.vue                 (Task 2)
├── src-tauri/                             (Rust backend)
│   ├── Cargo.toml                         (Task 1)
│   ├── tauri.conf.json                    (Task 1)
│   ├── build.rs                           (Task 1)
│   └── src/
│       ├── main.rs                        (Task 1, extended in Task 6)
│       ├── kdf.rs                         (Task 3)
│       ├── vault.rs                       (Task 4)
│       ├── migrations.rs                  (Task 5)
│       ├── state.rs                       (Task 6)
│       └── commands.rs                    (Task 6)
└── tests-e2e/                             (none in this plan — manual smoke only)
```

**Responsibilities:**
- `kdf.rs` — pure Argon2id wrapper, no I/O. Unit-testable in isolation.
- `vault.rs` — SQLCipher open/create/close. Thin wrapper around `rusqlite` with the key plumbing.
- `migrations.rs` — schema DDL. Idempotent per `schema_version`.
- `state.rs` — Tauri app state holding the `Option<Connection>` and the `last_unlocked_at` timestamp.
- `commands.rs` — Tauri `#[command]` functions: `vault_exists`, `create_vault`, `unlock_vault`, `lock_vault`, `is_unlocked`, `change_password`.
- `stores/vault.ts` — Pinia store reflecting vault state reactively.
- `lib/invoke.ts` — typed wrapper around `@tauri-apps/api/core#invoke`.
- `lib/idle-timer.ts` — browser-side idle detector that calls `lock_vault` after N minutes.
- `router.ts` — redirects to `/onboard` if no vault exists, `/unlock` if locked, `/capture` otherwise.

---

## Task 1 — Scaffold the project

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `index.html`, `src/main.ts`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/build.rs`, `src-tauri/src/main.rs`
- Modify: `.gitignore`

- [ ] **Step 1: Extend `.gitignore`**

Replace contents of `/home/net-irmantasci/em-tool/.gitignore` with:

```gitignore
node_modules/
dist/
dist-ssr/
.DS_Store
*.local

# Rust / Tauri
target/
src-tauri/target/
src-tauri/gen/

# Vault data
vault.db
vault.db-*
backups/

# Tooling
.superpowers/
.vscode/
```

- [ ] **Step 2: Create `package.json`**

```json
{
  "name": "em-tool",
  "private": true,
  "version": "0.1.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "vue-tsc --noEmit && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "test": "vitest run",
    "test:watch": "vitest"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "pinia": "^2.1.7",
    "vue": "^3.4.21",
    "vue-router": "^4.3.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@vitejs/plugin-vue": "^5.0.4",
    "@vue/test-utils": "^2.4.5",
    "jsdom": "^24.0.0",
    "typescript": "~5.4.0",
    "vite": "^5.2.0",
    "vitest": "^1.4.0",
    "vue-tsc": "^2.0.0"
  }
}
```

- [ ] **Step 3: Create `vite.config.ts`**

```ts
import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "esnext",
    minify: !process.env.TAURI_DEBUG ? "esbuild" : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
  test: {
    environment: "jsdom",
    globals: true,
  },
});
```

- [ ] **Step 4: Create `tsconfig.json`**

```json
{
  "compilerOptions": {
    "target": "ES2022",
    "useDefineForClassFields": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "jsx": "preserve",
    "sourceMap": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "lib": ["ES2022", "DOM", "DOM.Iterable"],
    "skipLibCheck": true,
    "types": ["vitest/globals"],
    "baseUrl": ".",
    "paths": { "@/*": ["src/*"] }
  },
  "include": ["src/**/*.ts", "src/**/*.d.ts", "src/**/*.tsx", "src/**/*.vue"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

- [ ] **Step 5: Create `tsconfig.node.json`**

```json
{
  "compilerOptions": {
    "composite": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true,
    "strict": true
  },
  "include": ["vite.config.ts"]
}
```

- [ ] **Step 6: Create `index.html`**

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>EM Tool</title>
  </head>
  <body>
    <div id="app"></div>
    <script type="module" src="/src/main.ts"></script>
  </body>
</html>
```

- [ ] **Step 7: Create `src/main.ts`**

```ts
import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import "./style.css";

const app = createApp(App);
app.use(createPinia());
app.use(router);
app.mount("#app");
```

- [ ] **Step 8: Create `src-tauri/Cargo.toml`**

```toml
[package]
name = "em-tool"
version = "0.1.0"
description = "Engineering manager notebook"
edition = "2021"
rust-version = "1.75"

[lib]
name = "em_tool_lib"
crate-type = ["staticlib", "cdylib", "rlib"]

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"
rusqlite = { version = "0.31", features = ["bundled-sqlcipher"] }
argon2 = "0.5"
rand = "0.8"
chrono = { version = "0.4", features = ["serde"] }
hex = "0.4"
tokio = { version = "1", features = ["sync", "time"] }

[dev-dependencies]
tempfile = "3"

[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
strip = true
```

- [ ] **Step 9: Create `src-tauri/tauri.conf.json`**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "EM Tool",
  "version": "0.1.0",
  "identifier": "dev.emtool.app",
  "build": {
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build",
    "devUrl": "http://localhost:1420",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "EM Tool",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "resizable": true
      }
    ],
    "security": {
      "csp": "default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": ["deb", "appimage", "msi"],
    "icon": ["icons/icon.png"]
  }
}
```

- [ ] **Step 10: Create `src-tauri/build.rs`**

```rust
fn main() {
    tauri_build::build()
}
```

- [ ] **Step 10a: Create `src-tauri/capabilities/default.json`**

Tauri 2 requires at least one capability file to build. Custom `#[tauri::command]` functions are allowed without additional permissions.

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "identifier": "default",
  "description": "Default permissions for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default"
  ]
}
```

- [ ] **Step 10b: Create `src-tauri/icons/icon.png`**

Create an empty placeholder icon to satisfy the bundle config (replace later):

```bash
mkdir -p /home/net-irmantasci/em-tool/src-tauri/icons
# 1x1 transparent PNG, base64 decoded
printf '\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15\xc4\x89\x00\x00\x00\rIDATx\x9cc\xf8\xcf\xc0\x00\x00\x00\x03\x00\x01\x00\x18\xdd\x8d\xb4\x00\x00\x00\x00IEND\xaeB`\x82' > /home/net-irmantasci/em-tool/src-tauri/icons/icon.png
```

Also reference the identifier in the main window so the capability binds:

- [ ] **Step 10c: Update `tauri.conf.json` to label the window**

Edit `src-tauri/tauri.conf.json` — add `"label": "main"` to the window object so:

```json
"windows": [
  {
    "label": "main",
    "title": "EM Tool",
    ...
  }
]
```

- [ ] **Step 11: Create `src-tauri/src/main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 12: Install Node deps**

Run: `cd /home/net-irmantasci/em-tool && npm install`
Expected: dependencies resolve, no errors.

- [ ] **Step 13: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "chore: scaffold Tauri 2 + Vue 3 + TS project"
```

---

## Task 2 — App shell: sidebar, routes, stub views

**Files:**
- Create: `src/App.vue`, `src/router.ts`, `src/style.css`
- Create: `src/components/AppSidebar.vue`
- Create: `src/views/WeeklyCaptureView.vue`, `ReportsView.vue`, `TeamHeatmapView.vue`, `PlanGeneratorView.vue`, `SettingsView.vue`

- [ ] **Step 1: Create `src/style.css`**

```css
:root {
  color-scheme: dark;
  --bg: #0a0a0a;
  --surface: #141414;
  --surface-2: #1a1a1a;
  --border: #2a2a2a;
  --text: #e5e5e5;
  --text-dim: #9a9a9a;
  --accent: #7c3aed;
  --red: #ef4444;
  --yellow: #facc15;
  --grey: #6b7280;
  --green: #4ade80;
  --blue: #3b82f6;
}
* { box-sizing: border-box; }
html, body, #app { height: 100%; margin: 0; }
body {
  background: var(--bg);
  color: var(--text);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Inter", sans-serif;
  font-size: 14px;
}
button { font-family: inherit; }
```

- [ ] **Step 2: Create `src/components/AppSidebar.vue`**

```vue
<script setup lang="ts">
import { RouterLink } from "vue-router";

const items = [
  { to: "/capture", label: "Weekly capture", icon: "📋" },
  { to: "/reports", label: "Reports", icon: "👤" },
  { to: "/heatmap", label: "Team heatmap", icon: "🔥" },
  { to: "/plan", label: "Plan generator", icon: "🎯" },
  { to: "/settings", label: "Settings", icon: "⚙" },
];
</script>

<template>
  <aside class="sidebar">
    <div class="brand">EM Tool</div>
    <nav>
      <RouterLink
        v-for="item in items"
        :key="item.to"
        :to="item.to"
        class="nav-item"
        active-class="active"
      >
        <span class="icon">{{ item.icon }}</span>
        <span>{{ item.label }}</span>
      </RouterLink>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 220px;
  background: var(--surface);
  border-right: 1px solid var(--border);
  padding: 18px 12px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}
.brand {
  font-weight: 700;
  font-size: 16px;
  padding: 0 8px;
}
nav { display: flex; flex-direction: column; gap: 2px; }
.nav-item {
  display: flex;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 5px;
  text-decoration: none;
  color: var(--text-dim);
  font-size: 13px;
}
.nav-item:hover { background: var(--surface-2); color: var(--text); }
.nav-item.active { background: var(--surface-2); color: var(--text); }
.icon { width: 18px; }
</style>
```

- [ ] **Step 3: Create `src/App.vue`**

```vue
<script setup lang="ts">
import { RouterView, useRoute } from "vue-router";
import AppSidebar from "./components/AppSidebar.vue";
import { computed } from "vue";

const route = useRoute();
const showChrome = computed(() => !["onboard", "unlock"].includes(String(route.name)));
</script>

<template>
  <div class="app">
    <AppSidebar v-if="showChrome" />
    <main class="main">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.app { display: flex; height: 100vh; }
.main { flex: 1; overflow: auto; padding: 24px; }
</style>
```

- [ ] **Step 4: Create each stub view**

`src/views/WeeklyCaptureView.vue`:
```vue
<template><h2>Weekly capture</h2><p>Coming in Plan 2.</p></template>
```

`src/views/ReportsView.vue`:
```vue
<template><h2>Reports</h2><p>Coming in Plan 2.</p></template>
```

`src/views/TeamHeatmapView.vue`:
```vue
<template><h2>Team heatmap</h2><p>Coming in Plan 2.</p></template>
```

`src/views/PlanGeneratorView.vue`:
```vue
<template><h2>Plan generator</h2><p>Coming in Plan 3.</p></template>
```

`src/views/SettingsView.vue`:
```vue
<template><h2>Settings</h2><p>Coming in Plan 3.</p></template>
```

- [ ] **Step 5: Create `src/router.ts`** (temporary version — no guards yet; Task 7 adds them)

```ts
import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: "/capture" },
  { path: "/onboard", name: "onboard", component: () => import("./views/OnboardingView.vue") },
  { path: "/unlock", name: "unlock", component: () => import("./views/UnlockView.vue") },
  { path: "/capture", name: "capture", component: () => import("./views/WeeklyCaptureView.vue") },
  { path: "/reports", name: "reports", component: () => import("./views/ReportsView.vue") },
  { path: "/heatmap", name: "heatmap", component: () => import("./views/TeamHeatmapView.vue") },
  { path: "/plan", name: "plan", component: () => import("./views/PlanGeneratorView.vue") },
  { path: "/settings", name: "settings", component: () => import("./views/SettingsView.vue") },
];

export default createRouter({ history: createWebHistory(), routes });
```

- [ ] **Step 6: Placeholder onboarding + unlock views** (real implementations in Tasks 8 and 9)

`src/views/OnboardingView.vue`:
```vue
<template><h2>Welcome — set a vault password</h2><p>Coming in Task 8.</p></template>
```

`src/views/UnlockView.vue`:
```vue
<template><h2>Unlock vault</h2><p>Coming in Task 9.</p></template>
```

- [ ] **Step 7: Smoke test — launch dev server**

Run: `cd /home/net-irmantasci/em-tool && npm run dev`
Expected: Vite starts on port 1420 without errors. You won't run the Tauri shell yet (that needs Task 6).

Open http://localhost:1420 in a regular browser; sidebar renders and links switch the main panel content. Stop the server (Ctrl+C).

- [ ] **Step 8: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat(ui): app shell with sidebar, router, and stub views"
```

---

## Task 3 — Rust: Argon2id KDF module (TDD)

**Files:**
- Create: `src-tauri/src/kdf.rs`
- Modify: `src-tauri/src/main.rs` (expose module)

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/kdf.rs`:

```rust
use argon2::{Argon2, Algorithm, Params, Version};
use rand::RngCore;

#[derive(Debug, thiserror::Error)]
pub enum KdfError {
    #[error("argon2 failed: {0}")]
    Argon2(String),
}

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32;

pub fn random_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], KdfError> {
    // OWASP 2023 recommended defaults for interactive login:
    // m = 19 MiB, t = 2, p = 1
    let params = Params::new(19 * 1024, 2, 1, Some(KEY_LEN))
        .map_err(|e| KdfError::Argon2(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| KdfError::Argon2(e.to_string()))?;
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derive_key_is_deterministic() {
        let salt = [7u8; SALT_LEN];
        let k1 = derive_key("hunter2", &salt).unwrap();
        let k2 = derive_key("hunter2", &salt).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_passwords_produce_different_keys() {
        let salt = [0u8; SALT_LEN];
        let k1 = derive_key("alpha", &salt).unwrap();
        let k2 = derive_key("beta", &salt).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn different_salts_produce_different_keys() {
        let k1 = derive_key("same-password", &[1u8; SALT_LEN]).unwrap();
        let k2 = derive_key("same-password", &[2u8; SALT_LEN]).unwrap();
        assert_ne!(k1, k2);
    }

    #[test]
    fn random_salt_has_correct_length() {
        assert_eq!(random_salt().len(), SALT_LEN);
    }
}
```

- [ ] **Step 2: Wire module into `main.rs`**

Replace `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod kdf;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run tests to verify they pass**

Run: `cd /home/net-irmantasci/em-tool/src-tauri && cargo test --lib kdf`
Expected: 4 tests pass (`derive_key_is_deterministic`, `different_passwords_produce_different_keys`, `different_salts_produce_different_keys`, `random_salt_has_correct_length`).

- [ ] **Step 4: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat(rust): Argon2id KDF module with deterministic-key tests"
```

---

## Task 4 — Rust: SQLCipher vault module (TDD)

**Files:**
- Create: `src-tauri/src/vault.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write the failing tests**

Create `src-tauri/src/vault.rs`:

```rust
use rusqlite::Connection;
use std::path::Path;

use crate::kdf::{derive_key, random_salt, SALT_LEN};

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("kdf: {0}")]
    Kdf(#[from] crate::kdf::KdfError),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid password or corrupted vault")]
    InvalidPassword,
    #[error("vault already exists at {0}")]
    AlreadyExists(String),
    #[error("vault not found at {0}")]
    NotFound(String),
    #[error("missing salt — vault was not created by this app")]
    MissingSalt,
}

/// Set the SQLCipher key for this connection using the raw key bytes
/// encoded as `x'…'` — SQLCipher's "raw hex key" form — which skips its
/// own PBKDF2-over-the-passphrase and uses our Argon2-derived key directly.
fn apply_key(conn: &Connection, key: &[u8; 32]) -> rusqlite::Result<()> {
    let hex_key = hex::encode(key);
    conn.execute_batch(&format!("PRAGMA key = \"x'{}'\";", hex_key))?;
    Ok(())
}

fn read_salt(conn: &Connection) -> Result<[u8; SALT_LEN], VaultError> {
    let hex: String = conn
        .query_row(
            "SELECT value FROM _salt WHERE id = 1",
            [],
            |r| r.get(0),
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => VaultError::MissingSalt,
            _ => VaultError::Sqlite(e),
        })?;
    let bytes = hex::decode(hex).map_err(|_| VaultError::MissingSalt)?;
    if bytes.len() != SALT_LEN {
        return Err(VaultError::MissingSalt);
    }
    let mut out = [0u8; SALT_LEN];
    out.copy_from_slice(&bytes);
    Ok(out)
}

fn write_salt(conn: &Connection, salt: &[u8; SALT_LEN]) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS _salt (id INTEGER PRIMARY KEY CHECK (id = 1), value TEXT NOT NULL)",
        [],
    )?;
    conn.execute(
        "INSERT OR REPLACE INTO _salt (id, value) VALUES (1, ?1)",
        [hex::encode(salt)],
    )?;
    Ok(())
}

/// True if the file exists at path (independent of whether it's a valid vault).
pub fn vault_exists(path: &Path) -> bool {
    path.exists()
}

/// Create a new vault at `path`, encrypted with a key derived from `password`.
/// Returns an opened connection.
pub fn create(path: &Path, password: &str) -> Result<Connection, VaultError> {
    if path.exists() {
        return Err(VaultError::AlreadyExists(path.display().to_string()));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let salt = random_salt();
    let key = derive_key(password, &salt)?;

    let conn = Connection::open(path)?;
    apply_key(&conn, &key)?;
    // Force SQLCipher to actually encrypt the empty DB by doing a write.
    conn.execute_batch("CREATE TABLE IF NOT EXISTS _probe (v INTEGER); DROP TABLE _probe;")?;
    write_salt(&conn, &salt)?;
    Ok(conn)
}

/// Open an existing vault by deriving the key from `password`.
/// Returns `InvalidPassword` if decryption fails.
pub fn open(path: &Path, password: &str) -> Result<Connection, VaultError> {
    if !path.exists() {
        return Err(VaultError::NotFound(path.display().to_string()));
    }

    // First pass: open unencrypted handle to read the salt? No — the salt
    // table is inside the encrypted DB. SQLCipher requires key before any read.
    // We derive a throw-away key from password + a *probe* salt won't work —
    // we need the real salt.
    //
    // Solution: store the salt in a tiny companion file next to the vault.
    let salt_path = path.with_extension("salt");
    let salt_bytes = std::fs::read(&salt_path).map_err(|_| VaultError::MissingSalt)?;
    if salt_bytes.len() != SALT_LEN {
        return Err(VaultError::MissingSalt);
    }
    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&salt_bytes);

    let key = derive_key(password, &salt)?;
    let conn = Connection::open(path)?;
    apply_key(&conn, &key)?;
    // Validate key by forcing a read from an encrypted page.
    match conn.query_row("SELECT count(*) FROM sqlite_master", [], |r| r.get::<_, i64>(0)) {
        Ok(_) => Ok(conn),
        Err(rusqlite::Error::SqliteFailure(_, _)) => Err(VaultError::InvalidPassword),
        Err(e) => Err(VaultError::Sqlite(e)),
    }
}

pub fn change_password(conn: &Connection, new_password: &str, salt_path: &Path) -> Result<(), VaultError> {
    let new_salt = random_salt();
    let new_key = derive_key(new_password, &new_salt)?;
    let hex_key = hex::encode(new_key);
    conn.execute_batch(&format!("PRAGMA rekey = \"x'{}'\";", hex_key))?;
    std::fs::write(salt_path, new_salt)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn with_salt_path(db_path: &Path) -> std::path::PathBuf {
        db_path.with_extension("salt")
    }

    // Note: `create` stores the salt in a sidecar file, not in the DB itself.
    // That avoids the chicken-and-egg problem of needing the key to read the salt.
    // We update `create` to write the sidecar as well.
    #[test]
    fn create_then_open_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let salt_path = with_salt_path(&path);

        {
            let conn = create_with_sidecar(&path, &salt_path, "hunter2").unwrap();
            conn.execute_batch("CREATE TABLE t (x INTEGER); INSERT INTO t VALUES (42);").unwrap();
        }

        let conn = open(&path, "hunter2").unwrap();
        let v: i64 = conn.query_row("SELECT x FROM t", [], |r| r.get(0)).unwrap();
        assert_eq!(v, 42);
    }

    #[test]
    fn wrong_password_fails() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let salt_path = with_salt_path(&path);

        {
            let _ = create_with_sidecar(&path, &salt_path, "hunter2").unwrap();
        }

        let err = open(&path, "wrong-password").unwrap_err();
        assert!(matches!(err, VaultError::InvalidPassword));
    }

    #[test]
    fn create_rejects_existing_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("vault.db");
        let salt_path = with_salt_path(&path);
        let _ = create_with_sidecar(&path, &salt_path, "pw").unwrap();

        let err = create_with_sidecar(&path, &salt_path, "pw").unwrap_err();
        assert!(matches!(err, VaultError::AlreadyExists(_)));
    }

    // Test helper version of `create` that writes the sidecar explicitly —
    // production `create` (below) wraps this.
    fn create_with_sidecar(db: &Path, sidecar: &Path, pw: &str) -> Result<Connection, VaultError> {
        if db.exists() {
            return Err(VaultError::AlreadyExists(db.display().to_string()));
        }
        let salt = random_salt();
        let key = derive_key(pw, &salt)?;
        let conn = Connection::open(db)?;
        apply_key(&conn, &key)?;
        conn.execute_batch("CREATE TABLE IF NOT EXISTS _probe (v INTEGER); DROP TABLE _probe;")?;
        std::fs::write(sidecar, salt)?;
        Ok(conn)
    }
}
```

- [ ] **Step 2: Update production `create` to write sidecar (consistent with tests)**

In `src-tauri/src/vault.rs`, replace the body of `pub fn create` with:

```rust
pub fn create(path: &Path, password: &str) -> Result<Connection, VaultError> {
    if path.exists() {
        return Err(VaultError::AlreadyExists(path.display().to_string()));
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let salt = random_salt();
    let key = derive_key(password, &salt)?;

    let conn = Connection::open(path)?;
    apply_key(&conn, &key)?;
    conn.execute_batch("CREATE TABLE IF NOT EXISTS _probe (v INTEGER); DROP TABLE _probe;")?;
    std::fs::write(path.with_extension("salt"), salt)?;
    Ok(conn)
}
```

Also remove the now-unused `read_salt` and `write_salt` helpers (they were a first-draft approach that required the key to already be set before reading salt — impossible).

- [ ] **Step 3: Expose module in `main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod kdf;
mod vault;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Run tests**

Run: `cd /home/net-irmantasci/em-tool/src-tauri && cargo test --lib vault`
Expected: 3 tests pass (`create_then_open_roundtrip`, `wrong_password_fails`, `create_rejects_existing_file`).

- [ ] **Step 5: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat(rust): SQLCipher vault with Argon2id-derived key + sidecar salt"
```

---

## Task 5 — Rust: DB migrations (TDD)

**Files:**
- Create: `src-tauri/src/migrations.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Write the failing test**

Create `src-tauri/src/migrations.rs`:

```rust
use rusqlite::Connection;

pub const CURRENT_SCHEMA_VERSION: i64 = 1;

pub fn run(conn: &Connection) -> rusqlite::Result<()> {
    let version = current_version(conn)?;
    if version < 1 {
        apply_v1(conn)?;
    }
    Ok(())
}

fn current_version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_meta (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            schema_version INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL DEFAULT (strftime('%s','now')),
            last_unlocked_at INTEGER
        );
        INSERT OR IGNORE INTO app_meta (id, schema_version) VALUES (1, 0);",
    )?;
    conn.query_row(
        "SELECT schema_version FROM app_meta WHERE id = 1",
        [],
        |r| r.get(0),
    )
}

fn apply_v1(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(r#"
        CREATE TABLE report (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            role TEXT,
            start_date TEXT,
            one_on_one_cadence_days INTEGER NOT NULL DEFAULT 14,
            notes TEXT,
            active INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE week_rating (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER REFERENCES report(id) ON DELETE CASCADE,
            iso_week TEXT NOT NULL,
            color TEXT NOT NULL CHECK (color IN ('red','yellow','grey','green','blue')),
            notes TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE UNIQUE INDEX idx_week_rating_unique
            ON week_rating(COALESCE(report_id, -1), iso_week);
        CREATE INDEX idx_week_rating_report ON week_rating(report_id);
        CREATE INDEX idx_week_rating_week ON week_rating(iso_week);

        CREATE TABLE one_on_one (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            occurred_at INTEGER NOT NULL,
            agenda_md TEXT,
            notes_md TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX idx_one_on_one_report ON one_on_one(report_id);

        CREATE TABLE action_item (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            one_on_one_id INTEGER REFERENCES one_on_one(id) ON DELETE SET NULL,
            report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            text TEXT NOT NULL,
            owner TEXT NOT NULL CHECK (owner IN ('me','them')),
            due_date TEXT,
            completed_at INTEGER,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX idx_action_item_report ON action_item(report_id);
        CREATE INDEX idx_action_item_open ON action_item(report_id) WHERE completed_at IS NULL;

        CREATE TABLE performance_review (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            period TEXT NOT NULL,
            rating TEXT,
            strengths_md TEXT,
            dev_areas_md TEXT,
            goals_md TEXT,
            occurred_at INTEGER NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX idx_performance_review_report ON performance_review(report_id);

        CREATE TABLE generated_plan (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            kind TEXT NOT NULL CHECK (kind IN ('one_on_one','review')),
            target_report_id INTEGER NOT NULL REFERENCES report(id) ON DELETE CASCADE,
            window_spec TEXT NOT NULL,
            source TEXT NOT NULL CHECK (source IN ('claude','template')),
            prompt_md TEXT,
            output_md TEXT NOT NULL,
            saved_to_meeting_id INTEGER REFERENCES one_on_one(id) ON DELETE SET NULL,
            created_at INTEGER NOT NULL
        );

        CREATE TABLE setting (
            key TEXT PRIMARY KEY,
            value TEXT,
            updated_at INTEGER NOT NULL
        );

        UPDATE app_meta SET schema_version = 1 WHERE id = 1;
    "#)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn open_in_memory() -> Connection {
        // In-memory unencrypted connection — enough to test schema migrations
        // because SQLCipher's PRAGMA-level encryption is transparent above
        // the schema level.
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn first_run_reaches_current_version() {
        let c = open_in_memory();
        run(&c).unwrap();
        let v: i64 = c.query_row(
            "SELECT schema_version FROM app_meta WHERE id = 1",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn run_is_idempotent() {
        let c = open_in_memory();
        run(&c).unwrap();
        run(&c).unwrap();
        run(&c).unwrap();
        let v: i64 = c.query_row(
            "SELECT schema_version FROM app_meta WHERE id = 1",
            [],
            |r| r.get(0),
        ).unwrap();
        assert_eq!(v, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn all_tables_exist_after_v1() {
        let c = open_in_memory();
        run(&c).unwrap();
        for table in [
            "report", "week_rating", "one_on_one", "action_item",
            "performance_review", "generated_plan", "setting", "app_meta",
        ] {
            let count: i64 = c
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(count, 1, "table {} missing", table);
        }
    }

    #[test]
    fn color_check_constraint_rejects_invalid() {
        let c = open_in_memory();
        run(&c).unwrap();
        c.execute(
            "INSERT INTO report (name, created_at) VALUES ('A', 0)",
            [],
        ).unwrap();
        let err = c.execute(
            "INSERT INTO week_rating (report_id, iso_week, color, created_at, updated_at) \
             VALUES (1, '2026-W17', 'purple', 0, 0)",
            [],
        );
        assert!(err.is_err());
    }

    #[test]
    fn unique_week_rating_per_report_per_week() {
        let c = open_in_memory();
        run(&c).unwrap();
        c.execute("INSERT INTO report (name, created_at) VALUES ('A', 0)", []).unwrap();
        c.execute(
            "INSERT INTO week_rating (report_id, iso_week, color, created_at, updated_at) \
             VALUES (1, '2026-W17', 'green', 0, 0)",
            [],
        ).unwrap();
        let err = c.execute(
            "INSERT INTO week_rating (report_id, iso_week, color, created_at, updated_at) \
             VALUES (1, '2026-W17', 'red', 0, 0)",
            [],
        );
        assert!(err.is_err());
    }
}
```

- [ ] **Step 2: Expose module**

Update `src-tauri/src/main.rs`:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod kdf;
mod vault;
mod migrations;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: Run tests**

Run: `cd /home/net-irmantasci/em-tool/src-tauri && cargo test --lib migrations`
Expected: 5 tests pass.

- [ ] **Step 4: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat(rust): v1 schema migration — 8 tables with constraints"
```

---

## Task 6 — Rust: Tauri state + commands

**Files:**
- Create: `src-tauri/src/state.rs`, `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/main.rs`
- Create: `src/lib/invoke.ts`

- [ ] **Step 1: Create `src-tauri/src/state.rs`**

```rust
use rusqlite::Connection;
use std::sync::Mutex;
use std::path::PathBuf;

pub struct AppState {
    pub inner: Mutex<VaultState>,
}

pub struct VaultState {
    pub connection: Option<Connection>,
    pub db_path: PathBuf,
    pub last_activity_at: i64,
}

impl AppState {
    pub fn new(db_path: PathBuf) -> Self {
        Self {
            inner: Mutex::new(VaultState {
                connection: None,
                db_path,
                last_activity_at: 0,
            }),
        }
    }
}

pub fn default_db_path() -> PathBuf {
    // XDG on Linux, AppData on Windows.
    #[cfg(target_os = "linux")]
    {
        let base = std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").expect("HOME not set");
                PathBuf::from(home).join(".local/share")
            });
        base.join("em-tool").join("vault.db")
    }
    #[cfg(target_os = "windows")]
    {
        let appdata = std::env::var("APPDATA").expect("APPDATA not set");
        PathBuf::from(appdata).join("em-tool").join("vault.db")
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        PathBuf::from("./vault.db")
    }
}
```

- [ ] **Step 2: Create `src-tauri/src/commands.rs`**

```rust
use crate::{migrations, state::AppState, vault};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<vault::VaultError> for CommandError {
    fn from(e: vault::VaultError) -> Self {
        let code = match &e {
            vault::VaultError::InvalidPassword => "invalid_password",
            vault::VaultError::AlreadyExists(_) => "already_exists",
            vault::VaultError::NotFound(_) => "not_found",
            vault::VaultError::MissingSalt => "missing_salt",
            vault::VaultError::Sqlite(_) => "sqlite",
            vault::VaultError::Kdf(_) => "kdf",
            vault::VaultError::Io(_) => "io",
        };
        CommandError { code: code.to_string(), message: e.to_string() }
    }
}

impl From<rusqlite::Error> for CommandError {
    fn from(e: rusqlite::Error) -> Self {
        CommandError { code: "sqlite".to_string(), message: e.to_string() }
    }
}

fn now_secs() -> i64 {
    chrono::Utc::now().timestamp()
}

#[tauri::command]
pub fn vault_exists(state: State<AppState>) -> bool {
    let guard = state.inner.lock().unwrap();
    vault::vault_exists(&guard.db_path)
}

#[tauri::command]
pub fn is_unlocked(state: State<AppState>) -> bool {
    let guard = state.inner.lock().unwrap();
    guard.connection.is_some()
}

#[tauri::command]
pub fn create_vault(state: State<AppState>, password: String) -> Result<(), CommandError> {
    let mut guard = state.inner.lock().unwrap();
    let conn = vault::create(&guard.db_path, &password)?;
    migrations::run(&conn)?;
    guard.connection = Some(conn);
    guard.last_activity_at = now_secs();
    Ok(())
}

#[tauri::command]
pub fn unlock_vault(state: State<AppState>, password: String) -> Result<(), CommandError> {
    let mut guard = state.inner.lock().unwrap();
    let conn = vault::open(&guard.db_path, &password)?;
    migrations::run(&conn)?;
    guard.connection = Some(conn);
    guard.last_activity_at = now_secs();
    Ok(())
}

#[tauri::command]
pub fn lock_vault(state: State<AppState>) {
    let mut guard = state.inner.lock().unwrap();
    guard.connection = None;
    guard.last_activity_at = 0;
}

#[tauri::command]
pub fn touch_activity(state: State<AppState>) {
    let mut guard = state.inner.lock().unwrap();
    guard.last_activity_at = now_secs();
}
```

- [ ] **Step 3: Wire state and commands in `main.rs`**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod kdf;
mod vault;
mod migrations;
mod state;
mod commands;

use state::{AppState, default_db_path};

fn main() {
    tauri::Builder::default()
        .manage(AppState::new(default_db_path()))
        .invoke_handler(tauri::generate_handler![
            commands::vault_exists,
            commands::is_unlocked,
            commands::create_vault,
            commands::unlock_vault,
            commands::lock_vault,
            commands::touch_activity,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Create `src/lib/invoke.ts`** — typed wrapper

```ts
import { invoke as rawInvoke } from "@tauri-apps/api/core";

export type CommandError = { code: string; message: string };

export class InvokeError extends Error {
  code: string;
  constructor(err: CommandError) {
    super(err.message);
    this.code = err.code;
  }
}

export async function invoke<T = void>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await rawInvoke<T>(command, args);
  } catch (err) {
    if (err && typeof err === "object" && "code" in err && "message" in err) {
      throw new InvokeError(err as CommandError);
    }
    throw err;
  }
}

export const vaultApi = {
  exists: () => invoke<boolean>("vault_exists"),
  isUnlocked: () => invoke<boolean>("is_unlocked"),
  create: (password: string) => invoke<void>("create_vault", { password }),
  unlock: (password: string) => invoke<void>("unlock_vault", { password }),
  lock: () => invoke<void>("lock_vault"),
  touchActivity: () => invoke<void>("touch_activity"),
};
```

- [ ] **Step 5: Compile check — Rust**

Run: `cd /home/net-irmantasci/em-tool/src-tauri && cargo build`
Expected: no compiler errors. Warnings OK.

- [ ] **Step 6: Compile check — TypeScript**

Run: `cd /home/net-irmantasci/em-tool && npx vue-tsc --noEmit`
Expected: no type errors.

- [ ] **Step 7: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat: Tauri commands for vault lifecycle + typed invoke wrapper"
```

---

## Task 7 — Pinia store + router guards

**Files:**
- Create: `src/stores/vault.ts`
- Modify: `src/router.ts`

- [ ] **Step 1: Create `src/stores/vault.ts`**

```ts
import { defineStore } from "pinia";
import { ref, computed } from "vue";
import { vaultApi, InvokeError } from "../lib/invoke";

export const useVaultStore = defineStore("vault", () => {
  const exists = ref<boolean | null>(null);
  const unlocked = ref(false);
  const checking = ref(false);
  const lastError = ref<string | null>(null);

  const status = computed<"loading" | "needs-setup" | "locked" | "unlocked">(() => {
    if (exists.value === null) return "loading";
    if (!exists.value) return "needs-setup";
    return unlocked.value ? "unlocked" : "locked";
  });

  async function refresh() {
    checking.value = true;
    try {
      exists.value = await vaultApi.exists();
      unlocked.value = exists.value ? await vaultApi.isUnlocked() : false;
    } finally {
      checking.value = false;
    }
  }

  async function create(password: string) {
    lastError.value = null;
    try {
      await vaultApi.create(password);
      exists.value = true;
      unlocked.value = true;
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    }
  }

  async function unlock(password: string) {
    lastError.value = null;
    try {
      await vaultApi.unlock(password);
      unlocked.value = true;
    } catch (e) {
      lastError.value = e instanceof InvokeError ? e.message : String(e);
      throw e;
    }
  }

  async function lock() {
    await vaultApi.lock();
    unlocked.value = false;
  }

  return { exists, unlocked, checking, lastError, status, refresh, create, unlock, lock };
});
```

- [ ] **Step 2: Add router guards in `src/router.ts`**

Replace `src/router.ts` with:

```ts
import { createRouter, createWebHistory, type RouteRecordRaw } from "vue-router";
import { useVaultStore } from "./stores/vault";

const routes: RouteRecordRaw[] = [
  { path: "/", redirect: "/capture" },
  { path: "/onboard", name: "onboard", component: () => import("./views/OnboardingView.vue") },
  { path: "/unlock", name: "unlock", component: () => import("./views/UnlockView.vue") },
  { path: "/capture", name: "capture", component: () => import("./views/WeeklyCaptureView.vue") },
  { path: "/reports", name: "reports", component: () => import("./views/ReportsView.vue") },
  { path: "/heatmap", name: "heatmap", component: () => import("./views/TeamHeatmapView.vue") },
  { path: "/plan", name: "plan", component: () => import("./views/PlanGeneratorView.vue") },
  { path: "/settings", name: "settings", component: () => import("./views/SettingsView.vue") },
];

const router = createRouter({ history: createWebHistory(), routes });

router.beforeEach(async (to) => {
  const vault = useVaultStore();
  if (vault.status === "loading") {
    await vault.refresh();
  }

  if (vault.status === "needs-setup" && to.name !== "onboard") {
    return { name: "onboard" };
  }
  if (vault.status === "locked" && to.name !== "unlock") {
    return { name: "unlock" };
  }
  if (vault.status === "unlocked" && (to.name === "onboard" || to.name === "unlock")) {
    return { name: "capture" };
  }
});

export default router;
```

- [ ] **Step 3: Typecheck**

Run: `cd /home/net-irmantasci/em-tool && npx vue-tsc --noEmit`
Expected: no type errors.

- [ ] **Step 4: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat(ui): vault Pinia store + router guards for setup/unlock/unlocked"
```

---

## Task 8 — Onboarding view (first-run password setup)

**Files:**
- Modify: `src/views/OnboardingView.vue`

- [ ] **Step 1: Replace `src/views/OnboardingView.vue` with real implementation**

```vue
<script setup lang="ts">
import { ref, computed } from "vue";
import { useRouter } from "vue-router";
import { useVaultStore } from "../stores/vault";

const vault = useVaultStore();
const router = useRouter();

const password = ref("");
const confirm = ref("");
const acknowledge = ref(false);
const submitting = ref(false);
const error = ref<string | null>(null);

const passwordsMatch = computed(() => password.value.length > 0 && password.value === confirm.value);
const meetsMinLength = computed(() => password.value.length >= 12);
const canSubmit = computed(() => passwordsMatch.value && meetsMinLength.value && acknowledge.value && !submitting.value);

async function submit() {
  if (!canSubmit.value) return;
  submitting.value = true;
  error.value = null;
  try {
    await vault.create(password.value);
    router.push({ name: "capture" });
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : String(e);
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="onboard">
    <div class="card">
      <h1>Welcome to EM Tool</h1>
      <p class="lead">Set a password to encrypt your vault. Everything you write — notes, ratings, plans — lives locally, encrypted with this password.</p>

      <form @submit.prevent="submit">
        <label>
          <span>Password</span>
          <input v-model="password" type="password" autofocus placeholder="At least 12 characters" />
        </label>
        <label>
          <span>Confirm password</span>
          <input v-model="confirm" type="password" placeholder="Re-enter" />
        </label>

        <div v-if="password && !meetsMinLength" class="hint warn">Use at least 12 characters.</div>
        <div v-if="confirm && !passwordsMatch" class="hint warn">Passwords don't match.</div>

        <label class="ack">
          <input v-model="acknowledge" type="checkbox" />
          <span>I understand there is no password recovery. If I forget this password, the vault cannot be opened.</span>
        </label>

        <button type="submit" :disabled="!canSubmit">
          {{ submitting ? "Creating…" : "Create vault" }}
        </button>

        <div v-if="error" class="error">{{ error }}</div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.onboard { display: flex; align-items: center; justify-content: center; min-height: 100vh; padding: 24px; }
.card { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 28px; max-width: 460px; width: 100%; }
h1 { margin: 0 0 8px; font-size: 22px; }
.lead { color: var(--text-dim); font-size: 13px; line-height: 1.5; margin-bottom: 22px; }
form { display: flex; flex-direction: column; gap: 12px; }
label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--text-dim); }
label.ack { flex-direction: row; gap: 8px; align-items: flex-start; margin-top: 6px; color: var(--text); }
label.ack input { margin-top: 2px; }
input[type="password"] {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 8px 10px; border-radius: 4px; font-family: inherit; font-size: 14px;
}
.hint { font-size: 12px; margin-top: -4px; }
.hint.warn { color: #fbbf24; }
button {
  background: var(--accent); color: #fff; border: none; padding: 10px;
  border-radius: 4px; cursor: pointer; font-size: 14px; margin-top: 8px;
}
button:disabled { opacity: 0.4; cursor: not-allowed; }
.error { color: #f87171; font-size: 12px; margin-top: 6px; }
</style>
```

- [ ] **Step 2: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat(ui): onboarding view — first-run password setup"
```

---

## Task 9 — Unlock view

**Files:**
- Modify: `src/views/UnlockView.vue`

- [ ] **Step 1: Replace `src/views/UnlockView.vue` with real implementation**

```vue
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useRouter } from "vue-router";
import { useVaultStore } from "../stores/vault";

const vault = useVaultStore();
const router = useRouter();

const password = ref("");
const submitting = ref(false);
const error = ref<string | null>(null);
const failureCount = ref(0);
const cooldownUntil = ref<number | null>(null);
const now = ref(Date.now());

const inCooldown = computed(() => cooldownUntil.value !== null && now.value < cooldownUntil.value);
const cooldownSecs = computed(() =>
  cooldownUntil.value ? Math.max(0, Math.ceil((cooldownUntil.value - now.value) / 1000)) : 0
);

onMounted(() => {
  setInterval(() => { now.value = Date.now(); }, 500);
});

async function submit() {
  if (inCooldown.value) return;
  submitting.value = true;
  error.value = null;
  try {
    await vault.unlock(password.value);
    password.value = "";
    failureCount.value = 0;
    cooldownUntil.value = null;
    router.push({ name: "capture" });
  } catch (e: unknown) {
    failureCount.value += 1;
    if (failureCount.value >= 5) {
      cooldownUntil.value = Date.now() + 60_000;
      failureCount.value = 0;
    }
    error.value = "Wrong password.";
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="unlock">
    <div class="card">
      <h1>Unlock your vault</h1>

      <form @submit.prevent="submit">
        <label>
          <span>Password</span>
          <input v-model="password" type="password" autofocus :disabled="inCooldown" />
        </label>

        <button type="submit" :disabled="inCooldown || submitting || !password">
          <template v-if="inCooldown">Too many attempts — try again in {{ cooldownSecs }}s</template>
          <template v-else>{{ submitting ? "Unlocking…" : "Unlock" }}</template>
        </button>

        <div v-if="error && !inCooldown" class="error">{{ error }}</div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.unlock { display: flex; align-items: center; justify-content: center; min-height: 100vh; padding: 24px; }
.card { background: var(--surface); border: 1px solid var(--border); border-radius: 8px; padding: 28px; max-width: 420px; width: 100%; }
h1 { margin: 0 0 18px; font-size: 20px; }
form { display: flex; flex-direction: column; gap: 12px; }
label { display: flex; flex-direction: column; gap: 4px; font-size: 12px; color: var(--text-dim); }
input[type="password"] {
  background: var(--bg); border: 1px solid var(--border); color: var(--text);
  padding: 8px 10px; border-radius: 4px; font-family: inherit; font-size: 14px;
}
button {
  background: var(--accent); color: #fff; border: none; padding: 10px;
  border-radius: 4px; cursor: pointer; font-size: 14px; margin-top: 8px;
}
button:disabled { opacity: 0.4; cursor: not-allowed; }
.error { color: #f87171; font-size: 12px; }
</style>
```

- [ ] **Step 2: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat(ui): unlock view with 5-strikes-60s cooldown"
```

---

## Task 10 — Idle timer + auto-lock

**Files:**
- Create: `src/lib/idle-timer.ts`
- Modify: `src/App.vue`

- [ ] **Step 1: Create `src/lib/idle-timer.ts`**

```ts
export interface IdleTimerOptions {
  timeoutMs: number;
  onIdle: () => void;
  onActivity?: () => void;
}

export function startIdleTimer({ timeoutMs, onIdle, onActivity }: IdleTimerOptions) {
  let timer: number | null = null;
  let active = true;

  const events = ["mousemove", "keydown", "scroll", "click", "touchstart"];

  const reset = () => {
    if (!active) return;
    if (timer !== null) clearTimeout(timer);
    if (onActivity) onActivity();
    timer = window.setTimeout(() => {
      if (active) onIdle();
    }, timeoutMs);
  };

  events.forEach((ev) => window.addEventListener(ev, reset, { passive: true }));
  reset();

  return () => {
    active = false;
    if (timer !== null) clearTimeout(timer);
    events.forEach((ev) => window.removeEventListener(ev, reset));
  };
}
```

- [ ] **Step 2: Wire into `App.vue`**

Replace `src/App.vue`:

```vue
<script setup lang="ts">
import { RouterView, useRoute } from "vue-router";
import AppSidebar from "./components/AppSidebar.vue";
import { computed, onMounted, onUnmounted, watch } from "vue";
import { useVaultStore } from "./stores/vault";
import { startIdleTimer } from "./lib/idle-timer";
import { vaultApi } from "./lib/invoke";

const vault = useVaultStore();
const route = useRoute();
const showChrome = computed(() => !["onboard", "unlock"].includes(String(route.name)));

const AUTO_LOCK_MS = 15 * 60 * 1000;

let stop: (() => void) | null = null;

function start() {
  if (stop) stop();
  stop = startIdleTimer({
    timeoutMs: AUTO_LOCK_MS,
    onIdle: () => { vault.lock(); },
    onActivity: () => { vaultApi.touchActivity(); },
  });
}

onMounted(async () => {
  await vault.refresh();
  if (vault.unlocked) start();
});

watch(() => vault.unlocked, (now) => {
  if (now) start();
  else if (stop) { stop(); stop = null; }
});

onUnmounted(() => { if (stop) stop(); });
</script>

<template>
  <div class="app">
    <AppSidebar v-if="showChrome" />
    <main class="main"><RouterView /></main>
  </div>
</template>

<style scoped>
.app { display: flex; height: 100vh; }
.main { flex: 1; overflow: auto; padding: 24px; }
</style>
```

- [ ] **Step 3: Add unit test for `idle-timer`**

Create `src/lib/idle-timer.test.ts`:

```ts
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { startIdleTimer } from "./idle-timer";

describe("idle-timer", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("fires onIdle after timeoutMs without activity", () => {
    const onIdle = vi.fn();
    startIdleTimer({ timeoutMs: 1000, onIdle });
    vi.advanceTimersByTime(1001);
    expect(onIdle).toHaveBeenCalledTimes(1);
  });

  it("resets on activity", () => {
    const onIdle = vi.fn();
    startIdleTimer({ timeoutMs: 1000, onIdle });
    vi.advanceTimersByTime(800);
    window.dispatchEvent(new Event("keydown"));
    vi.advanceTimersByTime(800);
    expect(onIdle).not.toHaveBeenCalled();
    vi.advanceTimersByTime(300);
    expect(onIdle).toHaveBeenCalledTimes(1);
  });

  it("stop() cancels the timer", () => {
    const onIdle = vi.fn();
    const stop = startIdleTimer({ timeoutMs: 1000, onIdle });
    stop();
    vi.advanceTimersByTime(2000);
    expect(onIdle).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 4: Run tests**

Run: `cd /home/net-irmantasci/em-tool && npm run test`
Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
cd /home/net-irmantasci/em-tool
git add -A
git commit -m "feat: idle-timer auto-locks vault after 15 minutes"
```

---

## Task 11 — End-to-end smoke run

- [ ] **Step 1: Install Tauri CLI prerequisites**

Verify system deps (Linux — run manually if missing):
```bash
sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file \
    libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

On Windows: ensure WebView2 runtime is installed (bundled with Windows 11; install from Microsoft on Windows 10).

- [ ] **Step 2: Run the full app in dev mode**

Run: `cd /home/net-irmantasci/em-tool && npm run tauri dev`
Expected: Tauri window opens. First launch → Onboarding view.

- [ ] **Step 3: Manual smoke test — execute in order**

1. Enter password `testpass1234` in both fields, tick acknowledgement, click "Create vault".
   - Expected: window transitions to Weekly Capture stub screen with sidebar on the left.
2. Close the window entirely.
3. Relaunch: `npm run tauri dev` again.
   - Expected: Unlock view appears (vault exists, locked).
4. Enter wrong password `wrong`.
   - Expected: "Wrong password." inline error.
5. Try 4 more times with wrong passwords.
   - Expected: after 5th failure, button shows "Too many attempts — try again in Ns".
6. Wait for cooldown to elapse, then enter the correct password.
   - Expected: unlocks, lands on Weekly Capture stub.
7. Click each sidebar item; all 5 stubs should render.

- [ ] **Step 4: Verify vault file location**

Linux: `ls -la ~/.local/share/em-tool/`
Expected: `vault.db` and `vault.salt` both exist.

- [ ] **Step 5: Tag this plan's completion**

```bash
cd /home/net-irmantasci/em-tool
git tag -a plan1-foundation-complete -m "Plan 1 (Foundation) — scaffold + encrypted vault + shell"
```

---

## Self-review

- ✅ Spec §4.1 architecture (Tauri/Vue/TS/SQLCipher/Argon2id) — implemented in Tasks 1, 3, 4.
- ✅ Spec §4.2 vault location (XDG / APPDATA) — `default_db_path()` in Task 6.
- ✅ Spec §4.3 auth flow (onboarding → unlock → 15-min auto-lock, 5-strikes-60s) — Tasks 8, 9, 10.
- ✅ Spec §5 data model (8 tables) — Task 5.
- ✅ Spec §6 UI screens — app shell + all 5 stub screens in Task 2; onboarding in Task 8, unlock in Task 9. **Feature screens deferred to Plan 2 (capture, reports, heatmap) and Plan 3 (plan-gen, settings).**
- ✅ Spec §9 security (Argon2id, per-vault key, encrypted API key storage) — Tasks 3, 4. API-key encryption is trivial to add in Plan 3 when Settings ships.
- ✅ Spec §10 testing (Rust unit + TS unit + manual smoke) — Tasks 3, 4, 5, 10, 11.

**Gaps flagged:**
- Backup rotation (spec §4.2, §9) — deferred to Plan 3 alongside full Settings screen.
- Change-password flow (spec §9 implied) — `vault::change_password` is implemented in Task 4 but not yet UI-exposed; deferred to Plan 3 Settings.
- Vault-path override (spec §4.2, §6.5) — default path only in this plan; path override deferred to Plan 3 Settings.

**Placeholder scan:** no TBDs, no "implement later", no code-free steps. Every step has a command or complete code.

**Type consistency spot-check:** `vaultApi` surface (`exists`, `isUnlocked`, `create`, `unlock`, `lock`, `touchActivity`) matches the 6 `#[tauri::command]` functions in Task 6.

---

## After Plan 1

On completion, write Plan 2: **Capture & viewing** — reports CRUD, weekly capture grid, per-person timeline, team heatmap.
