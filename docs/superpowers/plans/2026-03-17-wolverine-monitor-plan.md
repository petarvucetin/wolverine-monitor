# Wolverine Monitor Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Tauri 2 desktop application that monitors Wolverine.FX messages in PostgreSQL in real-time, with dashboards, dead letter management, and cluster health monitoring.

**Architecture:** Tauri 2 Rust backend connects to one or more PostgreSQL databases via `tokio-postgres`, subscribes to LISTEN/NOTIFY for real-time envelope changes, and pushes events to a Svelte 5 frontend. The frontend uses reactive stores updated via Tauri events — no polling from the UI layer.

**Tech Stack:** Tauri 2, Rust, tokio-postgres, deadpool-postgres, Svelte 5, TypeScript, Tailwind CSS 4, layerchart

**Spec:** `docs/superpowers/specs/2026-03-16-wolverine-monitor-design.md`

---

## File Structure

```
wolverine-monitor/
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   ├── icons/                          # Tauri app icons (generated)
│   └── src/
│       ├── main.rs                     # Tauri entry point (windows_subsystem)
│       ├── lib.rs                      # Plugin registration, command registration, app setup
│       ├── models/
│       │   ├── mod.rs
│       │   ├── envelope.rs             # IncomingEnvelope, OutgoingEnvelope, EnvelopeStatus, EnvelopeFilters
│       │   ├── dead_letter.rs          # DeadLetter, ReplayResult, BulkReplayResult
│       │   ├── node.rs                 # WolverineNode, NodeHealth
│       │   ├── connection.rs           # ConnectionConfig, ConnectionStatus, SslMode
│       │   ├── alert.rs               # AlertRule, AlertRuleKind, Alert
│       │   ├── dashboard.rs            # DashboardStats, ThroughputPoint
│       │   └── notification.rs         # NotifyPayload, NotifyOp
│       ├── connections/
│       │   ├── mod.rs
│       │   └── manager.rs             # ConnectionManager: pool creation, lifecycle, reconnect
│       ├── triggers/
│       │   ├── mod.rs
│       │   └── installer.rs           # TriggerInstaller: SQL generation, install/uninstall/check
│       ├── monitor/
│       │   ├── mod.rs
│       │   └── listener.rs            # NotifyListener: LISTEN loop, event emission, stats polling
│       ├── queries/
│       │   ├── mod.rs
│       │   ├── envelopes.rs           # Incoming/outgoing envelope queries with pagination + filters
│       │   ├── dead_letters.rs         # Dead letter queries + replay logic
│       │   ├── nodes.rs               # Node queries
│       │   └── dashboard.rs           # Aggregate stats queries
│       ├── alerts/
│       │   ├── mod.rs
│       │   └── engine.rs             # AlertEngine: rule matching, alert emission
│       ├── config/
│       │   ├── mod.rs
│       │   ├── settings.rs            # AppSettings, load/save
│       │   └── migrations.rs          # Sequential config migrations v1→v2→...
│       ├── commands/
│       │   ├── mod.rs                 # Re-exports all command functions
│       │   ├── connection_cmds.rs     # add/update/remove/test_connection
│       │   ├── envelope_cmds.rs       # get_incoming/outgoing_envelopes, get_message_detail
│       │   ├── dead_letter_cmds.rs    # get_dead_letters, replay_dead_letter, replay_dead_letters_bulk
│       │   ├── node_cmds.rs           # get_nodes
│       │   ├── trigger_cmds.rs        # install/uninstall_triggers
│       │   └── dashboard_cmds.rs      # get_dashboard_stats
│       └── error.rs                   # AppError enum, impl Into<InvokeError>
├── src/
│   ├── app.html                       # HTML shell
│   ├── app.css                        # Tailwind directives + dark theme globals
│   ├── App.svelte                     # Root component with sidebar + route switching
│   ├── lib/
│   │   ├── types.ts                   # TypeScript types mirroring Rust models
│   │   ├── tauri.ts                   # Typed wrappers around invoke() and listen()
│   │   ├── format.ts                  # Date formatting, body decoding, byte display utils
│   │   ├── stores/
│   │   │   ├── router.ts             # currentRoute writable store
│   │   │   ├── connections.ts         # Connection list + active connection + status
│   │   │   ├── dashboard.ts           # Stats, throughput, recent messages per connection
│   │   │   ├── deadLetters.ts         # Dead letter list + filters per connection
│   │   │   ├── nodes.ts              # Node list per connection
│   │   │   └── toasts.ts             # Toast notification queue
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   │   ├── Sidebar.svelte     # Navigation sidebar with view links + connection selector
│   │   │   │   └── ToastContainer.svelte  # Toast notification overlay
│   │   │   ├── dashboard/
│   │   │   │   ├── CounterCard.svelte     # Single metric card with sparkline
│   │   │   │   ├── ThroughputChart.svelte # 30-min line chart
│   │   │   │   └── LiveFeed.svelte        # Auto-scrolling recent message table
│   │   │   ├── explorer/
│   │   │   │   ├── EnvelopeTable.svelte   # Paginated envelope table
│   │   │   │   ├── FilterBar.svelte       # Filter controls (table, status, type, date)
│   │   │   │   └── MessageDetail.svelte   # Slide-over panel with decoded body
│   │   │   ├── deadletters/
│   │   │   │   ├── DeadLetterTable.svelte # DLQ table with exception info
│   │   │   │   └── ReplayControls.svelte  # Single + bulk replay buttons
│   │   │   ├── nodes/
│   │   │   │   ├── NodeTable.svelte       # Node list table
│   │   │   │   └── HealthIndicator.svelte # Green/yellow/red dot
│   │   │   └── connections/
│   │   │       ├── ConnectionForm.svelte  # Add/edit connection form
│   │   │       ├── ConnectionList.svelte  # List of saved connections
│   │   │       └── ConnectionStatus.svelte # Connected/disconnected/error badge
│   │   └── views/
│   │       ├── Dashboard.svelte
│   │       ├── Explorer.svelte
│   │       ├── DeadLetters.svelte
│   │       ├── Nodes.svelte
│   │       └── Connections.svelte
│   └── tests/
│       ├── setup.ts                   # Tauri mock setup for vitest
│       ├── stores/
│       │   ├── router.test.ts
│       │   ├── connections.test.ts
│       │   └── dashboard.test.ts
│       └── components/
│           ├── HealthIndicator.test.ts
│           └── CounterCard.test.ts
├── package.json
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
└── docs/superpowers/
    ├── specs/2026-03-16-wolverine-monitor-design.md
    └── plans/2026-03-17-wolverine-monitor-plan.md
```

---

## Chunk 1: Project Scaffolding + Core Rust Types

This chunk sets up the Tauri 2 + Svelte 5 project, installs all dependencies, configures Tailwind dark theme, and defines the Rust domain models + TypeScript types. At the end, the app compiles and opens a dark-themed window with a placeholder sidebar.

### Task 1: Initialize Tauri 2 + Svelte 5 project

**Files:**
- Create: entire project scaffolding via `create-tauri-app`
- Modify: `package.json`, `Cargo.toml`, `tauri.conf.json`

- [ ] **Step 1: Scaffold the project**

```bash
cd /mnt/d/ai/projects
npm create tauri-app@latest wolverine-monitor -- --template svelte-ts --manager npm
```

Select Tauri 2 when prompted. This creates the base structure with `src/` (Svelte) and `src-tauri/` (Rust).

- [ ] **Step 2: Install frontend dependencies**

```bash
cd /mnt/d/ai/projects/wolverine-monitor
npm install
npm install -D tailwindcss@4 @tailwindcss/vite
npm install layerchart @tauri-apps/plugin-notification @tauri-apps/plugin-store
```

- [ ] **Step 3: Install Rust dependencies**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
tokio-postgres = { version = "0.7", features = ["with-uuid-0_8", "with-chrono-0_4", "with-serde_json-1"] }
deadpool-postgres = "0.14"
postgres-types = { version = "0.2", features = ["derive"] }
uuid = { version = "0.8", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
tauri-plugin-notification = "2"
tauri-plugin-store = "2"
tauri-plugin-log = "2"
base64 = "0.22"
thiserror = "2"
```

Add to `[dev-dependencies]`:

```toml
testcontainers = "0.23"
testcontainers-modules = { version = "0.11", features = ["postgres"] }
```

- [ ] **Step 4: Configure Tailwind 4 with dark theme**

Replace `src/app.css`:

```css
@import "tailwindcss";

@theme {
  --color-surface: #0f1117;
  --color-surface-raised: #1a1d27;
  --color-surface-overlay: #252833;
  --color-border: #2e3140;
  --color-text-primary: #e2e4e9;
  --color-text-secondary: #8b8fa3;
  --color-status-handled: #22c55e;
  --color-status-incoming: #3b82f6;
  --color-status-outgoing: #f97316;
  --color-status-error: #ef4444;
}

body {
  background-color: var(--color-surface);
  color: var(--color-text-primary);
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
}
```

Add the Tailwind vite plugin to `vite.config.ts`:

```typescript
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";

export default defineConfig({
  plugins: [svelte(), tailwindcss()],
});
```

- [ ] **Step 5: Configure tauri.conf.json**

Update `src-tauri/tauri.conf.json` to set the window title, default size, and dark theme:

```json
{
  "app": {
    "windows": [
      {
        "title": "Wolverine Monitor",
        "width": 1280,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "decorations": true,
        "transparent": false
      }
    ]
  }
}
```

Update `src-tauri/capabilities/default.json` to include notification and store permissions:

```json
{
  "identifier": "default",
  "description": "Default capabilities",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "notification:default",
    "notification:allow-request-permission",
    "notification:allow-send-notification",
    "store:default"
  ]
}
```

- [ ] **Step 6: Verify the app compiles and opens**

```bash
cd /mnt/d/ai/projects/wolverine-monitor
npm run tauri dev
```

Expected: A window titled "Wolverine Monitor" opens with the default Svelte template on a dark background.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: scaffold Tauri 2 + Svelte 5 project with Tailwind dark theme"
```

---

### Task 2: Define Rust domain models

**Files:**
- Create: `src-tauri/src/models/mod.rs`
- Create: `src-tauri/src/models/envelope.rs`
- Create: `src-tauri/src/models/dead_letter.rs`
- Create: `src-tauri/src/models/node.rs`
- Create: `src-tauri/src/models/connection.rs`
- Create: `src-tauri/src/models/alert.rs`
- Create: `src-tauri/src/models/dashboard.rs`
- Create: `src-tauri/src/models/notification.rs`
- Create: `src-tauri/src/error.rs`

- [ ] **Step 1: Create the error type**

Create `src-tauri/src/error.rs`:

```rust
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] tokio_postgres::Error),

    #[error("Pool error: {0}")]
    Pool(#[from] deadpool_postgres::PoolError),

    #[error("Connection not found: {0}")]
    ConnectionNotFound(String),

    #[error("Message not replayable: {0}")]
    NotReplayable(uuid::Uuid),

    #[error("Trigger installation failed: {0}")]
    TriggerInstallFailed(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Timeout: query exceeded {0}s limit")]
    Timeout(u64),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
```

- [ ] **Step 2: Create envelope models**

Create `src-tauri/src/models/envelope.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnvelopeStatus {
    Incoming,
    Scheduled,
    Handled,
}

impl EnvelopeStatus {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Incoming" => Some(Self::Incoming),
            "Scheduled" => Some(Self::Scheduled),
            "Handled" => Some(Self::Handled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingEnvelope {
    pub id: Uuid,
    pub status: EnvelopeStatus,
    pub owner_id: i32,
    pub execution_time: Option<DateTime<Utc>>,
    pub attempts: i32,
    pub body: Vec<u8>,
    pub message_type: String,
    pub received_at: Option<String>,
    pub keep_until: Option<DateTime<Utc>>,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutgoingEnvelope {
    pub id: Uuid,
    pub owner_id: i32,
    pub destination: String,
    pub deliver_by: Option<DateTime<Utc>>,
    pub body: Vec<u8>,
    pub attempts: i32,
    pub message_type: String,
    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvelopeFilters {
    pub status: Option<String>,
    pub message_type: Option<String>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
```

- [ ] **Step 3: Create dead letter model**

Create `src-tauri/src/models/dead_letter.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetter {
    pub id: Uuid,
    pub execution_time: Option<DateTime<Utc>>,
    pub body: Vec<u8>,
    pub message_type: String,
    pub received_at: Option<String>,
    pub source: Option<String>,
    pub exception_type: Option<String>,
    pub exception_message: Option<String>,
    pub sent_at: Option<DateTime<Utc>>,
    pub replayable: bool,
    pub expires: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkReplayResult {
    pub succeeded: usize,
    pub failed: usize,
    pub errors: Vec<ReplayError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayError {
    pub id: Uuid,
    pub reason: String,
}
```

- [ ] **Step 4: Create node model**

Create `src-tauri/src/models/node.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WolverineNode {
    pub id: Uuid,
    pub node_number: i32,
    pub description: Option<String>,
    pub uri: Option<String>,
    pub started: Option<DateTime<Utc>>,
    pub health_check: Option<DateTime<Utc>>,
    pub version: Option<String>,
    pub capabilities: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeHealth {
    Healthy,
    Warning,
    Critical,
    Unknown,
}
```

- [ ] **Step 5: Create connection config model**

Create `src-tauri/src/models/connection.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SslMode {
    Disable,
    Prefer,
    Require,
    VerifyCa,
}

impl Default for SslMode {
    fn default() -> Self {
        Self::Prefer
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub schema: String,
    pub username: String,
    pub password: String,
    pub ssl_mode: SslMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
    Reconnecting,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub config: ConnectionConfig,
    pub status: ConnectionStatus,
    pub triggers_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionUpdate {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub database: Option<String>,
    pub schema: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub ssl_mode: Option<SslMode>,
}
```

- [ ] **Step 6: Create alert and dashboard models**

Create `src-tauri/src/models/alert.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertRuleKind {
    AnyDeadLetter,
    DeadLetterMessageType { message_type: String },
    IncomingQueueDepth { threshold: i64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub enabled: bool,
    pub kind: AlertRuleKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub rule_id: String,
    pub connection_id: String,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}
```

Create `src-tauri/src/models/dashboard.rs`:

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DashboardStats {
    pub incoming_count: i64,
    pub incoming_scheduled: i64,
    pub incoming_handled: i64,
    pub outgoing_count: i64,
    pub dead_letter_count: i64,
    pub throughput: Vec<ThroughputPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputPoint {
    pub timestamp: DateTime<Utc>,
    pub incoming: i64,
    pub outgoing: i64,
}
```

- [ ] **Step 7: Create notification payload model**

Create `src-tauri/src/models/notification.rs`:

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotifyOp {
    #[serde(rename = "INSERT")]
    Insert,
    #[serde(rename = "UPDATE")]
    Update,
    #[serde(rename = "DELETE")]
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyPayload {
    pub op: NotifyOp,
    pub id: Uuid,
    pub message_type: String,
}

impl NotifyPayload {
    pub fn parse(json_str: &str) -> Option<Self> {
        serde_json::from_str(json_str).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_insert_payload() {
        let json = r#"{"op":"INSERT","id":"a1b2c3d4-e5f6-7890-abcd-ef1234567890","message_type":"MyApp.Commands.PlaceOrder"}"#;
        let payload = NotifyPayload::parse(json).unwrap();
        assert_eq!(payload.op, NotifyOp::Insert);
        assert_eq!(payload.message_type, "MyApp.Commands.PlaceOrder");
    }

    #[test]
    fn test_parse_delete_payload() {
        let json = r#"{"op":"DELETE","id":"a1b2c3d4-e5f6-7890-abcd-ef1234567890","message_type":"MyApp.Events.OrderShipped"}"#;
        let payload = NotifyPayload::parse(json).unwrap();
        assert_eq!(payload.op, NotifyOp::Delete);
    }

    #[test]
    fn test_parse_malformed_returns_none() {
        assert!(NotifyPayload::parse("not json").is_none());
        assert!(NotifyPayload::parse(r#"{"op":"INVALID"}"#).is_none());
    }
}
```

- [ ] **Step 8: Create models mod.rs**

Create `src-tauri/src/models/mod.rs`:

```rust
pub mod alert;
pub mod connection;
pub mod dashboard;
pub mod dead_letter;
pub mod envelope;
pub mod node;
pub mod notification;
```

- [ ] **Step 9: Wire models into lib.rs**

Update `src-tauri/src/lib.rs` to include the models module:

```rust
mod error;
mod models;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 10: Run tests and verify compilation**

```bash
cd /mnt/d/ai/projects/wolverine-monitor/src-tauri
cargo test
cargo build
```

Expected: All notification payload tests pass. Build succeeds.

- [ ] **Step 11: Commit**

```bash
git add -A
git commit -m "feat: add Rust domain models and error types"
```

---

### Task 3: Define TypeScript types and Tauri bridge utilities

**Files:**
- Create: `src/lib/types.ts`
- Create: `src/lib/tauri.ts`
- Create: `src/lib/format.ts`

- [ ] **Step 1: Create TypeScript types mirroring Rust models**

Create `src/lib/types.ts`:

```typescript
export type EnvelopeStatus = "Incoming" | "Scheduled" | "Handled";

export interface IncomingEnvelope {
  id: string;
  status: EnvelopeStatus;
  owner_id: number;
  execution_time: string | null;
  attempts: number;
  body: number[];
  message_type: string;
  received_at: string | null;
  keep_until: string | null;
  timestamp: string | null;
}

export interface OutgoingEnvelope {
  id: string;
  owner_id: number;
  destination: string;
  deliver_by: string | null;
  body: number[];
  attempts: number;
  message_type: string;
  timestamp: string | null;
}

export interface DeadLetter {
  id: string;
  execution_time: string | null;
  body: number[];
  message_type: string;
  received_at: string | null;
  source: string | null;
  exception_type: string | null;
  exception_message: string | null;
  sent_at: string | null;
  replayable: boolean;
  expires: string | null;
}

export interface WolverineNode {
  id: string;
  node_number: number;
  description: string | null;
  uri: string | null;
  started: string | null;
  health_check: string | null;
  version: string | null;
  capabilities: string | null;
}

export type NodeHealth = "Healthy" | "Warning" | "Critical" | "Unknown";

export type SslMode = "Disable" | "Prefer" | "Require" | "VerifyCa";

export interface ConnectionConfig {
  id: string;
  name: string;
  host: string;
  port: number;
  database: string;
  schema: string;
  username: string;
  password: string;
  ssl_mode: SslMode;
}

export type ConnectionStatus =
  | "Connected"
  | "Disconnected"
  | "Reconnecting"
  | { Error: string };

export interface ConnectionInfo {
  config: ConnectionConfig;
  status: ConnectionStatus;
  triggers_installed: boolean;
}

export interface DashboardStats {
  incoming_count: number;
  incoming_scheduled: number;
  incoming_handled: number;
  outgoing_count: number;
  dead_letter_count: number;
  throughput: ThroughputPoint[];
}

export interface ThroughputPoint {
  timestamp: string;
  incoming: number;
  outgoing: number;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface EnvelopeFilters {
  status?: string;
  message_type?: string;
  date_from?: string;
  date_to?: string;
}

export interface BulkReplayResult {
  succeeded: number;
  failed: number;
  errors: { id: string; reason: string }[];
}

export type NotifyOp = "INSERT" | "UPDATE" | "DELETE";

export interface NotifyEvent {
  connection_id: string;
  table: "incoming" | "outgoing" | "dead_letter";
  op: NotifyOp;
  id: string;
  message_type: string;
}

export type Route = "dashboard" | "explorer" | "deadletters" | "nodes" | "connections";

export interface AlertRule {
  id: string;
  enabled: boolean;
  kind: AlertRuleKind;
}

export type AlertRuleKind =
  | { type: "AnyDeadLetter" }
  | { type: "DeadLetterMessageType"; message_type: string }
  | { type: "IncomingQueueDepth"; threshold: number };
```

- [ ] **Step 2: Create Tauri invoke/listen wrappers**

Create `src/lib/tauri.ts`:

```typescript
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ConnectionConfig,
  ConnectionInfo,
  DashboardStats,
  DeadLetter,
  EnvelopeFilters,
  IncomingEnvelope,
  OutgoingEnvelope,
  WolverineNode,
  PaginatedResult,
  BulkReplayResult,
  NotifyEvent,
  SslMode,
} from "./types";

// Connection commands
export const addConnection = (config: Omit<ConnectionConfig, "id">) =>
  invoke<string>("add_connection", { config });

export const updateConnection = (connectionId: string, updates: Partial<ConnectionConfig>) =>
  invoke<void>("update_connection", { connectionId, updates });

export const removeConnection = (connectionId: string) =>
  invoke<void>("remove_connection", { connectionId });

export const testConnection = (
  host: string, port: number, database: string,
  username: string, password: string, sslMode: SslMode
) =>
  invoke<void>("test_connection", { host, port, database, username, password, sslMode });

// Envelope commands
export const getIncomingEnvelopes = (
  connectionId: string, filters: EnvelopeFilters, page: number, pageSize: number
) =>
  invoke<PaginatedResult<IncomingEnvelope>>("get_incoming_envelopes", {
    connectionId, filters, page, pageSize,
  });

export const getOutgoingEnvelopes = (
  connectionId: string, filters: EnvelopeFilters, page: number, pageSize: number
) =>
  invoke<PaginatedResult<OutgoingEnvelope>>("get_outgoing_envelopes", {
    connectionId, filters, page, pageSize,
  });

// Dead letter commands
export const getDeadLetters = (
  connectionId: string, filters: EnvelopeFilters, page: number, pageSize: number
) =>
  invoke<PaginatedResult<DeadLetter>>("get_dead_letters", {
    connectionId, filters, page, pageSize,
  });

export const replayDeadLetter = (connectionId: string, id: string) =>
  invoke<void>("replay_dead_letter", { connectionId, id });

export const replayDeadLettersBulk = (connectionId: string, ids: string[]) =>
  invoke<BulkReplayResult>("replay_dead_letters_bulk", { connectionId, ids });

// Node commands
export const getNodes = (connectionId: string) =>
  invoke<WolverineNode[]>("get_nodes", { connectionId });

// Dashboard commands
export const getDashboardStats = (connectionId: string) =>
  invoke<DashboardStats>("get_dashboard_stats", { connectionId });

export const getMessageDetail = (connectionId: string, table: string, id: string) =>
  invoke<Record<string, unknown>>("get_message_detail", { connectionId, table, id });

// Trigger commands
export const installTriggers = (connectionId: string) =>
  invoke<void>("install_triggers", { connectionId });

export const uninstallTriggers = (connectionId: string) =>
  invoke<void>("uninstall_triggers", { connectionId });

// Event listeners
export const onEnvelopeChange = (callback: (event: NotifyEvent) => void): Promise<UnlistenFn> =>
  listen<NotifyEvent>("envelope:changed", (e) => callback(e.payload));

export const onConnectionStatus = (
  callback: (event: { connection_id: string; status: string }) => void
): Promise<UnlistenFn> =>
  listen("connection:status", (e) => callback(e.payload as any));

export const onAlert = (
  callback: (event: { connection_id: string; message: string }) => void
): Promise<UnlistenFn> =>
  listen("alert:triggered", (e) => callback(e.payload as any));
```

- [ ] **Step 3: Create format utilities**

Create `src/lib/format.ts`:

```typescript
export function decodeBody(body: number[]): { type: "json"; content: string } | { type: "hex"; content: string; base64: string } {
  try {
    const bytes = new Uint8Array(body);
    const text = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    JSON.parse(text); // validate JSON
    return { type: "json", content: JSON.stringify(JSON.parse(text), null, 2) };
  } catch {
    const bytes = new Uint8Array(body);
    const hex = Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join(" ");
    const base64 = btoa(bytes.reduce((s, b) => s + String.fromCharCode(b), ""));
    return { type: "hex", content: hex, base64 };
  }
}

export function formatRelativeTime(isoDate: string | null): string {
  if (!isoDate) return "—";
  const diff = Date.now() - new Date(isoDate).getTime();
  const seconds = Math.floor(diff / 1000);
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

export function getNodeHealth(
  healthCheck: string | null,
  thresholds = { warning: 30, critical: 120 }
): "Healthy" | "Warning" | "Critical" | "Unknown" {
  if (!healthCheck) return "Unknown";
  const ageSeconds = (Date.now() - new Date(healthCheck).getTime()) / 1000;
  if (ageSeconds < thresholds.warning) return "Healthy";
  if (ageSeconds < thresholds.critical) return "Warning";
  return "Critical";
}

export function shortenMessageType(messageType: string): string {
  const parts = messageType.split(".");
  return parts[parts.length - 1] ?? messageType;
}
```

- [ ] **Step 4: Commit**

```bash
git add src/lib/types.ts src/lib/tauri.ts src/lib/format.ts
git commit -m "feat: add TypeScript types, Tauri bridge wrappers, and format utilities"
```

---

### Task 4: Create app shell with sidebar navigation and route store

**Files:**
- Create: `src/lib/stores/router.ts`
- Create: `src/lib/stores/toasts.ts`
- Create: `src/lib/components/layout/Sidebar.svelte`
- Create: `src/lib/components/layout/ToastContainer.svelte`
- Create: `src/lib/views/Dashboard.svelte` (placeholder)
- Create: `src/lib/views/Explorer.svelte` (placeholder)
- Create: `src/lib/views/DeadLetters.svelte` (placeholder)
- Create: `src/lib/views/Nodes.svelte` (placeholder)
- Create: `src/lib/views/Connections.svelte` (placeholder)
- Modify: `src/App.svelte`

- [ ] **Step 1: Create router store**

Create `src/lib/stores/router.ts`:

```typescript
import { writable } from "svelte/store";
import type { Route } from "../types";

export const currentRoute = writable<Route>("dashboard");

export function navigate(route: Route) {
  currentRoute.set(route);
}
```

- [ ] **Step 2: Create toast store**

Create `src/lib/stores/toasts.ts`:

```typescript
import { writable } from "svelte/store";

export interface Toast {
  id: number;
  message: string;
  type: "info" | "error" | "success" | "warning";
}

let nextId = 0;

function createToastStore() {
  const { subscribe, update } = writable<Toast[]>([]);

  return {
    subscribe,
    add(message: string, type: Toast["type"] = "info") {
      const id = nextId++;
      update((toasts) => [...toasts, { id, message, type }]);
      setTimeout(() => {
        update((toasts) => toasts.filter((t) => t.id !== id));
      }, 5000);
    },
    dismiss(id: number) {
      update((toasts) => toasts.filter((t) => t.id !== id));
    },
  };
}

export const toasts = createToastStore();
```

- [ ] **Step 3: Create Sidebar component**

Create `src/lib/components/layout/Sidebar.svelte`:

```svelte
<script lang="ts">
  import { currentRoute, navigate } from "../../stores/router";
  import type { Route } from "../../types";

  const navItems: { route: Route; label: string; icon: string }[] = [
    { route: "dashboard", label: "Dashboard", icon: "◉" },
    { route: "explorer", label: "Explorer", icon: "⊞" },
    { route: "deadletters", label: "Dead Letters", icon: "⚠" },
    { route: "nodes", label: "Nodes", icon: "⬡" },
    { route: "connections", label: "Connections", icon: "⛁" },
  ];
</script>

<aside class="w-56 h-screen flex flex-col bg-[var(--color-surface-raised)] border-r border-[var(--color-border)]">
  <div class="px-4 py-5 text-lg font-bold tracking-tight">
    Wolverine Monitor
  </div>

  <nav class="flex-1 px-2 space-y-1">
    {#each navItems as item}
      <button
        class="w-full flex items-center gap-3 px-3 py-2 rounded-md text-sm transition-colors
          {$currentRoute === item.route
            ? 'bg-[var(--color-surface-overlay)] text-white'
            : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-overlay)] hover:text-white'}"
        onclick={() => navigate(item.route)}
      >
        <span class="text-base">{item.icon}</span>
        {item.label}
      </button>
    {/each}
  </nav>

  <div class="px-4 py-3 text-xs text-[var(--color-text-secondary)] border-t border-[var(--color-border)]">
    v0.1.0
  </div>
</aside>
```

- [ ] **Step 4: Create ToastContainer component**

Create `src/lib/components/layout/ToastContainer.svelte`:

```svelte
<script lang="ts">
  import { toasts } from "../../stores/toasts";

  const colorMap = {
    info: "border-blue-500",
    error: "border-red-500",
    success: "border-green-500",
    warning: "border-orange-500",
  };
</script>

<div class="fixed top-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
  {#each $toasts as toast (toast.id)}
    <div
      class="bg-[var(--color-surface-raised)] border-l-4 {colorMap[toast.type]} px-4 py-3 rounded shadow-lg text-sm"
      role="alert"
    >
      <div class="flex justify-between items-start gap-2">
        <span>{toast.message}</span>
        <button
          class="text-[var(--color-text-secondary)] hover:text-white"
          onclick={() => toasts.dismiss(toast.id)}
        >
          ✕
        </button>
      </div>
    </div>
  {/each}
</div>
```

- [ ] **Step 5: Create placeholder views**

Create each view file with a placeholder. Example for `src/lib/views/Dashboard.svelte`:

```svelte
<div class="p-6">
  <h1 class="text-xl font-semibold mb-4">Dashboard</h1>
  <p class="text-[var(--color-text-secondary)]">Real-time monitoring coming soon.</p>
</div>
```

Repeat for `Explorer.svelte`, `DeadLetters.svelte`, `Nodes.svelte`, `Connections.svelte` (change the title/description in each).

- [ ] **Step 6: Wire up App.svelte with sidebar + view switching**

Replace `src/App.svelte`:

```svelte
<script lang="ts">
  import Sidebar from "./lib/components/layout/Sidebar.svelte";
  import ToastContainer from "./lib/components/layout/ToastContainer.svelte";
  import Dashboard from "./lib/views/Dashboard.svelte";
  import Explorer from "./lib/views/Explorer.svelte";
  import DeadLetters from "./lib/views/DeadLetters.svelte";
  import Nodes from "./lib/views/Nodes.svelte";
  import Connections from "./lib/views/Connections.svelte";
  import { currentRoute } from "./lib/stores/router";
</script>

<div class="flex h-screen overflow-hidden">
  <Sidebar />

  <main class="flex-1 overflow-y-auto">
    {#if $currentRoute === "dashboard"}
      <Dashboard />
    {:else if $currentRoute === "explorer"}
      <Explorer />
    {:else if $currentRoute === "deadletters"}
      <DeadLetters />
    {:else if $currentRoute === "nodes"}
      <Nodes />
    {:else if $currentRoute === "connections"}
      <Connections />
    {/if}
  </main>
</div>

<ToastContainer />
```

- [ ] **Step 7: Verify the app compiles and displays the shell**

```bash
npm run tauri dev
```

Expected: Dark window with sidebar navigation. Clicking items switches the placeholder content.

- [ ] **Step 8: Commit**

```bash
git add -A
git commit -m "feat: add app shell with sidebar navigation, route store, and toast system"
```

---

## Chunk 2: Connection Management (Backend + Frontend)

This chunk implements the full connection lifecycle: saving/loading connection configs, creating PostgreSQL pools, testing connections, and the Connections View UI. At the end, a user can add a PostgreSQL connection, test it, and see its status.

### Task 5: Implement connection config persistence

**Files:**
- Create: `src-tauri/src/config/mod.rs`
- Create: `src-tauri/src/config/settings.rs`
- Create: `src-tauri/src/config/migrations.rs`

- [ ] **Step 1: Create settings module**

Create `src-tauri/src/config/settings.rs`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub version: u32,
    pub polling_interval_secs: u64,
    pub rolling_window_size: usize,
    pub query_timeout_secs: u64,
    pub node_health_warning_secs: u64,
    pub node_health_critical_secs: u64,
    pub node_poll_interval_secs: u64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            version: 1,
            polling_interval_secs: 5,
            rolling_window_size: 500,
            query_timeout_secs: 10,
            node_health_warning_secs: 30,
            node_health_critical_secs: 120,
            node_poll_interval_secs: 10,
        }
    }
}
```

- [ ] **Step 2: Create migrations module**

Create `src-tauri/src/config/migrations.rs`:

```rust
use serde_json::Value;

/// Apply all migrations from current version to latest.
pub fn migrate(config: &mut Value, current_version: u32, target_version: u32) -> Result<(), String> {
    for version in current_version..target_version {
        match version {
            // Future migrations go here:
            // 1 => migrate_v1_to_v2(config)?,
            _ => {} // No migration needed
        }
    }
    config["version"] = serde_json::json!(target_version);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_noop_same_version() {
        let mut config = serde_json::json!({"version": 1});
        migrate(&mut config, 1, 1).unwrap();
        assert_eq!(config["version"], 1);
    }
}
```

- [ ] **Step 3: Create config mod.rs**

Create `src-tauri/src/config/mod.rs`:

```rust
pub mod migrations;
pub mod settings;
```

- [ ] **Step 4: Run tests**

```bash
cd /mnt/d/ai/projects/wolverine-monitor/src-tauri
cargo test
```

Expected: All tests pass.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/config/
git commit -m "feat: add config settings and migration framework"
```

---

### Task 6: Implement ConnectionManager

**Files:**
- Create: `src-tauri/src/connections/mod.rs`
- Create: `src-tauri/src/connections/manager.rs`

- [ ] **Step 1: Create ConnectionManager**

Create `src-tauri/src/connections/manager.rs`:

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use deadpool_postgres::{Config, Pool, Runtime};
use tokio_postgres::NoTls;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, ConnectionStatus, ConnectionInfo, SslMode};

pub struct ManagedConnection {
    pub config: ConnectionConfig,
    pub pool: Pool,
    pub status: ConnectionStatus,
    pub triggers_installed: bool,
}

pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, ManagedConnection>>>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add(&self, mut config: ConnectionConfig) -> Result<String, AppError> {
        if config.id.is_empty() {
            config.id = Uuid::new_v4().to_string();
        }
        let id = config.id.clone();
        let pool = self.create_pool(&config)?;

        // Test the connection
        let client = pool.get().await?;
        client.simple_query("SELECT 1").await?;
        drop(client);

        let managed = ManagedConnection {
            config,
            pool,
            status: ConnectionStatus::Connected,
            triggers_installed: false,
        };

        self.connections.write().await.insert(id.clone(), managed);
        Ok(id)
    }

    pub async fn remove(&self, connection_id: &str) -> Result<(), AppError> {
        self.connections
            .write()
            .await
            .remove(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;
        Ok(())
    }

    pub async fn get_pool(&self, connection_id: &str) -> Result<Pool, AppError> {
        let conns = self.connections.read().await;
        let managed = conns
            .get(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;
        Ok(managed.pool.clone())
    }

    pub async fn get_schema(&self, connection_id: &str) -> Result<String, AppError> {
        let conns = self.connections.read().await;
        let managed = conns
            .get(connection_id)
            .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;
        Ok(managed.config.schema.clone())
    }

    pub async fn list(&self) -> Vec<ConnectionInfo> {
        let conns = self.connections.read().await;
        conns
            .values()
            .map(|m| ConnectionInfo {
                config: m.config.clone(),
                status: m.status.clone(),
                triggers_installed: m.triggers_installed,
            })
            .collect()
    }

    pub async fn set_triggers_installed(&self, connection_id: &str, installed: bool) {
        let mut conns = self.connections.write().await;
        if let Some(managed) = conns.get_mut(connection_id) {
            managed.triggers_installed = installed;
        }
    }

    pub async fn test_connection(
        host: &str, port: u16, database: &str,
        username: &str, password: &str, _ssl_mode: &SslMode,
    ) -> Result<(), AppError> {
        let mut cfg = Config::new();
        cfg.host = Some(host.to_string());
        cfg.port = Some(port);
        cfg.dbname = Some(database.to_string());
        cfg.user = Some(username.to_string());
        cfg.password = Some(password.to_string());

        let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AppError::Config(e.to_string()))?;

        let client = pool.get().await?;
        client.simple_query("SELECT 1").await?;
        Ok(())
    }

    fn create_pool(&self, config: &ConnectionConfig) -> Result<Pool, AppError> {
        let mut cfg = Config::new();
        cfg.host = Some(config.host.clone());
        cfg.port = Some(config.port);
        cfg.dbname = Some(config.database.clone());
        cfg.user = Some(config.username.clone());
        cfg.password = Some(config.password.clone());

        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .map_err(|e| AppError::Config(e.to_string()))
    }
}
```

Note: SSL/TLS (`NoTls` vs `MakeTlsConnector`) is simplified to `NoTls` initially. Full SSL support with `native-tls` or `rustls` can be added as an enhancement — the `SslMode` enum is already in the model.

- [ ] **Step 2: Create connections mod.rs**

Create `src-tauri/src/connections/mod.rs`:

```rust
pub mod manager;
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/connections/
git commit -m "feat: add ConnectionManager with pool creation and lifecycle management"
```

---

### Task 7: Implement connection Tauri commands

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Create: `src-tauri/src/commands/connection_cmds.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Create connection commands**

Create `src-tauri/src/commands/connection_cmds.rs`:

```rust
use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::connection::{ConnectionConfig, ConnectionInfo, ConnectionUpdate, SslMode};

#[tauri::command]
pub async fn add_connection(
    config: ConnectionConfig,
    manager: State<'_, ConnectionManager>,
) -> Result<String, AppError> {
    manager.add(config).await
}

#[tauri::command]
pub async fn remove_connection(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), AppError> {
    manager.remove(&connection_id).await
}

#[tauri::command]
pub async fn test_connection(
    host: String,
    port: u16,
    database: String,
    username: String,
    password: String,
    ssl_mode: SslMode,
) -> Result<(), AppError> {
    ConnectionManager::test_connection(&host, port, &database, &username, &password, &ssl_mode).await
}

#[tauri::command]
pub async fn list_connections(
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<ConnectionInfo>, AppError> {
    Ok(manager.list().await)
}
```

- [ ] **Step 2: Create commands mod.rs**

Create `src-tauri/src/commands/mod.rs`:

```rust
pub mod connection_cmds;
```

- [ ] **Step 3: Register commands and state in lib.rs**

Update `src-tauri/src/lib.rs`:

```rust
mod commands;
mod config;
mod connections;
mod error;
mod models;

use connections::manager::ConnectionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(ConnectionManager::new())
        .invoke_handler(tauri::generate_handler![
            commands::connection_cmds::add_connection,
            commands::connection_cmds::remove_connection,
            commands::connection_cmds::test_connection,
            commands::connection_cmds::list_connections,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd /mnt/d/ai/projects/wolverine-monitor/src-tauri
cargo build
```

Expected: Build succeeds.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add connection Tauri commands with state management"
```

---

### Task 8: Build Connections View frontend

**Files:**
- Create: `src/lib/stores/connections.ts`
- Create: `src/lib/components/connections/ConnectionForm.svelte`
- Create: `src/lib/components/connections/ConnectionList.svelte`
- Create: `src/lib/components/connections/ConnectionStatus.svelte`
- Modify: `src/lib/views/Connections.svelte`

- [ ] **Step 1: Create connections store**

Create `src/lib/stores/connections.ts`:

```typescript
import { writable, derived } from "svelte/store";
import type { ConnectionInfo } from "../types";
import { addConnection, removeConnection, testConnection as testConn } from "../tauri";
import { toasts } from "./toasts";

export const connections = writable<ConnectionInfo[]>([]);
export const activeConnectionId = writable<string | null>(null);

export const activeConnection = derived(
  [connections, activeConnectionId],
  ([$connections, $activeId]) =>
    $connections.find((c) => c.config.id === $activeId) ?? null
);

export async function createConnection(config: Omit<import("../types").ConnectionConfig, "id">) {
  try {
    const id = await addConnection(config as any);
    toasts.add(`Connected to ${config.name}`, "success");
    // Refresh connection list
    const { invoke } = await import("@tauri-apps/api/core");
    const list = await invoke<ConnectionInfo[]>("list_connections");
    connections.set(list);
    activeConnectionId.set(id);
    return id;
  } catch (e) {
    toasts.add(`Connection failed: ${e}`, "error");
    throw e;
  }
}

export async function deleteConnection(id: string) {
  try {
    await removeConnection(id);
    connections.update((cs) => cs.filter((c) => c.config.id !== id));
    activeConnectionId.update((current) => (current === id ? null : current));
    toasts.add("Connection removed", "info");
  } catch (e) {
    toasts.add(`Failed to remove: ${e}`, "error");
  }
}

export async function testConnectionConfig(
  host: string, port: number, database: string,
  username: string, password: string, sslMode: import("../types").SslMode
) {
  try {
    await testConn(host, port, database, username, password, sslMode);
    toasts.add("Connection successful!", "success");
    return true;
  } catch (e) {
    toasts.add(`Connection test failed: ${e}`, "error");
    return false;
  }
}
```

- [ ] **Step 2: Create ConnectionStatus component**

Create `src/lib/components/connections/ConnectionStatus.svelte`:

```svelte
<script lang="ts">
  import type { ConnectionStatus } from "../../types";

  interface Props {
    status: ConnectionStatus;
  }
  let { status }: Props = $props();

  const statusText = $derived(typeof status === "string" ? status : "Error");
  const statusColor = $derived(
    status === "Connected"
      ? "bg-green-500"
      : status === "Reconnecting"
        ? "bg-yellow-500"
        : status === "Disconnected"
          ? "bg-gray-500"
          : "bg-red-500"
  );
</script>

<span class="inline-flex items-center gap-1.5 text-xs">
  <span class="w-2 h-2 rounded-full {statusColor}"></span>
  {statusText}
</span>
```

- [ ] **Step 3: Create ConnectionForm component**

Create `src/lib/components/connections/ConnectionForm.svelte`:

```svelte
<script lang="ts">
  import type { SslMode } from "../../types";
  import { createConnection, testConnectionConfig } from "../../stores/connections";

  let name = $state("");
  let host = $state("localhost");
  let port = $state(5432);
  let database = $state("");
  let schema = $state("public");
  let username = $state("");
  let password = $state("");
  let sslMode = $state<SslMode>("Prefer");
  let testing = $state(false);
  let saving = $state(false);

  async function handleTest() {
    testing = true;
    await testConnectionConfig(host, port, database, username, password, sslMode);
    testing = false;
  }

  async function handleSave() {
    saving = true;
    try {
      await createConnection({ name, host, port, database, schema, username, password, ssl_mode: sslMode });
      // Reset form
      name = ""; database = ""; username = ""; password = "";
    } catch { /* toast already shown */ }
    saving = false;
  }
</script>

<form class="space-y-4 p-4 bg-[var(--color-surface-raised)] rounded-lg border border-[var(--color-border)]"
      onsubmit={(e) => { e.preventDefault(); handleSave(); }}>
  <h3 class="text-sm font-semibold">New Connection</h3>

  <div class="grid grid-cols-2 gap-3">
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Name</span>
      <input bind:value={name} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Host</span>
      <input bind:value={host} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Port</span>
      <input bind:value={port} type="number" required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Database</span>
      <input bind:value={database} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Schema</span>
      <input bind:value={schema} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Username</span>
      <input bind:value={username} required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">Password</span>
      <input bind:value={password} type="password" required
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
    </label>
    <label class="block">
      <span class="text-xs text-[var(--color-text-secondary)]">SSL Mode</span>
      <select bind:value={sslMode}
        class="mt-1 w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm">
        <option value="Disable">Disable</option>
        <option value="Prefer">Prefer</option>
        <option value="Require">Require</option>
        <option value="VerifyCa">Verify CA</option>
      </select>
    </label>
  </div>

  <div class="flex gap-2">
    <button type="button" onclick={handleTest} disabled={testing}
      class="px-4 py-1.5 text-sm rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] disabled:opacity-50">
      {testing ? "Testing..." : "Test Connection"}
    </button>
    <button type="submit" disabled={saving}
      class="px-4 py-1.5 text-sm rounded bg-blue-600 hover:bg-blue-700 text-white disabled:opacity-50">
      {saving ? "Connecting..." : "Save & Connect"}
    </button>
  </div>
</form>
```

- [ ] **Step 4: Create ConnectionList component**

Create `src/lib/components/connections/ConnectionList.svelte`:

```svelte
<script lang="ts">
  import { connections, activeConnectionId, deleteConnection } from "../../stores/connections";
  import ConnectionStatus from "./ConnectionStatus.svelte";

  function activate(id: string) {
    activeConnectionId.set(id);
  }
</script>

{#if $connections.length === 0}
  <p class="text-sm text-[var(--color-text-secondary)] p-4">No connections configured.</p>
{:else}
  <div class="space-y-2">
    {#each $connections as conn (conn.config.id)}
      <div
        class="flex items-center justify-between p-3 rounded-lg border transition-colors cursor-pointer
          {$activeConnectionId === conn.config.id
            ? 'border-blue-500 bg-[var(--color-surface-overlay)]'
            : 'border-[var(--color-border)] bg-[var(--color-surface-raised)] hover:bg-[var(--color-surface-overlay)]'}"
        onclick={() => activate(conn.config.id)}
        role="button"
        tabindex="0"
      >
        <div>
          <div class="text-sm font-medium">{conn.config.name}</div>
          <div class="text-xs text-[var(--color-text-secondary)]">
            {conn.config.host}:{conn.config.port}/{conn.config.database}
          </div>
        </div>
        <div class="flex items-center gap-3">
          <ConnectionStatus status={conn.status} />
          <button
            onclick={(e) => { e.stopPropagation(); deleteConnection(conn.config.id); }}
            class="text-xs text-red-400 hover:text-red-300"
          >
            Remove
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}
```

- [ ] **Step 5: Build the Connections view**

Replace `src/lib/views/Connections.svelte`:

```svelte
<script lang="ts">
  import ConnectionForm from "../components/connections/ConnectionForm.svelte";
  import ConnectionList from "../components/connections/ConnectionList.svelte";
</script>

<div class="p-6 max-w-3xl">
  <h1 class="text-xl font-semibold mb-6">Connections</h1>

  <div class="space-y-6">
    <ConnectionList />
    <ConnectionForm />
  </div>
</div>
```

- [ ] **Step 6: Verify the app compiles**

```bash
npm run tauri dev
```

Expected: Connections view shows the form and empty list. Adding a connection to a running PostgreSQL shows "Connected" status.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add Connections view with add/remove/test connection functionality"
```

---

## Chunk 3: Trigger Installer + Query Engine (Backend)

This chunk adds trigger SQL generation, installation/uninstallation, and all query commands for envelopes, dead letters, nodes, and dashboard stats. At the end, the backend can install NOTIFY triggers and serve paginated envelope data.

### Task 9: Implement TriggerInstaller

**Files:**
- Create: `src-tauri/src/triggers/mod.rs`
- Create: `src-tauri/src/triggers/installer.rs`
- Create: `src-tauri/src/commands/trigger_cmds.rs`

- [ ] **Step 1: Write trigger SQL generation tests**

Add to `src-tauri/src/triggers/installer.rs`:

```rust
pub struct TriggerInstaller;

impl TriggerInstaller {
    /// Generate the CREATE FUNCTION SQL for the NOTIFY trigger.
    pub fn create_function_sql(schema: &str) -> String {
        format!(
            r#"CREATE OR REPLACE FUNCTION {schema}.wolverine_monitor_notify()
RETURNS trigger AS $$
BEGIN
  IF TG_OP = 'DELETE' THEN
    PERFORM pg_notify(
      '{schema}_' || TG_TABLE_NAME || '_changed',
      json_build_object(
        'op', TG_OP,
        'id', OLD.id::text,
        'message_type', OLD.message_type
      )::text
    );
    RETURN OLD;
  ELSE
    PERFORM pg_notify(
      '{schema}_' || TG_TABLE_NAME || '_changed',
      json_build_object(
        'op', TG_OP,
        'id', NEW.id::text,
        'message_type', NEW.message_type
      )::text
    );
    RETURN NEW;
  END IF;
END;
$$ LANGUAGE plpgsql;
-- wolverine_monitor_version: 0.1.0"#,
            schema = schema
        )
    }

    /// Generate CREATE TRIGGER SQL for a specific table.
    pub fn create_trigger_sql(schema: &str, table: &str) -> String {
        let trigger_name = format!("wolverine_monitor_{}_notify", table);
        format!(
            r#"CREATE OR REPLACE TRIGGER {trigger_name}
AFTER INSERT OR UPDATE OR DELETE ON {schema}.{table}
FOR EACH ROW EXECUTE FUNCTION {schema}.wolverine_monitor_notify();"#,
            trigger_name = trigger_name,
            schema = schema,
            table = table
        )
    }

    /// Generate DROP TRIGGER SQL for a specific table.
    pub fn drop_trigger_sql(schema: &str, table: &str) -> String {
        let trigger_name = format!("wolverine_monitor_{}_notify", table);
        format!(
            "DROP TRIGGER IF EXISTS {trigger_name} ON {schema}.{table};",
            trigger_name = trigger_name,
            schema = schema,
            table = table
        )
    }

    /// Generate DROP FUNCTION SQL.
    pub fn drop_function_sql(schema: &str) -> String {
        format!(
            "DROP FUNCTION IF EXISTS {schema}.wolverine_monitor_notify();",
            schema = schema
        )
    }

    pub const ENVELOPE_TABLES: &[&str] = &[
        "wolverine_incoming_envelopes",
        "wolverine_outgoing_envelopes",
        "wolverine_dead_letters",
    ];

    /// Install triggers on all envelope tables.
    pub async fn install(
        client: &tokio_postgres::Client,
        schema: &str,
    ) -> Result<(), crate::error::AppError> {
        // Create the function
        client.batch_execute(&Self::create_function_sql(schema)).await?;

        // Create triggers on each table
        for table in Self::ENVELOPE_TABLES {
            client.batch_execute(&Self::create_trigger_sql(schema, table)).await?;
        }

        Ok(())
    }

    /// Remove all triggers and the function.
    pub async fn uninstall(
        client: &tokio_postgres::Client,
        schema: &str,
    ) -> Result<(), crate::error::AppError> {
        for table in Self::ENVELOPE_TABLES {
            client.batch_execute(&Self::drop_trigger_sql(schema, table)).await?;
        }
        client.batch_execute(&Self::drop_function_sql(schema)).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_function_sql_contains_schema() {
        let sql = TriggerInstaller::create_function_sql("myschema");
        assert!(sql.contains("myschema.wolverine_monitor_notify()"));
        assert!(sql.contains("'myschema_'"));
    }

    #[test]
    fn test_create_trigger_sql_format() {
        let sql = TriggerInstaller::create_trigger_sql("public", "wolverine_incoming_envelopes");
        assert!(sql.contains("wolverine_monitor_wolverine_incoming_envelopes_notify"));
        assert!(sql.contains("AFTER INSERT OR UPDATE OR DELETE"));
        assert!(sql.contains("public.wolverine_incoming_envelopes"));
    }

    #[test]
    fn test_drop_trigger_sql_format() {
        let sql = TriggerInstaller::drop_trigger_sql("public", "wolverine_dead_letters");
        assert!(sql.contains("DROP TRIGGER IF EXISTS"));
        assert!(sql.contains("public.wolverine_dead_letters"));
    }

    #[test]
    fn test_function_includes_version_comment() {
        let sql = TriggerInstaller::create_function_sql("public");
        assert!(sql.contains("wolverine_monitor_version: 0.1.0"));
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cd /mnt/d/ai/projects/wolverine-monitor/src-tauri
cargo test triggers
```

Expected: All 4 trigger tests pass.

- [ ] **Step 3: Create triggers mod.rs**

Create `src-tauri/src/triggers/mod.rs`:

```rust
pub mod installer;
```

- [ ] **Step 4: Create trigger Tauri commands**

Create `src-tauri/src/commands/trigger_cmds.rs`:

```rust
use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::triggers::installer::TriggerInstaller;

#[tauri::command]
pub async fn install_triggers(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    TriggerInstaller::install(&client, &schema).await?;
    manager.set_triggers_installed(&connection_id, true).await;
    Ok(())
}

#[tauri::command]
pub async fn uninstall_triggers(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    TriggerInstaller::uninstall(&client, &schema).await?;
    manager.set_triggers_installed(&connection_id, false).await;
    Ok(())
}
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add TriggerInstaller with SQL generation, install/uninstall commands"
```

---

### Task 10: Implement envelope and dead letter queries

**Files:**
- Create: `src-tauri/src/queries/mod.rs`
- Create: `src-tauri/src/queries/envelopes.rs`
- Create: `src-tauri/src/queries/dead_letters.rs`
- Create: `src-tauri/src/commands/envelope_cmds.rs`
- Create: `src-tauri/src/commands/dead_letter_cmds.rs`

- [ ] **Step 1: Create envelope query functions**

Create `src-tauri/src/queries/envelopes.rs`:

```rust
use tokio_postgres::Client;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::envelope::*;

pub async fn query_incoming(
    client: &Client,
    schema: &str,
    filters: &EnvelopeFilters,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<IncomingEnvelope>, AppError> {
    let offset = page * page_size;
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();
    let mut idx = 1;

    if let Some(ref status) = filters.status {
        conditions.push(format!("status = ${}", idx));
        params.push(Box::new(status.clone()));
        idx += 1;
    }
    if let Some(ref msg_type) = filters.message_type {
        conditions.push(format!("message_type ILIKE ${}", idx));
        params.push(Box::new(format!("%{}%", msg_type)));
        idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    // Count query
    let count_sql = format!(
        "SELECT COUNT(*) FROM {schema}.wolverine_incoming_envelopes {where_clause}"
    );
    let count_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref()).collect();
    let count_row = client.query_one(&count_sql, &count_params).await?;
    let total: i64 = count_row.get(0);

    // Data query
    let data_sql = format!(
        "SELECT id, status, owner_id, execution_time, attempts, body, message_type, received_at, keep_until \
         FROM {schema}.wolverine_incoming_envelopes {where_clause} \
         ORDER BY execution_time DESC NULLS LAST \
         LIMIT ${idx} OFFSET ${next}",
        schema = schema,
        where_clause = where_clause,
        idx = idx,
        next = idx + 1,
    );
    params.push(Box::new(page_size));
    params.push(Box::new(offset));
    let data_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref()).collect();

    let rows = client.query(&data_sql, &data_params).await?;
    let items: Vec<IncomingEnvelope> = rows
        .iter()
        .map(|row| IncomingEnvelope {
            id: row.get("id"),
            status: EnvelopeStatus::from_str(row.get::<_, &str>("status"))
                .unwrap_or(EnvelopeStatus::Incoming),
            owner_id: row.get("owner_id"),
            execution_time: row.get("execution_time"),
            attempts: row.get("attempts"),
            body: row.get("body"),
            message_type: row.get("message_type"),
            received_at: row.get("received_at"),
            keep_until: row.get("keep_until"),
        })
        .collect();

    Ok(PaginatedResult { items, total, page, page_size })
}

pub async fn query_outgoing(
    client: &Client,
    schema: &str,
    filters: &EnvelopeFilters,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<OutgoingEnvelope>, AppError> {
    let offset = page * page_size;
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();
    let mut idx = 1;

    if let Some(ref msg_type) = filters.message_type {
        conditions.push(format!("message_type ILIKE ${}", idx));
        params.push(Box::new(format!("%{}%", msg_type)));
        idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM {schema}.wolverine_outgoing_envelopes {where_clause}"
    );
    let count_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref()).collect();
    let count_row = client.query_one(&count_sql, &count_params).await?;
    let total: i64 = count_row.get(0);

    let data_sql = format!(
        "SELECT id, owner_id, destination, deliver_by, body, attempts, message_type \
         FROM {schema}.wolverine_outgoing_envelopes {where_clause} \
         ORDER BY deliver_by DESC NULLS LAST \
         LIMIT ${idx} OFFSET ${next}",
        schema = schema,
        where_clause = where_clause,
        idx = idx,
        next = idx + 1,
    );
    params.push(Box::new(page_size));
    params.push(Box::new(offset));
    let data_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref()).collect();

    let rows = client.query(&data_sql, &data_params).await?;
    let items: Vec<OutgoingEnvelope> = rows
        .iter()
        .map(|row| OutgoingEnvelope {
            id: row.get("id"),
            owner_id: row.get("owner_id"),
            destination: row.get("destination"),
            deliver_by: row.get("deliver_by"),
            body: row.get("body"),
            attempts: row.get("attempts"),
            message_type: row.get("message_type"),
        })
        .collect();

    Ok(PaginatedResult { items, total, page, page_size })
}
```

- [ ] **Step 2: Create dead letter query + replay functions**

Create `src-tauri/src/queries/dead_letters.rs`:

```rust
use tokio_postgres::Client;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::dead_letter::*;
use crate::models::envelope::{EnvelopeFilters, PaginatedResult};

pub async fn query_dead_letters(
    client: &Client,
    schema: &str,
    filters: &EnvelopeFilters,
    page: i64,
    page_size: i64,
) -> Result<PaginatedResult<DeadLetter>, AppError> {
    let offset = page * page_size;
    let mut conditions = Vec::new();
    let mut params: Vec<Box<dyn tokio_postgres::types::ToSql + Sync>> = Vec::new();
    let mut idx = 1;

    if let Some(ref msg_type) = filters.message_type {
        conditions.push(format!("message_type ILIKE ${}", idx));
        params.push(Box::new(format!("%{}%", msg_type)));
        idx += 1;
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_sql = format!(
        "SELECT COUNT(*) FROM {schema}.wolverine_dead_letters {where_clause}"
    );
    let count_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref()).collect();
    let count_row = client.query_one(&count_sql, &count_params).await?;
    let total: i64 = count_row.get(0);

    let data_sql = format!(
        "SELECT id, execution_time, body, message_type, received_at, source, \
         exception_type, exception_message, sent_at, replayable \
         FROM {schema}.wolverine_dead_letters {where_clause} \
         ORDER BY sent_at DESC NULLS LAST \
         LIMIT ${idx} OFFSET ${next}",
        schema = schema,
        where_clause = where_clause,
        idx = idx,
        next = idx + 1,
    );
    params.push(Box::new(page_size));
    params.push(Box::new(offset));
    let data_params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> =
        params.iter().map(|p| p.as_ref()).collect();

    let rows = client.query(&data_sql, &data_params).await?;
    let items: Vec<DeadLetter> = rows
        .iter()
        .map(|row| DeadLetter {
            id: row.get("id"),
            execution_time: row.get("execution_time"),
            body: row.get("body"),
            message_type: row.get("message_type"),
            received_at: row.get("received_at"),
            source: row.get("source"),
            exception_type: row.get("exception_type"),
            exception_message: row.get("exception_message"),
            sent_at: row.get("sent_at"),
            replayable: row.get("replayable"),
        })
        .collect();

    Ok(PaginatedResult { items, total, page, page_size })
}

pub async fn replay_single(
    client: &Client,
    schema: &str,
    id: Uuid,
) -> Result<(), AppError> {
    let tx = client.transaction().await?;

    // 1. Read dead letter
    let row = tx
        .query_one(
            &format!(
                "SELECT id, body, message_type, received_at, replayable FROM {schema}.wolverine_dead_letters WHERE id = $1"
            ),
            &[&id],
        )
        .await?;

    let replayable: bool = row.get("replayable");
    if !replayable {
        return Err(AppError::NotReplayable(id));
    }

    // 2. Insert into incoming
    tx.execute(
        &format!(
            "INSERT INTO {schema}.wolverine_incoming_envelopes \
             (id, status, owner_id, execution_time, attempts, body, message_type, received_at, keep_until) \
             VALUES ($1, 'Incoming', 0, NULL, 0, $2, $3, $4, NULL)"
        ),
        &[
            &id,
            &row.get::<_, Vec<u8>>("body"),
            &row.get::<_, &str>("message_type"),
            &row.get::<_, Option<&str>>("received_at"),
        ],
    )
    .await?;

    // 3. Delete from dead letters
    tx.execute(
        &format!("DELETE FROM {schema}.wolverine_dead_letters WHERE id = $1"),
        &[&id],
    )
    .await?;

    tx.commit().await?;
    Ok(())
}

pub async fn replay_bulk(
    client: &Client,
    schema: &str,
    ids: &[Uuid],
) -> Result<BulkReplayResult, AppError> {
    let mut result = BulkReplayResult {
        succeeded: 0,
        failed: 0,
        errors: Vec::new(),
    };

    for &id in ids {
        match replay_single(client, schema, id).await {
            Ok(()) => result.succeeded += 1,
            Err(e) => {
                result.failed += 1;
                result.errors.push(ReplayError {
                    id,
                    reason: e.to_string(),
                });
            }
        }
    }

    Ok(result)
}
```

- [ ] **Step 3: Create Tauri commands for envelopes and dead letters**

Create `src-tauri/src/commands/envelope_cmds.rs`:

```rust
use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::envelope::*;
use crate::queries::envelopes;

#[tauri::command]
pub async fn get_incoming_envelopes(
    connection_id: String,
    filters: EnvelopeFilters,
    page: i64,
    page_size: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<PaginatedResult<IncomingEnvelope>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    envelopes::query_incoming(&client, &schema, &filters, page, page_size).await
}

#[tauri::command]
pub async fn get_outgoing_envelopes(
    connection_id: String,
    filters: EnvelopeFilters,
    page: i64,
    page_size: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<PaginatedResult<OutgoingEnvelope>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    envelopes::query_outgoing(&client, &schema, &filters, page, page_size).await
}
```

Create `src-tauri/src/commands/dead_letter_cmds.rs`:

```rust
use tauri::State;
use uuid::Uuid;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::dead_letter::*;
use crate::models::envelope::*;
use crate::queries::dead_letters;

#[tauri::command]
pub async fn get_dead_letters(
    connection_id: String,
    filters: EnvelopeFilters,
    page: i64,
    page_size: i64,
    manager: State<'_, ConnectionManager>,
) -> Result<PaginatedResult<DeadLetter>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    dead_letters::query_dead_letters(&client, &schema, &filters, page, page_size).await
}

#[tauri::command]
pub async fn replay_dead_letter(
    connection_id: String,
    id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<(), AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    let uuid = Uuid::parse_str(&id).map_err(|e| AppError::Config(e.to_string()))?;
    dead_letters::replay_single(&client, &schema, uuid).await
}

#[tauri::command]
pub async fn replay_dead_letters_bulk(
    connection_id: String,
    ids: Vec<String>,
    manager: State<'_, ConnectionManager>,
) -> Result<BulkReplayResult, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    let uuids: Vec<Uuid> = ids
        .iter()
        .map(|s| Uuid::parse_str(s).map_err(|e| AppError::Config(e.to_string())))
        .collect::<Result<_, _>>()?;
    dead_letters::replay_bulk(&client, &schema, &uuids).await
}
```

- [ ] **Step 4: Create queries mod.rs and update commands mod.rs**

Create `src-tauri/src/queries/mod.rs`:

```rust
pub mod dead_letters;
pub mod envelopes;
```

Update `src-tauri/src/commands/mod.rs`:

```rust
pub mod connection_cmds;
pub mod dead_letter_cmds;
pub mod envelope_cmds;
pub mod trigger_cmds;
```

- [ ] **Step 5: Register new commands in lib.rs**

Add all new commands to the `invoke_handler` in `src-tauri/src/lib.rs`:

```rust
mod commands;
mod config;
mod connections;
mod error;
mod models;
mod queries;
mod triggers;

use connections::manager::ConnectionManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_log::Builder::default().build())
        .manage(ConnectionManager::new())
        .invoke_handler(tauri::generate_handler![
            commands::connection_cmds::add_connection,
            commands::connection_cmds::remove_connection,
            commands::connection_cmds::test_connection,
            commands::connection_cmds::list_connections,
            commands::trigger_cmds::install_triggers,
            commands::trigger_cmds::uninstall_triggers,
            commands::envelope_cmds::get_incoming_envelopes,
            commands::envelope_cmds::get_outgoing_envelopes,
            commands::dead_letter_cmds::get_dead_letters,
            commands::dead_letter_cmds::replay_dead_letter,
            commands::dead_letter_cmds::replay_dead_letters_bulk,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Verify compilation and run tests**

```bash
cd /mnt/d/ai/projects/wolverine-monitor/src-tauri
cargo test && cargo build
```

Expected: All tests pass, build succeeds.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: add envelope/dead letter queries, replay mechanism, and trigger commands"
```

---

### Task 11: Implement node queries and dashboard stats

**Files:**
- Create: `src-tauri/src/queries/nodes.rs`
- Create: `src-tauri/src/queries/dashboard.rs`
- Create: `src-tauri/src/commands/node_cmds.rs`
- Create: `src-tauri/src/commands/dashboard_cmds.rs`

- [ ] **Step 1: Create node query**

Create `src-tauri/src/queries/nodes.rs`:

```rust
use tokio_postgres::Client;

use crate::error::AppError;
use crate::models::node::WolverineNode;

pub async fn query_nodes(
    client: &Client,
    schema: &str,
) -> Result<Vec<WolverineNode>, AppError> {
    let sql = format!(
        "SELECT id, node_number, description, uri, started, health_check, version, capabilities \
         FROM {schema}.wolverine_nodes \
         ORDER BY node_number"
    );
    let rows = client.query(&sql, &[]).await?;
    let nodes = rows
        .iter()
        .map(|row| WolverineNode {
            id: row.get("id"),
            node_number: row.get("node_number"),
            description: row.get("description"),
            uri: row.get("uri"),
            started: row.get("started"),
            health_check: row.get("health_check"),
            version: row.get("version"),
            capabilities: row.get("capabilities"),
        })
        .collect();
    Ok(nodes)
}
```

- [ ] **Step 2: Create dashboard stats query**

Create `src-tauri/src/queries/dashboard.rs`:

```rust
use tokio_postgres::Client;

use crate::error::AppError;
use crate::models::dashboard::DashboardStats;

pub async fn query_stats(
    client: &Client,
    schema: &str,
) -> Result<DashboardStats, AppError> {
    // Count incoming by status
    let incoming_sql = format!(
        "SELECT status, COUNT(*) as cnt FROM {schema}.wolverine_incoming_envelopes GROUP BY status"
    );
    let rows = client.query(&incoming_sql, &[]).await?;

    let mut stats = DashboardStats::default();
    for row in &rows {
        let status: &str = row.get("status");
        let count: i64 = row.get("cnt");
        match status {
            "Incoming" => stats.incoming_count = count,
            "Scheduled" => stats.incoming_scheduled = count,
            "Handled" => stats.incoming_handled = count,
            _ => {}
        }
    }

    // Count outgoing
    let outgoing_sql = format!(
        "SELECT COUNT(*) FROM {schema}.wolverine_outgoing_envelopes"
    );
    let row = client.query_one(&outgoing_sql, &[]).await?;
    stats.outgoing_count = row.get(0);

    // Count dead letters
    let dead_sql = format!(
        "SELECT COUNT(*) FROM {schema}.wolverine_dead_letters"
    );
    let row = client.query_one(&dead_sql, &[]).await?;
    stats.dead_letter_count = row.get(0);

    Ok(stats)
}
```

- [ ] **Step 3: Create Tauri commands**

Create `src-tauri/src/commands/node_cmds.rs`:

```rust
use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::node::WolverineNode;
use crate::queries::nodes;

#[tauri::command]
pub async fn get_nodes(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<Vec<WolverineNode>, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    nodes::query_nodes(&client, &schema).await
}
```

Create `src-tauri/src/commands/dashboard_cmds.rs`:

```rust
use tauri::State;

use crate::connections::manager::ConnectionManager;
use crate::error::AppError;
use crate::models::dashboard::DashboardStats;
use crate::queries::dashboard;

#[tauri::command]
pub async fn get_dashboard_stats(
    connection_id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<DashboardStats, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    dashboard::query_stats(&client, &schema).await
}
```

- [ ] **Step 4: Update mod.rs files and register commands**

Update `src-tauri/src/queries/mod.rs`:

```rust
pub mod dashboard;
pub mod dead_letters;
pub mod envelopes;
pub mod nodes;
```

Update `src-tauri/src/commands/mod.rs`:

```rust
pub mod connection_cmds;
pub mod dashboard_cmds;
pub mod dead_letter_cmds;
pub mod envelope_cmds;
pub mod node_cmds;
pub mod trigger_cmds;
```

Add to `invoke_handler` in `src-tauri/src/lib.rs`:

```rust
commands::node_cmds::get_nodes,
commands::dashboard_cmds::get_dashboard_stats,
```

- [ ] **Step 5: Verify compilation**

```bash
cd /mnt/d/ai/projects/wolverine-monitor/src-tauri
cargo build
```

Expected: Build succeeds.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add node queries, dashboard stats, and their Tauri commands"
```

---

## Chunk 4: LISTEN/NOTIFY Monitor + Alert Engine

This chunk adds the real-time monitoring layer: subscribing to PostgreSQL NOTIFY channels, emitting Tauri events, and the alert engine for dead letter notifications.

### Task 12: Implement NotifyListener

**Files:**
- Create: `src-tauri/src/monitor/mod.rs`
- Create: `src-tauri/src/monitor/listener.rs`

- [ ] **Step 1: Create the LISTEN/NOTIFY subscriber**

Create `src-tauri/src/monitor/listener.rs`:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{AsyncMessage, NoTls};
use futures_util::StreamExt;
use tauri::{AppHandle, Emitter};
use tracing::{info, warn, error};

use crate::models::connection::ConnectionConfig;
use crate::models::notification::{NotifyPayload, NotifyOp};

pub struct NotifyListener {
    handles: Arc<RwLock<Vec<tokio::task::JoinHandle<()>>>>,
}

impl NotifyListener {
    pub fn new() -> Self {
        Self {
            handles: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start listening for NOTIFY events on a connection.
    pub async fn start_listening(
        &self,
        app: AppHandle,
        config: ConnectionConfig,
    ) {
        let connection_id = config.id.clone();
        let schema = config.schema.clone();

        let handle = tokio::spawn(async move {
            let conn_str = format!(
                "host={} port={} dbname={} user={} password={}",
                config.host, config.port, config.database, config.username, config.password
            );

            loop {
                match tokio_postgres::connect(&conn_str, NoTls).await {
                    Ok((client, mut connection)) => {
                        // Subscribe to channels
                        let channels = [
                            format!("{schema}_wolverine_incoming_envelopes_changed"),
                            format!("{schema}_wolverine_outgoing_envelopes_changed"),
                            format!("{schema}_wolverine_dead_letters_changed"),
                        ];

                        for channel in &channels {
                            if let Err(e) = client
                                .batch_execute(&format!("LISTEN \"{}\";", channel))
                                .await
                            {
                                error!("Failed to LISTEN on {}: {}", channel, e);
                                continue;
                            }
                        }

                        info!("Listening for NOTIFY on connection {}", connection_id);

                        // Process notifications
                        let mut stream = futures_util::stream::poll_fn(move |cx| {
                            connection.poll_message(cx)
                        });

                        while let Some(msg) = stream.next().await {
                            match msg {
                                Ok(AsyncMessage::Notification(n)) => {
                                    if let Some(payload) = NotifyPayload::parse(n.payload()) {
                                        let table = if n.channel().contains("incoming") {
                                            "incoming"
                                        } else if n.channel().contains("outgoing") {
                                            "outgoing"
                                        } else {
                                            "dead_letter"
                                        };

                                        let event = serde_json::json!({
                                            "connection_id": connection_id,
                                            "table": table,
                                            "op": match payload.op {
                                                NotifyOp::Insert => "INSERT",
                                                NotifyOp::Update => "UPDATE",
                                                NotifyOp::Delete => "DELETE",
                                            },
                                            "id": payload.id.to_string(),
                                            "message_type": payload.message_type,
                                        });

                                        let _ = app.emit("envelope:changed", event);
                                    } else {
                                        warn!("Malformed NOTIFY payload: {}", n.payload());
                                    }
                                }
                                Ok(AsyncMessage::Notice(n)) => {
                                    info!("PostgreSQL notice: {}", n.message());
                                }
                                Ok(_) => {}
                                Err(e) => {
                                    error!("LISTEN connection error: {}", e);
                                    break;
                                }
                            }
                        }

                        warn!("LISTEN connection dropped for {}, reconnecting...", connection_id);
                    }
                    Err(e) => {
                        error!("Failed to connect LISTEN for {}: {}", connection_id, e);
                    }
                }

                // Exponential backoff: 1s, 2s, 4s, 8s, 16s, 30s max
                let delay = std::cmp::min(30, 1u64 << retry_count.min(5));
                retry_count += 1;
                warn!("Reconnecting in {}s (attempt {})", delay, retry_count);
                tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                if retry_count >= 5 {
                    error!("Max retries reached for LISTEN on {}, giving up", connection_id);
                    let _ = app.emit("connection:status", serde_json::json!({
                        "connection_id": connection_id,
                        "status": "Error",
                        "message": "LISTEN connection lost after 5 retries"
                    }));
                    break;
                }
            }
        });

        self.handles.write().await.push(handle);
    }

    /// Stop all listeners.
    pub async fn stop_all(&self) {
        let mut handles = self.handles.write().await;
        for handle in handles.drain(..) {
            handle.abort();
        }
    }
}
```

- [ ] **Step 1a: Add futures-util dependency**

Add to `src-tauri/Cargo.toml` under `[dependencies]`:

```toml
futures-util = "0.3"
```

- [ ] **Step 2: Create monitor mod.rs**

Create `src-tauri/src/monitor/mod.rs`:

```rust
pub mod listener;
```

- [ ] **Step 3: Register NotifyListener as Tauri state and start on connection add**

Update `src-tauri/src/lib.rs` to manage `NotifyListener`:

```rust
mod monitor;

use monitor::listener::NotifyListener;

// In the Builder:
.manage(NotifyListener::new())
```

Update `src-tauri/src/commands/connection_cmds.rs` to start listening when a connection is added:

```rust
use crate::monitor::listener::NotifyListener;

#[tauri::command]
pub async fn add_connection(
    config: ConnectionConfig,
    manager: State<'_, ConnectionManager>,
    listener: State<'_, NotifyListener>,
    app: tauri::AppHandle,
) -> Result<String, AppError> {
    let id = manager.add(config.clone()).await?;

    // Start LISTEN/NOTIFY for this connection
    let mut listen_config = config;
    listen_config.id = id.clone();
    listener.start_listening(app, listen_config).await;

    Ok(id)
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd /mnt/d/ai/projects/wolverine-monitor/src-tauri
cargo build
```

Expected: Build succeeds.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add NotifyListener for real-time LISTEN/NOTIFY event streaming"
```

---

### Task 13: Implement AlertEngine

**Files:**
- Create: `src-tauri/src/alerts/mod.rs`
- Create: `src-tauri/src/alerts/engine.rs`

- [ ] **Step 1: Write alert matching tests first**

Create `src-tauri/src/alerts/engine.rs`:

```rust
use tauri::{AppHandle, Emitter};
use tracing::info;

use crate::models::alert::*;
use crate::models::notification::{NotifyOp, NotifyPayload};

pub struct AlertEngine {
    rules: Vec<AlertRule>,
}

impl AlertEngine {
    pub fn new() -> Self {
        Self {
            rules: vec![AlertRule {
                id: "default_any_dead_letter".to_string(),
                enabled: true,
                kind: AlertRuleKind::AnyDeadLetter,
            }],
        }
    }

    /// Check if a notification matches any alert rule.
    pub fn check(&self, table: &str, payload: &NotifyPayload) -> Vec<Alert> {
        if payload.op != NotifyOp::Insert {
            return Vec::new();
        }

        let mut alerts = Vec::new();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let matched = match &rule.kind {
                AlertRuleKind::AnyDeadLetter => table == "dead_letter",
                AlertRuleKind::DeadLetterMessageType { message_type } => {
                    table == "dead_letter" && payload.message_type.contains(message_type)
                }
                AlertRuleKind::IncomingQueueDepth { .. } => false, // Checked during stats polling
            };

            if matched {
                alerts.push(Alert {
                    rule_id: rule.id.clone(),
                    connection_id: String::new(), // Filled by caller
                    message: format!(
                        "Dead letter: {} ({})",
                        crate::models::notification::NotifyPayload::short_type(&payload.message_type),
                        payload.id
                    ),
                    timestamp: chrono::Utc::now(),
                });
            }
        }

        alerts
    }

    /// Emit alerts via Tauri events and system notifications.
    pub fn emit_alerts(&self, app: &AppHandle, connection_id: &str, alerts: &[Alert]) {
        for alert in alerts {
            let mut alert = alert.clone();
            alert.connection_id = connection_id.to_string();

            let _ = app.emit("alert:triggered", &alert);
            info!("Alert: {}", alert.message);

            // System notification (best-effort)
            #[cfg(not(test))]
            {
                use tauri_plugin_notification::NotificationExt;
                let _ = app
                    .notification()
                    .builder()
                    .title("Wolverine Monitor")
                    .body(&alert.message)
                    .show();
            }
        }
    }
}

// Add helper to NotifyPayload
impl crate::models::notification::NotifyPayload {
    pub fn short_type(message_type: &str) -> String {
        message_type
            .split('.')
            .last()
            .unwrap_or(message_type)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::notification::{NotifyOp, NotifyPayload};

    fn make_engine(rules: Vec<AlertRule>) -> AlertEngine {
        AlertEngine { rules }
    }

    fn make_payload(op: NotifyOp, msg_type: &str) -> NotifyPayload {
        NotifyPayload {
            op,
            id: uuid::Uuid::new_v4(),
            message_type: msg_type.to_string(),
        }
    }

    #[test]
    fn test_any_dead_letter_rule_matches_insert() {
        let engine = make_engine(vec![AlertRule {
            id: "r1".to_string(),
            enabled: true,
            kind: AlertRuleKind::AnyDeadLetter,
        }]);
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.Foo");
        let alerts = engine.check("dead_letter", &payload);
        assert_eq!(alerts.len(), 1);
    }

    #[test]
    fn test_any_dead_letter_rule_ignores_update() {
        let engine = make_engine(vec![AlertRule {
            id: "r1".to_string(),
            enabled: true,
            kind: AlertRuleKind::AnyDeadLetter,
        }]);
        let payload = make_payload(NotifyOp::Update, "MyApp.Commands.Foo");
        let alerts = engine.check("dead_letter", &payload);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_any_dead_letter_rule_ignores_incoming_table() {
        let engine = make_engine(vec![AlertRule {
            id: "r1".to_string(),
            enabled: true,
            kind: AlertRuleKind::AnyDeadLetter,
        }]);
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.Foo");
        let alerts = engine.check("incoming", &payload);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_message_type_filter_matches() {
        let engine = make_engine(vec![AlertRule {
            id: "r2".to_string(),
            enabled: true,
            kind: AlertRuleKind::DeadLetterMessageType {
                message_type: "Payment".to_string(),
            },
        }]);
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.PaymentFailed");
        let alerts = engine.check("dead_letter", &payload);
        assert_eq!(alerts.len(), 1);
    }

    #[test]
    fn test_message_type_filter_no_match() {
        let engine = make_engine(vec![AlertRule {
            id: "r2".to_string(),
            enabled: true,
            kind: AlertRuleKind::DeadLetterMessageType {
                message_type: "Payment".to_string(),
            },
        }]);
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.SendEmail");
        let alerts = engine.check("dead_letter", &payload);
        assert!(alerts.is_empty());
    }

    #[test]
    fn test_disabled_rule_ignored() {
        let engine = make_engine(vec![AlertRule {
            id: "r1".to_string(),
            enabled: false,
            kind: AlertRuleKind::AnyDeadLetter,
        }]);
        let payload = make_payload(NotifyOp::Insert, "MyApp.Commands.Foo");
        let alerts = engine.check("dead_letter", &payload);
        assert!(alerts.is_empty());
    }
}
```

- [ ] **Step 2: Run alert tests**

```bash
cd /mnt/d/ai/projects/wolverine-monitor/src-tauri
cargo test alerts
```

Expected: All 6 alert engine tests pass.

- [ ] **Step 3: Create alerts mod.rs**

Create `src-tauri/src/alerts/mod.rs`:

```rust
pub mod engine;
```

- [ ] **Step 4: Register AlertEngine in lib.rs**

Add to `src-tauri/src/lib.rs`:

```rust
mod alerts;
use alerts::engine::AlertEngine;

// In Builder:
.manage(AlertEngine::new())
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add AlertEngine with rule matching, dead letter alerts, and system notifications"
```

---

## Chunk 5: Frontend — Dashboard + Explorer Views

This chunk builds the Dashboard and Explorer views with their supporting stores and components. At the end, the dashboard shows live counters and message feed, and the explorer supports filtered/paginated browsing.

### Task 14: Create dashboard store and components

**Files:**
- Create: `src/lib/stores/dashboard.ts`
- Create: `src/lib/components/dashboard/CounterCard.svelte`
- Create: `src/lib/components/dashboard/LiveFeed.svelte`
- Modify: `src/lib/views/Dashboard.svelte`

- [ ] **Step 1: Create dashboard store**

Create `src/lib/stores/dashboard.ts`:

```typescript
import { writable, derived } from "svelte/store";
import type { DashboardStats, NotifyEvent } from "../types";
import { getDashboardStats, onEnvelopeChange } from "../tauri";
import { activeConnectionId } from "./connections";

export const stats = writable<DashboardStats>({
  incoming_count: 0,
  incoming_scheduled: 0,
  incoming_handled: 0,
  outgoing_count: 0,
  dead_letter_count: 0,
  throughput: [],
});

export interface RecentMessage {
  id: string;
  message_type: string;
  table: "incoming" | "outgoing" | "dead_letter";
  op: string;
  timestamp: number;
}

export const recentMessages = writable<RecentMessage[]>([]);

const MAX_RECENT = 500;

export function addRecentMessage(event: NotifyEvent) {
  recentMessages.update((msgs) => {
    const newMsg: RecentMessage = {
      id: event.id,
      message_type: event.message_type,
      table: event.table,
      op: event.op,
      timestamp: Date.now(),
    };
    const updated = [newMsg, ...msgs];
    return updated.slice(0, MAX_RECENT);
  });
}

export async function refreshStats(connectionId: string) {
  try {
    const s = await getDashboardStats(connectionId);
    stats.set(s);
  } catch {
    // Silently fail — will retry on next poll
  }
}

let pollInterval: ReturnType<typeof setInterval> | null = null;
let unlistenFn: (() => void) | null = null;

export async function startDashboard(connectionId: string) {
  stopDashboard();

  await refreshStats(connectionId);
  pollInterval = setInterval(() => refreshStats(connectionId), 5000);

  unlistenFn = await onEnvelopeChange((event) => {
    if (event.connection_id === connectionId) {
      addRecentMessage(event);
      // Refresh stats on change
      refreshStats(connectionId);
    }
  }) as any;
}

export function stopDashboard() {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
  if (unlistenFn) {
    unlistenFn();
    unlistenFn = null;
  }
}
```

- [ ] **Step 2: Create CounterCard component**

Create `src/lib/components/dashboard/CounterCard.svelte`:

```svelte
<script lang="ts">
  interface Props {
    label: string;
    value: number;
    color: string;
    rate?: string;
  }
  let { label, value, color, rate }: Props = $props();
</script>

<div class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg p-4">
  <div class="text-xs text-[var(--color-text-secondary)] uppercase tracking-wider">{label}</div>
  <div class="text-3xl font-bold mt-1" style="color: {color}">{value.toLocaleString()}</div>
  {#if rate}
    <div class="text-xs text-[var(--color-text-secondary)] mt-1">{rate}</div>
  {/if}
</div>
```

- [ ] **Step 3: Create LiveFeed component**

Create `src/lib/components/dashboard/LiveFeed.svelte`:

```svelte
<script lang="ts">
  import { recentMessages } from "../../stores/dashboard";
  import { shortenMessageType, formatRelativeTime } from "../../format";

  const statusIcon: Record<string, string> = {
    incoming: "→",
    outgoing: "←",
    dead_letter: "✗",
  };

  const statusColor: Record<string, string> = {
    incoming: "text-[var(--color-status-incoming)]",
    outgoing: "text-[var(--color-status-outgoing)]",
    dead_letter: "text-[var(--color-status-error)]",
  };
</script>

<div class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg">
  <div class="px-4 py-3 border-b border-[var(--color-border)] text-sm font-semibold">
    Recent Messages
  </div>
  <div class="max-h-80 overflow-y-auto">
    {#if $recentMessages.length === 0}
      <div class="p-4 text-sm text-[var(--color-text-secondary)]">Waiting for messages...</div>
    {:else}
      {#each $recentMessages.slice(0, 50) as msg (msg.id + msg.timestamp)}
        <div class="flex items-center gap-3 px-4 py-2 border-b border-[var(--color-border)] text-sm hover:bg-[var(--color-surface-overlay)]">
          <span class="{statusColor[msg.table]} font-mono">{statusIcon[msg.table]}</span>
          <span class="flex-1 truncate">{shortenMessageType(msg.message_type)}</span>
          <span class="text-xs text-[var(--color-text-secondary)] capitalize">{msg.table.replace("_", " ")}</span>
          <span class="text-xs text-[var(--color-text-secondary)] w-16 text-right">
            {formatRelativeTime(new Date(msg.timestamp).toISOString())}
          </span>
        </div>
      {/each}
    {/if}
  </div>
</div>
```

- [ ] **Step 4: Build the Dashboard view**

Replace `src/lib/views/Dashboard.svelte`:

```svelte
<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { stats, startDashboard, stopDashboard } from "../stores/dashboard";
  import { activeConnectionId } from "../stores/connections";
  import CounterCard from "../components/dashboard/CounterCard.svelte";
  import LiveFeed from "../components/dashboard/LiveFeed.svelte";

  $effect(() => {
    const connId = $activeConnectionId;
    if (connId) {
      startDashboard(connId);
    }
    return () => stopDashboard();
  });
</script>

<div class="p-6">
  <h1 class="text-xl font-semibold mb-6">Dashboard</h1>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view the dashboard.</p>
  {:else}
    <div class="grid grid-cols-3 gap-4 mb-6">
      <CounterCard
        label="Incoming"
        value={$stats.incoming_count}
        color="var(--color-status-incoming)"
      />
      <CounterCard
        label="Outgoing"
        value={$stats.outgoing_count}
        color="var(--color-status-outgoing)"
      />
      <CounterCard
        label="Dead Letters"
        value={$stats.dead_letter_count}
        color="var(--color-status-error)"
      />
    </div>

    <LiveFeed />
  {/if}
</div>
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Dashboard view with counter cards and live message feed"
```

---

### Task 15: Build Message Explorer view

**Files:**
- Create: `src/lib/components/explorer/FilterBar.svelte`
- Create: `src/lib/components/explorer/EnvelopeTable.svelte`
- Create: `src/lib/components/explorer/MessageDetail.svelte`
- Modify: `src/lib/views/Explorer.svelte`

- [ ] **Step 1: Create FilterBar component**

Create `src/lib/components/explorer/FilterBar.svelte`:

```svelte
<script lang="ts">
  interface Props {
    onFilter: (filters: { table: string; status: string; messageType: string }) => void;
  }
  let { onFilter }: Props = $props();

  let table = $state("incoming");
  let status = $state("");
  let messageType = $state("");

  function applyFilters() {
    onFilter({ table, status, messageType });
  }
</script>

<div class="flex items-center gap-3 p-4 bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg">
  <select bind:value={table} onchange={applyFilters}
    class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm">
    <option value="incoming">Incoming</option>
    <option value="outgoing">Outgoing</option>
    <option value="dead_letter">Dead Letters</option>
  </select>

  {#if table === "incoming"}
    <select bind:value={status} onchange={applyFilters}
      class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm">
      <option value="">All Statuses</option>
      <option value="Incoming">Incoming</option>
      <option value="Scheduled">Scheduled</option>
      <option value="Handled">Handled</option>
    </select>
  {/if}

  <input
    bind:value={messageType}
    placeholder="Filter by message type..."
    oninput={applyFilters}
    class="flex-1 bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm"
  />
</div>
```

- [ ] **Step 2: Create EnvelopeTable component**

Create `src/lib/components/explorer/EnvelopeTable.svelte`:

```svelte
<script lang="ts">
  import type { IncomingEnvelope, OutgoingEnvelope, DeadLetter } from "../../types";
  import { shortenMessageType, formatRelativeTime } from "../../format";

  type AnyEnvelope = IncomingEnvelope | OutgoingEnvelope | DeadLetter;

  interface Props {
    items: AnyEnvelope[];
    table: string;
    total: number;
    page: number;
    pageSize: number;
    onPageChange: (page: number) => void;
    onSelect: (item: AnyEnvelope) => void;
  }
  let { items, table, total, page, pageSize, onPageChange, onSelect }: Props = $props();

  const totalPages = $derived(Math.ceil(total / pageSize));
</script>

<div class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg overflow-hidden">
  <table class="w-full text-sm">
    <thead>
      <tr class="border-b border-[var(--color-border)] text-[var(--color-text-secondary)] text-xs uppercase">
        <th class="px-4 py-3 text-left">ID</th>
        <th class="px-4 py-3 text-left">Message Type</th>
        {#if table === "incoming"}
          <th class="px-4 py-3 text-left">Status</th>
        {:else if table === "outgoing"}
          <th class="px-4 py-3 text-left">Destination</th>
        {:else}
          <th class="px-4 py-3 text-left">Exception</th>
        {/if}
        <th class="px-4 py-3 text-right">Attempts</th>
      </tr>
    </thead>
    <tbody>
      {#each items as item (item.id)}
        <tr
          class="border-b border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] cursor-pointer"
          onclick={() => onSelect(item)}
        >
          <td class="px-4 py-2 font-mono text-xs truncate max-w-32">{item.id.slice(0, 8)}...</td>
          <td class="px-4 py-2">{shortenMessageType(item.message_type)}</td>
          {#if table === "incoming"}
            <td class="px-4 py-2">{(item as IncomingEnvelope).status}</td>
          {:else if table === "outgoing"}
            <td class="px-4 py-2 truncate max-w-48">{(item as OutgoingEnvelope).destination}</td>
          {:else}
            <td class="px-4 py-2 text-red-400 truncate max-w-48">{(item as DeadLetter).exception_type ?? "—"}</td>
          {/if}
          <td class="px-4 py-2 text-right">{item.attempts}</td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if totalPages > 1}
    <div class="flex items-center justify-between px-4 py-3 border-t border-[var(--color-border)]">
      <span class="text-xs text-[var(--color-text-secondary)]">{total} total</span>
      <div class="flex gap-2">
        <button disabled={page === 0} onclick={() => onPageChange(page - 1)}
          class="px-2 py-1 text-xs rounded border border-[var(--color-border)] disabled:opacity-30">Prev</button>
        <span class="text-xs py-1">{page + 1} / {totalPages}</span>
        <button disabled={page >= totalPages - 1} onclick={() => onPageChange(page + 1)}
          class="px-2 py-1 text-xs rounded border border-[var(--color-border)] disabled:opacity-30">Next</button>
      </div>
    </div>
  {/if}
</div>
```

- [ ] **Step 3: Create MessageDetail slide-over**

Create `src/lib/components/explorer/MessageDetail.svelte`:

```svelte
<script lang="ts">
  import { decodeBody } from "../../format";

  interface Props {
    item: any;
    onClose: () => void;
  }
  let { item, onClose }: Props = $props();

  const decoded = $derived(item ? decodeBody(item.body) : null);
</script>

{#if item}
  <div class="fixed inset-y-0 right-0 w-[480px] bg-[var(--color-surface-raised)] border-l border-[var(--color-border)] shadow-xl z-40 flex flex-col">
    <div class="flex items-center justify-between px-4 py-3 border-b border-[var(--color-border)]">
      <h3 class="text-sm font-semibold">Message Detail</h3>
      <button onclick={onClose} class="text-[var(--color-text-secondary)] hover:text-white">✕</button>
    </div>

    <div class="flex-1 overflow-y-auto p-4 space-y-4">
      <div>
        <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">ID</div>
        <div class="font-mono text-sm">{item.id}</div>
      </div>

      <div>
        <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Message Type</div>
        <div class="text-sm">{item.message_type}</div>
      </div>

      {#if item.status}
        <div>
          <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Status</div>
          <div class="text-sm">{item.status}</div>
        </div>
      {/if}

      {#if item.exception_type}
        <div>
          <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Exception</div>
          <div class="text-sm text-red-400">{item.exception_type}</div>
          <div class="text-xs text-red-300 mt-1">{item.exception_message}</div>
        </div>
      {/if}

      <div>
        <div class="text-xs text-[var(--color-text-secondary)] uppercase mb-1">Body</div>
        {#if decoded?.type === "json"}
          <pre class="text-xs bg-[var(--color-surface)] p-3 rounded overflow-x-auto">{decoded.content}</pre>
        {:else if decoded}
          <div>
            <pre class="text-xs bg-[var(--color-surface)] p-3 rounded overflow-x-auto font-mono">{decoded.content}</pre>
            <button
              onclick={() => navigator.clipboard.writeText(decoded.base64)}
              class="mt-2 text-xs text-blue-400 hover:text-blue-300"
            >
              Copy Base64
            </button>
          </div>
        {/if}
      </div>
    </div>
  </div>
{/if}
```

- [ ] **Step 4: Build the Explorer view**

Replace `src/lib/views/Explorer.svelte`:

```svelte
<script lang="ts">
  import { activeConnectionId } from "../stores/connections";
  import { getIncomingEnvelopes, getOutgoingEnvelopes, getDeadLetters } from "../tauri";
  import { toasts } from "../stores/toasts";
  import FilterBar from "../components/explorer/FilterBar.svelte";
  import EnvelopeTable from "../components/explorer/EnvelopeTable.svelte";
  import MessageDetail from "../components/explorer/MessageDetail.svelte";

  let items = $state<any[]>([]);
  let total = $state(0);
  let page = $state(0);
  let pageSize = 25;
  let currentTable = $state("incoming");
  let currentFilters = $state({ table: "incoming", status: "", messageType: "" });
  let selectedItem = $state<any>(null);

  async function loadData() {
    const connId = $activeConnectionId;
    if (!connId) return;

    const filters = {
      status: currentFilters.status || undefined,
      message_type: currentFilters.messageType || undefined,
    };

    try {
      let result;
      if (currentTable === "incoming") {
        result = await getIncomingEnvelopes(connId, filters, page, pageSize);
      } else if (currentTable === "outgoing") {
        result = await getOutgoingEnvelopes(connId, filters, page, pageSize);
      } else {
        result = await getDeadLetters(connId, filters, page, pageSize);
      }
      items = result.items;
      total = result.total;
    } catch (e) {
      toasts.add(`Failed to load: ${e}`, "error");
    }
  }

  function handleFilter(f: { table: string; status: string; messageType: string }) {
    currentFilters = f;
    currentTable = f.table;
    page = 0;
    loadData();
  }

  $effect(() => {
    if ($activeConnectionId) loadData();
  });
</script>

<div class="p-6">
  <h1 class="text-xl font-semibold mb-6">Message Explorer</h1>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to explore messages.</p>
  {:else}
    <div class="space-y-4">
      <FilterBar onFilter={handleFilter} />
      <EnvelopeTable
        {items}
        table={currentTable}
        {total}
        {page}
        {pageSize}
        onPageChange={(p) => { page = p; loadData(); }}
        onSelect={(item) => { selectedItem = item; }}
      />
    </div>
  {/if}
</div>

<MessageDetail item={selectedItem} onClose={() => { selectedItem = null; }} />
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Explorer view with filtered envelope table and message detail panel"
```

---

## Chunk 6: Frontend — Dead Letters + Nodes Views

This chunk completes the frontend with the Dead Letter Queue and Nodes views. At the end, all five views are functional.

### Task 16: Build Dead Letters view

**Files:**
- Create: `src/lib/stores/deadLetters.ts`
- Create: `src/lib/components/deadletters/DeadLetterTable.svelte`
- Create: `src/lib/components/deadletters/ReplayControls.svelte`
- Modify: `src/lib/views/DeadLetters.svelte`

- [ ] **Step 1: Create dead letters store**

Create `src/lib/stores/deadLetters.ts`:

```typescript
import { writable } from "svelte/store";
import type { DeadLetter, BulkReplayResult } from "../types";
import { getDeadLetters, replayDeadLetter, replayDeadLettersBulk } from "../tauri";
import { toasts } from "./toasts";

export const deadLetterList = writable<DeadLetter[]>([]);
export const deadLetterTotal = writable(0);
export const selectedIds = writable<Set<string>>(new Set());

export async function loadDeadLetters(connectionId: string, page = 0, pageSize = 25, messageType?: string) {
  try {
    const result = await getDeadLetters(connectionId, { message_type: messageType }, page, pageSize);
    deadLetterList.set(result.items);
    deadLetterTotal.set(result.total);
  } catch (e) {
    toasts.add(`Failed to load dead letters: ${e}`, "error");
  }
}

export async function replaySingle(connectionId: string, id: string) {
  try {
    await replayDeadLetter(connectionId, id);
    toasts.add("Message replayed successfully", "success");
    deadLetterList.update((list) => list.filter((d) => d.id !== id));
    deadLetterTotal.update((t) => t - 1);
  } catch (e) {
    toasts.add(`Replay failed: ${e}`, "error");
  }
}

export async function replaySelected(connectionId: string, ids: string[]) {
  try {
    const result = await replayDeadLettersBulk(connectionId, ids);
    if (result.succeeded > 0) {
      toasts.add(`Replayed ${result.succeeded} messages`, "success");
    }
    if (result.failed > 0) {
      toasts.add(`${result.failed} replays failed`, "error");
    }
    // Reload
    const replayedIds = new Set(ids.filter((id) => !result.errors.some((e) => e.id === id)));
    deadLetterList.update((list) => list.filter((d) => !replayedIds.has(d.id)));
    deadLetterTotal.update((t) => t - result.succeeded);
    selectedIds.set(new Set());
  } catch (e) {
    toasts.add(`Bulk replay failed: ${e}`, "error");
  }
}
```

- [ ] **Step 2: Create DeadLetterTable component**

Create `src/lib/components/deadletters/DeadLetterTable.svelte`:

```svelte
<script lang="ts">
  import type { DeadLetter } from "../../types";
  import { selectedIds } from "../../stores/deadLetters";
  import { shortenMessageType, formatRelativeTime } from "../../format";

  interface Props {
    items: DeadLetter[];
    onSelect: (item: DeadLetter) => void;
  }
  let { items, onSelect }: Props = $props();

  function toggleSelect(id: string) {
    selectedIds.update((s) => {
      const next = new Set(s);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  }

  function toggleAll() {
    selectedIds.update((s) => {
      if (s.size === items.length) return new Set();
      return new Set(items.map((i) => i.id));
    });
  }
</script>

<table class="w-full text-sm">
  <thead>
    <tr class="border-b border-[var(--color-border)] text-[var(--color-text-secondary)] text-xs uppercase">
      <th class="px-4 py-3 w-8">
        <input type="checkbox" checked={$selectedIds.size === items.length && items.length > 0} onchange={toggleAll} />
      </th>
      <th class="px-4 py-3 text-left">Message Type</th>
      <th class="px-4 py-3 text-left">Exception</th>
      <th class="px-4 py-3 text-left">Sent</th>
      <th class="px-4 py-3 text-center w-20">Replay</th>
    </tr>
  </thead>
  <tbody>
    {#each items as dl (dl.id)}
      <tr class="border-b border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)]">
        <td class="px-4 py-2">
          <input type="checkbox" checked={$selectedIds.has(dl.id)} onchange={() => toggleSelect(dl.id)} />
        </td>
        <td class="px-4 py-2 cursor-pointer" onclick={() => onSelect(dl)}>
          {shortenMessageType(dl.message_type)}
        </td>
        <td class="px-4 py-2 text-red-400 truncate max-w-64">
          {dl.exception_type ?? "—"}: {dl.exception_message ?? ""}
        </td>
        <td class="px-4 py-2 text-xs text-[var(--color-text-secondary)]">
          {formatRelativeTime(dl.sent_at)}
        </td>
        <td class="px-4 py-2 text-center">
          {#if dl.replayable}
            <span class="text-green-400 text-xs">Yes</span>
          {:else}
            <span class="text-[var(--color-text-secondary)] text-xs">No</span>
          {/if}
        </td>
      </tr>
    {/each}
  </tbody>
</table>
```

- [ ] **Step 3: Create ReplayControls component**

Create `src/lib/components/deadletters/ReplayControls.svelte`:

```svelte
<script lang="ts">
  import { selectedIds, replaySingle, replaySelected } from "../../stores/deadLetters";
  import { activeConnectionId } from "../../stores/connections";

  interface Props {
    items: import("../../types").DeadLetter[];
  }
  let { items }: Props = $props();

  let replaying = $state(false);

  const selectedCount = $derived($selectedIds.size);
  const nonReplayableSelected = $derived(
    items.filter((d) => $selectedIds.has(d.id) && !d.replayable).length
  );

  async function handleBulkReplay() {
    const connId = $activeConnectionId;
    if (!connId) return;
    replaying = true;
    const ids = [...$selectedIds].filter((id) => {
      const dl = items.find((d) => d.id === id);
      return dl?.replayable;
    });
    await replaySelected(connId, ids);
    replaying = false;
  }
</script>

{#if selectedCount > 0}
  <div class="flex items-center gap-3 p-3 bg-[var(--color-surface-overlay)] rounded-lg">
    <span class="text-sm">{selectedCount} selected</span>
    {#if nonReplayableSelected > 0}
      <span class="text-xs text-yellow-400">({nonReplayableSelected} not replayable, will be skipped)</span>
    {/if}
    <button
      onclick={handleBulkReplay}
      disabled={replaying}
      class="px-4 py-1.5 text-sm rounded bg-green-600 hover:bg-green-700 text-white disabled:opacity-50"
    >
      {replaying ? "Replaying..." : "Replay Selected"}
    </button>
  </div>
{/if}
```

- [ ] **Step 4: Build the DeadLetters view**

Replace `src/lib/views/DeadLetters.svelte`:

```svelte
<script lang="ts">
  import { activeConnectionId } from "../stores/connections";
  import { deadLetterList, deadLetterTotal, loadDeadLetters } from "../stores/deadLetters";
  import DeadLetterTable from "../components/deadletters/DeadLetterTable.svelte";
  import ReplayControls from "../components/deadletters/ReplayControls.svelte";
  import MessageDetail from "../components/explorer/MessageDetail.svelte";

  let page = $state(0);
  let selectedItem = $state<any>(null);

  $effect(() => {
    const connId = $activeConnectionId;
    if (connId) loadDeadLetters(connId, page);
  });
</script>

<div class="p-6">
  <h1 class="text-xl font-semibold mb-6">Dead Letter Queue</h1>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view dead letters.</p>
  {:else}
    <div class="space-y-4">
      <ReplayControls items={$deadLetterList} />

      <div class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg overflow-hidden">
        <DeadLetterTable items={$deadLetterList} onSelect={(item) => { selectedItem = item; }} />
      </div>

      <div class="text-xs text-[var(--color-text-secondary)]">{$deadLetterTotal} total</div>
    </div>
  {/if}
</div>

<MessageDetail item={selectedItem} onClose={() => { selectedItem = null; }} />
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add Dead Letters view with replay controls and bulk selection"
```

---

### Task 17: Build Nodes view

**Files:**
- Create: `src/lib/stores/nodes.ts`
- Create: `src/lib/components/nodes/HealthIndicator.svelte`
- Create: `src/lib/components/nodes/NodeTable.svelte`
- Modify: `src/lib/views/Nodes.svelte`

- [ ] **Step 1: Create nodes store**

Create `src/lib/stores/nodes.ts`:

```typescript
import { writable } from "svelte/store";
import type { WolverineNode } from "../types";
import { getNodes } from "../tauri";

export const nodeList = writable<WolverineNode[]>([]);

let pollInterval: ReturnType<typeof setInterval> | null = null;

export async function refreshNodes(connectionId: string) {
  try {
    const nodes = await getNodes(connectionId);
    nodeList.set(nodes);
  } catch {
    // Silently fail — will retry
  }
}

export function startNodePolling(connectionId: string, intervalSecs = 10) {
  stopNodePolling();
  refreshNodes(connectionId);
  pollInterval = setInterval(() => refreshNodes(connectionId), intervalSecs * 1000);
}

export function stopNodePolling() {
  if (pollInterval) {
    clearInterval(pollInterval);
    pollInterval = null;
  }
}
```

- [ ] **Step 2: Create HealthIndicator component**

Create `src/lib/components/nodes/HealthIndicator.svelte`:

```svelte
<script lang="ts">
  import { getNodeHealth } from "../../format";

  interface Props {
    healthCheck: string | null;
  }
  let { healthCheck }: Props = $props();

  const health = $derived(getNodeHealth(healthCheck));

  const colors: Record<string, string> = {
    Healthy: "bg-green-500",
    Warning: "bg-yellow-500",
    Critical: "bg-red-500",
    Unknown: "bg-gray-500",
  };
</script>

<span class="inline-flex items-center gap-1.5 text-xs">
  <span class="w-2.5 h-2.5 rounded-full {colors[health]}"></span>
  {health}
</span>
```

- [ ] **Step 3: Create NodeTable component**

Create `src/lib/components/nodes/NodeTable.svelte`:

```svelte
<script lang="ts">
  import type { WolverineNode } from "../../types";
  import { formatRelativeTime } from "../../format";
  import HealthIndicator from "./HealthIndicator.svelte";

  interface Props {
    nodes: WolverineNode[];
  }
  let { nodes }: Props = $props();
</script>

<div class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg overflow-hidden">
  <table class="w-full text-sm">
    <thead>
      <tr class="border-b border-[var(--color-border)] text-[var(--color-text-secondary)] text-xs uppercase">
        <th class="px-4 py-3 text-left">Node</th>
        <th class="px-4 py-3 text-left">Health</th>
        <th class="px-4 py-3 text-left">URI</th>
        <th class="px-4 py-3 text-left">Version</th>
        <th class="px-4 py-3 text-left">Started</th>
        <th class="px-4 py-3 text-left">Last Check</th>
      </tr>
    </thead>
    <tbody>
      {#each nodes as node (node.id)}
        <tr class="border-b border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)]">
          <td class="px-4 py-2">
            <div class="font-medium">#{node.node_number}</div>
            <div class="text-xs text-[var(--color-text-secondary)]">{node.description ?? "—"}</div>
          </td>
          <td class="px-4 py-2">
            <HealthIndicator healthCheck={node.health_check} />
          </td>
          <td class="px-4 py-2 font-mono text-xs truncate max-w-48">{node.uri ?? "—"}</td>
          <td class="px-4 py-2 text-xs">{node.version ?? "—"}</td>
          <td class="px-4 py-2 text-xs text-[var(--color-text-secondary)]">
            {formatRelativeTime(node.started)}
          </td>
          <td class="px-4 py-2 text-xs text-[var(--color-text-secondary)]">
            {formatRelativeTime(node.health_check)}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  {#if nodes.length === 0}
    <div class="p-4 text-sm text-[var(--color-text-secondary)]">No nodes found.</div>
  {/if}
</div>
```

- [ ] **Step 4: Build the Nodes view**

Replace `src/lib/views/Nodes.svelte`:

```svelte
<script lang="ts">
  import { activeConnectionId } from "../stores/connections";
  import { nodeList, startNodePolling, stopNodePolling } from "../stores/nodes";
  import NodeTable from "../components/nodes/NodeTable.svelte";

  $effect(() => {
    const connId = $activeConnectionId;
    if (connId) {
      startNodePolling(connId);
    }
    return () => stopNodePolling();
  });
</script>

<div class="p-6">
  <h1 class="text-xl font-semibold mb-6">Cluster Nodes</h1>

  {#if !$activeConnectionId}
    <p class="text-[var(--color-text-secondary)]">Select a connection to view nodes.</p>
  {:else}
    <NodeTable nodes={$nodeList} />
  {/if}
</div>
```

- [ ] **Step 5: Verify the full app compiles and all views work**

```bash
npm run tauri dev
```

Expected: All five views render. Dashboard shows counters. Explorer shows paginated envelopes. Dead Letters shows the DLQ with replay. Nodes shows cluster health. Connections allows add/test/remove.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add Nodes view with health indicators and auto-polling"
```

---

### Task 18: Add connection selector to sidebar

**Files:**
- Modify: `src/lib/components/layout/Sidebar.svelte`

- [ ] **Step 1: Add connection dropdown to Sidebar**

Update `src/lib/components/layout/Sidebar.svelte` to add a connection selector at the top, below the title:

Add imports and the selector:

```svelte
<script lang="ts">
  import { currentRoute, navigate } from "../../stores/router";
  import { connections, activeConnectionId } from "../../stores/connections";
  import type { Route } from "../../types";

  // ... existing navItems ...
</script>

<!-- After the title div, before nav -->
{#if $connections.length > 0}
  <div class="px-3 pb-3">
    <select
      value={$activeConnectionId ?? ""}
      onchange={(e) => activeConnectionId.set(e.currentTarget.value || null)}
      class="w-full bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-2 py-1.5 text-sm"
    >
      <option value="">Select connection...</option>
      {#each $connections as conn (conn.config.id)}
        <option value={conn.config.id}>{conn.config.name}</option>
      {/each}
    </select>
  </div>
{/if}
```

- [ ] **Step 2: Commit**

```bash
git add -A
git commit -m "feat: add connection selector to sidebar"
```

---

### Task 19: Final integration — load connections on startup

**Files:**
- Modify: `src/App.svelte`

- [ ] **Step 1: Load saved connections on app mount**

Update `src/App.svelte` to load connections on startup:

```svelte
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import Sidebar from "./lib/components/layout/Sidebar.svelte";
  import ToastContainer from "./lib/components/layout/ToastContainer.svelte";
  import Dashboard from "./lib/views/Dashboard.svelte";
  import Explorer from "./lib/views/Explorer.svelte";
  import DeadLetters from "./lib/views/DeadLetters.svelte";
  import Nodes from "./lib/views/Nodes.svelte";
  import Connections from "./lib/views/Connections.svelte";
  import { currentRoute } from "./lib/stores/router";
  import { connections } from "./lib/stores/connections";
  import { onAlert } from "./lib/tauri";
  import { toasts } from "./lib/stores/toasts";
  import type { ConnectionInfo } from "./lib/types";

  onMount(async () => {
    // Load saved connections
    try {
      const list = await invoke<ConnectionInfo[]>("list_connections");
      connections.set(list);
    } catch {
      // No connections yet
    }

    // Listen for alerts
    await onAlert((event) => {
      toasts.add(event.message, "warning");
    });
  });
</script>

<!-- ... rest unchanged ... -->
```

- [ ] **Step 2: Final build verification**

```bash
npm run tauri build
```

Expected: App builds to a distributable binary.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: load connections on startup and subscribe to alert events"
```

---

## Chunk 7: Review Fixes — Missing Features, Tests, and Correctness

This chunk addresses all issues found during plan review. These tasks can be done in parallel where marked.

### Task 20: Add schema name validation (SQL injection prevention)

**Files:**
- Modify: `src-tauri/src/models/connection.rs`

- [ ] **Step 1: Add validation function to ConnectionConfig**

Add to `src-tauri/src/models/connection.rs`:

```rust
impl ConnectionConfig {
    /// Validate schema name to prevent SQL injection.
    /// Only allows alphanumeric characters and underscores.
    pub fn validate_schema(schema: &str) -> Result<(), String> {
        if schema.is_empty() {
            return Err("Schema name cannot be empty".to_string());
        }
        if !schema.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(format!("Schema name '{}' contains invalid characters. Only alphanumeric and underscores allowed.", schema));
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Call validation in ConnectionManager::add**

In `src-tauri/src/connections/manager.rs`, add at the start of `add()`:

```rust
ConnectionConfig::validate_schema(&config.schema)
    .map_err(|e| AppError::Config(e))?;
```

- [ ] **Step 3: Write test**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_schema_names() {
        assert!(ConnectionConfig::validate_schema("public").is_ok());
        assert!(ConnectionConfig::validate_schema("my_schema").is_ok());
        assert!(ConnectionConfig::validate_schema("Schema123").is_ok());
    }

    #[test]
    fn test_invalid_schema_names() {
        assert!(ConnectionConfig::validate_schema("").is_err());
        assert!(ConnectionConfig::validate_schema("schema; DROP TABLE").is_err());
        assert!(ConnectionConfig::validate_schema("my-schema").is_err());
        assert!(ConnectionConfig::validate_schema("schema.name").is_err());
    }
}
```

- [ ] **Step 4: Commit**

```bash
git add -A
git commit -m "fix: add schema name validation to prevent SQL injection"
```

---

### Task 21: Add missing commands — update_connection, get_message_detail

**Files:**
- Modify: `src-tauri/src/connections/manager.rs`
- Modify: `src-tauri/src/commands/connection_cmds.rs`
- Modify: `src-tauri/src/commands/envelope_cmds.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add update method to ConnectionManager**

Add to `ConnectionManager`:

```rust
pub async fn update(&self, connection_id: &str, updates: ConnectionUpdate) -> Result<(), AppError> {
    let mut conns = self.connections.write().await;
    let managed = conns
        .get_mut(connection_id)
        .ok_or_else(|| AppError::ConnectionNotFound(connection_id.to_string()))?;

    if let Some(name) = updates.name { managed.config.name = name; }
    if let Some(host) = updates.host { managed.config.host = host; }
    if let Some(port) = updates.port { managed.config.port = port; }
    if let Some(database) = updates.database { managed.config.database = database; }
    if let Some(schema) = updates.schema {
        ConnectionConfig::validate_schema(&schema).map_err(|e| AppError::Config(e))?;
        managed.config.schema = schema;
    }
    if let Some(username) = updates.username { managed.config.username = username; }
    if let Some(password) = updates.password { managed.config.password = password; }
    if let Some(ssl_mode) = updates.ssl_mode { managed.config.ssl_mode = ssl_mode; }

    Ok(())
}
```

- [ ] **Step 2: Add update_connection Tauri command**

Add to `connection_cmds.rs`:

```rust
#[tauri::command]
pub async fn update_connection(
    connection_id: String,
    updates: ConnectionUpdate,
    manager: State<'_, ConnectionManager>,
) -> Result<(), AppError> {
    manager.update(&connection_id, updates).await
}
```

- [ ] **Step 3: Add get_message_detail Tauri command**

Add to `envelope_cmds.rs`:

```rust
#[tauri::command]
pub async fn get_message_detail(
    connection_id: String,
    table: String,
    id: String,
    manager: State<'_, ConnectionManager>,
) -> Result<serde_json::Value, AppError> {
    let pool = manager.get_pool(&connection_id).await?;
    let schema = manager.get_schema(&connection_id).await?;
    let client = pool.get().await?;
    let uuid = uuid::Uuid::parse_str(&id).map_err(|e| AppError::Config(e.to_string()))?;

    let table_name = match table.as_str() {
        "incoming" => "wolverine_incoming_envelopes",
        "outgoing" => "wolverine_outgoing_envelopes",
        "dead_letter" => "wolverine_dead_letters",
        _ => return Err(AppError::Config(format!("Unknown table: {}", table))),
    };

    let sql = format!("SELECT * FROM {schema}.{table_name} WHERE id = $1");
    let row = client.query_one(&sql, &[&uuid]).await?;

    // Convert row to JSON
    let mut map = serde_json::Map::new();
    for (i, col) in row.columns().iter().enumerate() {
        let val: serde_json::Value = match col.type_().name() {
            "uuid" => serde_json::json!(row.get::<_, uuid::Uuid>(i).to_string()),
            "int4" => serde_json::json!(row.get::<_, i32>(i)),
            "int8" => serde_json::json!(row.get::<_, i64>(i)),
            "bool" => serde_json::json!(row.get::<_, bool>(i)),
            "bytea" => {
                let bytes: Vec<u8> = row.get(i);
                // Try UTF-8 JSON decode
                match String::from_utf8(bytes.clone()) {
                    Ok(s) if serde_json::from_str::<serde_json::Value>(&s).is_ok() => {
                        serde_json::from_str(&s).unwrap()
                    }
                    _ => serde_json::json!(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes)),
                }
            }
            _ => {
                // Try as string, fall back to null
                row.try_get::<_, String>(i)
                    .map(|s| serde_json::json!(s))
                    .unwrap_or(serde_json::Value::Null)
            }
        };
        map.insert(col.name().to_string(), val);
    }

    Ok(serde_json::Value::Object(map))
}
```

- [ ] **Step 4: Register both commands in lib.rs invoke_handler**

Add to the `generate_handler!` macro:

```rust
commands::connection_cmds::update_connection,
commands::envelope_cmds::get_message_detail,
```

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: add update_connection and get_message_detail commands"
```

---

### Task 22: Add date range filters to queries

**Files:**
- Modify: `src-tauri/src/queries/envelopes.rs`
- Modify: `src-tauri/src/queries/dead_letters.rs`

- [ ] **Step 1: Add date filtering to query_incoming**

In `query_incoming`, add after the message_type filter:

```rust
if let Some(ref from) = filters.date_from {
    conditions.push(format!("execution_time >= ${}", idx));
    params.push(Box::new(from.clone()));
    idx += 1;
}
if let Some(ref to) = filters.date_to {
    conditions.push(format!("execution_time <= ${}", idx));
    params.push(Box::new(to.clone()));
    idx += 1;
}
```

Apply the same pattern to `query_outgoing` (using `deliver_by`) and `query_dead_letters` (using `sent_at`).

- [ ] **Step 2: Add date range pickers to FilterBar.svelte**

Add to `FilterBar.svelte`:

```svelte
<input
  type="datetime-local"
  bind:value={dateFrom}
  class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm"
  placeholder="From"
/>
<input
  type="datetime-local"
  bind:value={dateTo}
  class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm"
  placeholder="To"
/>
```

Pass `dateFrom` and `dateTo` in the filter callback.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: add date range filtering to envelope and dead letter queries"
```

---

### Task 23: Add ThroughputChart component

**Files:**
- Create: `src/lib/components/dashboard/ThroughputChart.svelte`
- Modify: `src/lib/views/Dashboard.svelte`

- [ ] **Step 1: Create ThroughputChart using layerchart**

Create `src/lib/components/dashboard/ThroughputChart.svelte`:

```svelte
<script lang="ts">
  import { Chart, Svg, Area, Axis } from "layerchart";
  import type { ThroughputPoint } from "../../types";

  interface Props {
    data: ThroughputPoint[];
  }
  let { data }: Props = $props();

  const chartData = $derived(
    data.map((d) => ({
      date: new Date(d.timestamp),
      incoming: d.incoming,
      outgoing: d.outgoing,
    }))
  );
</script>

<div class="bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg p-4">
  <div class="text-sm font-semibold mb-3">Throughput (last 30 min)</div>
  {#if chartData.length > 1}
    <div class="h-48">
      <Chart data={chartData} x="date" y="incoming">
        <Svg>
          <Area color="var(--color-status-incoming)" opacity={0.3} />
        </Svg>
      </Chart>
    </div>
  {:else}
    <div class="h-48 flex items-center justify-center text-sm text-[var(--color-text-secondary)]">
      Collecting throughput data...
    </div>
  {/if}
</div>
```

- [ ] **Step 2: Wire into Dashboard.svelte**

Add import and render between counter cards and LiveFeed:

```svelte
<ThroughputChart data={$stats.throughput} />
```

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: add ThroughputChart component to dashboard"
```

---

### Task 24: Add DLQ filters and node cross-reference

**Files:**
- Modify: `src/lib/views/DeadLetters.svelte`
- Modify: `src/lib/components/nodes/NodeTable.svelte`

- [ ] **Step 1: Add filter bar to Dead Letters view**

Add filter controls to `DeadLetters.svelte` above the table:

```svelte
<div class="flex items-center gap-3 p-4 bg-[var(--color-surface-raised)] border border-[var(--color-border)] rounded-lg">
  <input bind:value={filterMessageType} placeholder="Message type..."
    oninput={() => loadDeadLetters($activeConnectionId!, page, 25, filterMessageType || undefined)}
    class="flex-1 bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm" />
  <select bind:value={filterReplayable}
    onchange={() => loadFiltered()}
    class="bg-[var(--color-surface)] border border-[var(--color-border)] rounded px-3 py-1.5 text-sm">
    <option value="">All</option>
    <option value="true">Replayable</option>
    <option value="false">Not Replayable</option>
  </select>
</div>
```

Note: Full exception_type and replayable filtering requires extending the backend `EnvelopeFilters` struct and queries. Add `exception_type: Option<String>` and `replayable: Option<bool>` fields.

- [ ] **Step 2: Add owner_id display to NodeTable**

Update `NodeTable.svelte` to show the `capabilities` field and note the owner_id mapping:

Add a column header `<th>Capabilities</th>` and cell:
```svelte
<td class="px-4 py-2 text-xs truncate max-w-40">{node.capabilities ?? "—"}</td>
```

For owner_id cross-reference, add a note in the UI: nodes can be referenced by their `node_number` which maps to `owner_id` in envelope tables.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "feat: add DLQ filters and node capabilities display"
```

---

### Task 25: Add frontend test infrastructure and basic tests

**Files:**
- Create: `src/tests/setup.ts`
- Create: `src/tests/stores/router.test.ts`
- Create: `src/tests/components/HealthIndicator.test.ts`
- Modify: `package.json` (add vitest)

- [ ] **Step 1: Install test dependencies**

```bash
npm install -D vitest @testing-library/svelte jsdom
```

- [ ] **Step 2: Create test setup with Tauri mocks**

Create `src/tests/setup.ts`:

```typescript
import { vi } from "vitest";

// Mock Tauri API
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
```

Add to `vite.config.ts`:

```typescript
test: {
  environment: "jsdom",
  setupFiles: ["./src/tests/setup.ts"],
},
```

- [ ] **Step 3: Write router store test**

Create `src/tests/stores/router.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { get } from "svelte/store";
import { currentRoute, navigate } from "../../lib/stores/router";

describe("router store", () => {
  it("starts at dashboard", () => {
    expect(get(currentRoute)).toBe("dashboard");
  });

  it("navigates to a new route", () => {
    navigate("connections");
    expect(get(currentRoute)).toBe("connections");
    navigate("dashboard"); // reset
  });
});
```

- [ ] **Step 4: Write format utility tests**

Create `src/tests/format.test.ts`:

```typescript
import { describe, it, expect } from "vitest";
import { decodeBody, shortenMessageType, getNodeHealth } from "../lib/format";

describe("decodeBody", () => {
  it("decodes valid JSON body", () => {
    const json = '{"key":"value"}';
    const bytes = Array.from(new TextEncoder().encode(json));
    const result = decodeBody(bytes);
    expect(result.type).toBe("json");
  });

  it("falls back to hex for non-JSON", () => {
    const result = decodeBody([0xff, 0xfe, 0x00]);
    expect(result.type).toBe("hex");
  });
});

describe("shortenMessageType", () => {
  it("extracts last segment", () => {
    expect(shortenMessageType("MyApp.Commands.PlaceOrder")).toBe("PlaceOrder");
  });
});

describe("getNodeHealth", () => {
  it("returns Healthy for recent check", () => {
    const recent = new Date().toISOString();
    expect(getNodeHealth(recent)).toBe("Healthy");
  });

  it("returns Unknown for null", () => {
    expect(getNodeHealth(null)).toBe("Unknown");
  });
});
```

- [ ] **Step 5: Run tests**

```bash
npm run test
```

Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add -A
git commit -m "feat: add frontend test infrastructure with router and format utility tests"
```

---

### Task 26: Fix EnvelopeTable dead letter attempts and debounce stats refresh

**Files:**
- Modify: `src/lib/components/explorer/EnvelopeTable.svelte`
- Modify: `src/lib/stores/dashboard.ts`

- [ ] **Step 1: Fix attempts column for dead letters**

In `EnvelopeTable.svelte`, change the attempts column to handle dead letters:

```svelte
<td class="px-4 py-2 text-right">{'attempts' in item ? item.attempts : '—'}</td>
```

- [ ] **Step 2: Debounce stats refresh in dashboard store**

In `src/lib/stores/dashboard.ts`, add debounce to the envelope change handler:

```typescript
let refreshTimeout: ReturnType<typeof setTimeout> | null = null;

function debouncedRefreshStats(connectionId: string) {
  if (refreshTimeout) clearTimeout(refreshTimeout);
  refreshTimeout = setTimeout(() => refreshStats(connectionId), 1000);
}
```

Use `debouncedRefreshStats` instead of `refreshStats` in the `onEnvelopeChange` callback.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "fix: handle missing attempts field for dead letters, debounce stats refresh"
```

---

### Task 27: Add listConnections to tauri.ts wrapper

**Files:**
- Modify: `src/lib/tauri.ts`
- Modify: `src/lib/stores/connections.ts`

- [ ] **Step 1: Add listConnections wrapper**

Add to `src/lib/tauri.ts`:

```typescript
export const listConnections = () =>
  invoke<ConnectionInfo[]>("list_connections");
```

- [ ] **Step 2: Use it in connections store and App.svelte**

Replace raw `invoke` calls with `listConnections()` in both `src/lib/stores/connections.ts` and `src/App.svelte`.

- [ ] **Step 3: Commit**

```bash
git add -A
git commit -m "refactor: use typed listConnections wrapper instead of raw invoke"
```

---

## Summary

| Chunk | Tasks | What it delivers |
|-------|-------|------------------|
| 1 | 1–4 | Project scaffold, Rust models, TS types, app shell with sidebar |
| 2 | 5–8 | Connection management (backend + frontend), config persistence |
| 3 | 9–11 | Trigger installer, envelope/dead letter queries, node/dashboard queries |
| 4 | 12–13 | LISTEN/NOTIFY real-time monitor, alert engine with dead letter detection |
| 5 | 14–15 | Dashboard view (counters, live feed), Message Explorer (filters, pagination, detail) |
| 6 | 16–19 | Dead Letters view (replay), Nodes view (health), sidebar connection selector, startup integration |
| 7 | 20–27 | Review fixes: schema validation, missing commands, date filters, ThroughputChart, DLQ filters, frontend tests, bug fixes |

**Total: 27 tasks, ~130 steps.** Each task produces a working commit. The app is functional after each chunk.
