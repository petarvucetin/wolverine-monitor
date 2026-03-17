# Wolverine Monitor — Design Specification

## Overview

A desktop application for real-time monitoring of Wolverine.FX messaging infrastructure backed by PostgreSQL. Provides an ops dashboard, message inspector, dead letter queue management, node health monitoring, and configurable alerting.

**Stack:** Tauri 2 (Rust backend) + Svelte 5 + TypeScript + Tailwind CSS

## Problem Statement

Wolverine.FX persists messages in PostgreSQL using the transactional inbox/outbox pattern. There is no dedicated tooling for:

- Visualizing message throughput and queue depths in real time
- Quickly detecting and triaging dead letters
- Inspecting individual message payloads and metadata
- Monitoring cluster node health
- Managing multiple Wolverine databases from a single interface

This tool fills that gap as a lightweight, self-contained desktop application.

## Architecture

```
┌─────────────────────────────────────┐
│  Svelte 5 Frontend                  │
│  ├─ Dashboard (charts, counters)    │
│  ├─ Message Explorer                │
│  ├─ Dead Letter Queue viewer        │
│  ├─ Nodes viewer                    │
│  └─ Connection Manager              │
│         ↕ Tauri invoke / events     │
├─────────────────────────────────────┤
│  Tauri Rust Backend                 │
│  ├─ Connection Pool (per DB)        │
│  ├─ LISTEN/NOTIFY subscriber        │
│  ├─ Query engine (envelope tables)  │
│  ├─ Alert engine (dead letters)     │
│  └─ Trigger installer               │
│         ↕ tokio-postgres            │
├─────────────────────────────────────┤
│  PostgreSQL (one or many)           │
│  └─ Wolverine envelope tables       │
└─────────────────────────────────────┘
```

### Why Tauri-Native Backend

- Single binary (~10MB), no runtime dependencies
- Rust handles multiple persistent PostgreSQL connections efficiently with tokio
- Tauri's event system maps naturally to pushing real-time updates to the frontend
- Low memory footprint compared to Electron

## Data Model

### Wolverine PostgreSQL Tables

All tables live in a configurable schema (default varies by Wolverine configuration). The schema name is stored per connection profile.

#### `wolverine_incoming_envelopes`

| Column | Type | Purpose |
|--------|------|---------|
| `id` | UUID (PK) | Message identifier |
| `status` | varchar | `Incoming`, `Scheduled`, `Handled` |
| `owner_id` | int | Node that owns this message |
| `execution_time` | timestamptz | Scheduled execution time |
| `attempts` | int | Processing retry count |
| `body` | bytea | Serialized message payload |
| `message_type` | varchar | .NET type name |
| `received_at` | varchar | Destination endpoint URI |
| `keep_until` | timestamptz | TTL expiration |
| `timestamp` | timestamptz | When persisted (if `InboxStaleTime` configured) |

Statuses: `Incoming` (awaiting processing), `Scheduled` (deferred), `Handled` (successfully processed).

Optional partitioning by status (Wolverine 5.3+).

#### `wolverine_outgoing_envelopes`

| Column | Type | Purpose |
|--------|------|---------|
| `id` | UUID (PK) | Message identifier |
| `owner_id` | int | Sending node |
| `destination` | varchar | Target endpoint URI |
| `deliver_by` | timestamptz | Expiration deadline |
| `body` | bytea | Serialized payload |
| `attempts` | int | Send attempts |
| `message_type` | varchar | .NET type name |
| `timestamp` | timestamptz | When persisted (if `OutboxStaleTime` configured) |

#### `wolverine_dead_letters`

| Column | Type | Purpose |
|--------|------|---------|
| `id` | UUID (PK) | Message identifier |
| `execution_time` | timestamptz | Original execution time |
| `body` | bytea | Serialized payload |
| `message_type` | varchar | .NET type name |
| `received_at` | varchar | Destination endpoint |
| `source` | varchar | Origin |
| `exception_type` | varchar | .NET exception type |
| `exception_message` | varchar | Error details |
| `sent_at` | timestamptz | When originally sent |
| `replayable` | bool | Whether the message can be retried |
| `expires` | timestamptz | DLQ expiration (if enabled) |

#### `wolverine_nodes`

| Column | Type | Purpose |
|--------|------|---------|
| `id` | UUID (PK) | Node identifier |
| `node_number` | int | Assigned node number |
| `description` | varchar | Node description |
| `uri` | varchar | Node URI |
| `started` | timestamptz | When node started |
| `health_check` | timestamptz | Last health check time |
| `version` | varchar | Wolverine version |
| `capabilities` | varchar | Node capabilities |

### Message Body Serialization

Wolverine serializes message bodies using JSON (Newtonsoft.Json) by default. The `body` column contains UTF-8 encoded JSON bytes.

**Decoding strategy:**
1. Attempt to parse `body` as UTF-8 JSON. If valid, pretty-print it in the UI.
2. If UTF-8 decoding fails or the result is not valid JSON, display as a hex dump with a Base64 copy button.
3. The `message_type` column provides the .NET type name for context (displayed alongside the body).

This handles the common case (JSON) and gracefully degrades for custom serializers (MessagePack, protobuf, etc.).

## Real-Time Update Strategy: PostgreSQL LISTEN/NOTIFY

### Trigger Installation

The monitor installs lightweight PostgreSQL triggers on each envelope table to emit notifications on data changes. Triggers are installed with user confirmation on first connection and can be removed on disconnect.

**Required PostgreSQL permissions:**
- `SELECT` on all Wolverine envelope and node tables
- `CREATE FUNCTION` and `CREATE TRIGGER` on the Wolverine schema (for trigger installation only)
- `INSERT` on `wolverine_incoming_envelopes` (for dead letter replay)
- `DELETE` on `wolverine_dead_letters` (for dead letter replay)

**Trigger function template:**

```sql
CREATE OR REPLACE FUNCTION {schema}.wolverine_monitor_notify()
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
```

Applied to each table with `AFTER INSERT OR UPDATE OR DELETE` triggers.

### NOTIFY Channels

Channel names are prefixed with the schema name to avoid collisions when multiple Wolverine schemas exist in the same PostgreSQL instance:

- `{schema}_wolverine_incoming_envelopes_changed`
- `{schema}_wolverine_outgoing_envelopes_changed`
- `{schema}_wolverine_dead_letters_changed`

**Note:** The `wolverine_nodes` table is **not** monitored via NOTIFY. Node state changes infrequently (heartbeats every few seconds), and the Nodes View already polls every 10s, which is sufficient. Adding NOTIFY triggers to the nodes table would add unnecessary trigger overhead for no latency benefit.

### Notification Payload

```json
{
  "op": "INSERT",
  "id": "a1b2c3d4-...",
  "message_type": "MyApp.Commands.PlaceOrder"
}
```

### Event Flow

```
PostgreSQL                 Rust Backend                    Svelte Frontend
─────────                  ────────────                    ───────────────

INSERT/UPDATE:
Table change        ──→  NOTIFY trigger fires
                         ──→  LISTEN connection receives
                              ──→  Fetch full row by ID
                                   ──→  Emit Tauri event
                                        ──→  Svelte store updates
                                             ──→  UI reactively renders

DELETE:
Row deleted         ──→  NOTIFY trigger fires (with ID from OLD)
                         ──→  LISTEN connection receives
                              ──→  Emit Tauri event with "op":"DELETE"
                                   (no re-fetch — row is gone)
                                   ──→  Svelte store removes item by ID
                                        ──→  UI reactively renders
```

## Rust Backend Modules

### Connection Manager (`src-tauri/src/connections/`)

- Manages multiple named PostgreSQL connections
- Each connection maintains:
  - A `tokio-postgres` client pool for queries (via `deadpool-postgres`)
  - A dedicated LISTEN connection for NOTIFY subscriptions
- Connection configs stored persistently using `tauri-plugin-store` (encrypts sensitive fields)
- SSL/TLS: configurable per connection (off, prefer, require, require + verify-ca). Default: `prefer`.
- Automatic reconnection with exponential backoff (1s, 2s, 4s, 8s, max 30s)
- Schema name configurable per connection

### Envelope Monitor (`src-tauri/src/monitor/`)

- Subscribes to NOTIFY channels per connection
- On INSERT/UPDATE notification: fetches changed row by ID, emits Tauri event to frontend
- On DELETE notification: emits Tauri event with delete payload directly (no re-fetch)
- Periodically polls aggregate stats (counts by status, throughput/min) every 5s (configurable)
- Maintains rolling window of the last 500 messages in memory for dashboard feed (configurable via `settings.json`)

### Alert Engine (`src-tauri/src/alerts/`)

- Watches for new dead letter entries via NOTIFY
- Configurable alert rules:
  - Alert on any dead letter
  - Alert on dead letters matching specific message types
  - Alert when incoming queue depth exceeds threshold N
- Delivers alerts via:
  - System notifications (Tauri notification API)
  - In-app toast notifications

### Query Engine (`src-tauri/src/queries/`)

Tauri `invoke` commands exposed to the frontend:

| Command | Parameters | Returns |
|---------|-----------|---------|
| `get_incoming_envelopes` | `connection_id, filters, page, page_size` | Paginated envelope list |
| `get_outgoing_envelopes` | `connection_id, filters, page, page_size` | Paginated envelope list |
| `get_dead_letters` | `connection_id, filters, page, page_size` | Paginated dead letter list |
| `get_dashboard_stats` | `connection_id` | Aggregate counts, throughput |
| `get_message_detail` | `connection_id, table, id` | Full envelope with decoded body |
| `get_nodes` | `connection_id` | List of active Wolverine nodes |
| `replay_dead_letter` | `connection_id, id` | Success/failure |
| `replay_dead_letters_bulk` | `connection_id, ids: Vec<Uuid>` | Batch result (succeeded/failed counts) |
| `add_connection` | `name, host, port, database, schema, username, password, ssl_mode` | Connection ID |
| `update_connection` | `connection_id, name?, host?, port?, database?, schema?, username?, password?, ssl_mode?` (partial update — only provided fields are changed) | Success/failure |
| `remove_connection` | `connection_id` | Success/failure |
| `test_connection` | `host, port, database, username, password, ssl_mode` | Success/failure with error details |
| `install_triggers` | `connection_id` | Success/failure |
| `uninstall_triggers` | `connection_id` | Success/failure |

### Dead Letter Replay Mechanism

Replaying a dead letter performs these operations in a single transaction:

1. Read the dead letter row from `wolverine_dead_letters`
2. Verify `replayable = true` (reject if false)
3. INSERT into `wolverine_incoming_envelopes` with:
   - Same `id`
   - `status` = `'Incoming'`
   - `owner_id` = `0` (unassigned — Wolverine's durability agent will pick it up)
   - `execution_time` = `NULL` (process immediately)
   - `attempts` = `0` (reset retry count)
   - `body` = copied from dead letter
   - `message_type` = copied from dead letter
   - `received_at` = copied from dead letter
   - `keep_until` = `NULL`
4. DELETE the row from `wolverine_dead_letters`

Bulk replay uses per-message transactions with a summary result: each message is replayed independently, and the response reports `{ succeeded: N, failed: M, errors: [{id, reason}] }`. This avoids one bad message blocking the entire batch. The UI pre-filters bulk selection to exclude messages with `replayable = false` and warns the user if any are selected.

### Trigger Installer (`src-tauri/src/triggers/`)

- SQL templates for NOTIFY triggers with schema parameterization
- Install/uninstall commands
- Idempotent — checks if triggers already exist before installing
- Reports clear error if the connected user lacks CREATE FUNCTION / CREATE TRIGGER permissions

## Frontend Views

### Dashboard View (landing page)

Real-time overview per connected database:

- **Connection selector** dropdown at top
- **Counter cards** for incoming, outgoing, and dead letter counts with sparkline trends and per-minute throughput
- **Throughput chart** showing message flow over the last 30 minutes (line chart using `layerchart`)
- **Live message feed** auto-scrolling table of recent messages, color-coded by status

### Message Explorer View

Paginated table with filtering:

- Filter by: table (incoming/outgoing/dead), status, message type, date range
- Columns: ID, message type, status, attempts, timestamps
- Click row to open detail panel showing full metadata and decoded body

### Dead Letter Queue View

Focused failure management:

- Table of dead letters with exception type/message prominently displayed
- Per-message "Replay" button (moves back to incoming for reprocessing)
- Bulk select + replay
- Filter by exception type, message type, replayable flag

### Nodes View

Cluster health monitoring:

- Table of active nodes: node number, description, URI, started time, last health check, version, capabilities
- Health indicator per node:
  - **Green:** last health check < 30 seconds ago
  - **Yellow:** last health check 30s–120s ago
  - **Red:** last health check > 120s ago or node unreachable
  - Thresholds configurable in `settings.json`
- Cross-reference: which node owns which messages (maps `owner_id` in envelopes to node numbers). If an `owner_id` doesn't match any known node (e.g., node not yet loaded or node removed), display the raw `owner_id` number.
- Auto-refreshes every 10s (configurable)

### Connections View

Database management:

- Add/edit/remove PostgreSQL connections (name, host, port, database, schema, username, password, SSL mode)
- Connection status indicator (connected/disconnected/error)
- Test connection button
- Install/uninstall NOTIFY triggers per connection

## Frontend State Management

Svelte writable stores per connection:

- `dashboardStats` — aggregate counts, throughput data
- `recentMessages` — rolling buffer of last 500 messages for live feed
- `deadLetters` — current dead letter list
- `nodes` — active Wolverine nodes
- `connectionManager` — all connections and their status

All stores updated reactively via Tauri events — no polling from the frontend.

**Routing:** Tauri serves files locally via a custom protocol (`tauri://`), so the History API works natively without a server. Use Svelte 5's built-in `{#snippet}` blocks with a simple reactive `currentRoute` store for navigation between the five views. No routing library needed — the app has a flat set of views with no nested routes or URL parameters.

## Error Handling

### Connection Errors

- Failed initial connection: display error message in Connections view, do not retry automatically
- Dropped connection: automatic reconnection with exponential backoff (1s, 2s, 4s, 8s, max 30s). Show "Reconnecting..." status in UI. After 5 failed attempts, show "Connection lost" and stop retrying until user manually reconnects.
- LISTEN connection dropped: same reconnection strategy; re-subscribe to all NOTIFY channels on reconnect

### Query Errors

- SQL query failures: return structured error to frontend via Tauri invoke error. Frontend displays error toast with the message. Logged to Tauri's log file.
- Timeout: queries use a 10s timeout (configurable). On timeout, return a timeout-specific error.

### Notification Handling Errors

- Row fetch after NOTIFY fails (row deleted between notification and fetch): silently skip for INSERT/UPDATE; this is a benign race condition. The next periodic stats poll will correct any drift.
- Malformed NOTIFY payload: log warning, skip the notification.

### Body Deserialization Errors

- If `body` bytes cannot be decoded as UTF-8 or parsed as JSON: display hex dump in the UI with a "Copy Base64" button. No error toast — this is a graceful fallback, not a failure.

### Trigger Installation Errors

- Permission denied: display clear message explaining which PostgreSQL permissions are needed (CREATE FUNCTION, CREATE TRIGGER on schema). Offer to copy the trigger SQL to clipboard so a DBA can install it manually.
- Trigger already exists: no-op, report success.

### Dead Letter Replay Errors

- Non-replayable message: reject with clear message ("Message is not marked as replayable")
- Transaction failure: roll back, report the SQL error. No partial state changes.

## Configuration & Persistence

Stored in Tauri's `app_data_dir`:

| File | Version | Purpose |
|------|---------|---------|
| `connections.json` | 1 | Saved connection profiles (credentials encrypted via `tauri-plugin-store`) |
| `settings.json` | 1 | User preferences: polling interval, rolling window size, node health thresholds, theme, window size |
| `alert_rules.json` | 1 | Configurable alert conditions |

Each config file includes a `version` field. On startup, the app checks the version and applies sequential migration functions (defined in `src-tauri/src/config/migrations.rs`). Each migration transforms the JSON structure from version N to N+1. Migrations are applied in order until the config matches the current app version.

**Trigger versioning:** The installed trigger function includes a comment with the monitor version. On connection, the app checks the installed trigger version against its current version and offers to upgrade if they differ.

## Security

### Credential Storage

All database credentials are stored using `tauri-plugin-store` with encryption at rest. Connection passwords are never logged or included in error messages.

### Database Permissions

Minimum required PostgreSQL role permissions:

| Permission | Purpose |
|-----------|---------|
| `SELECT` on envelope + node tables | Reading messages and node state |
| `INSERT` on `wolverine_incoming_envelopes` | Dead letter replay |
| `DELETE` on `wolverine_dead_letters` | Dead letter replay |
| `USAGE` on schema | Accessing tables in the configured schema |
| `CREATE` on schema (optional) | Installing NOTIFY triggers. If unavailable, the app offers manual trigger SQL. |

### SSL/TLS

Configurable per connection with four modes: `disable`, `prefer` (default), `require`, `verify-ca`. The connection form validates that a CA certificate file is provided when `verify-ca` is selected.

## Testing Strategy

### Rust Backend

- **Unit tests:** Test query building, notification payload parsing, alert rule matching, body deserialization logic, and trigger SQL generation. Mock the database layer using trait abstractions.
- **Integration tests:** Use `testcontainers-rs` with a PostgreSQL container to test:
  - Full LISTEN/NOTIFY flow (install trigger → insert row → receive notification)
  - Dead letter replay transactions
  - Connection lifecycle (connect, disconnect, reconnect)
  - Trigger installation and uninstallation

### Frontend

- **Component tests:** Use `@testing-library/svelte` to test each view component with mocked Tauri invoke responses.
- **Store tests:** Unit test Svelte stores in isolation — verify they update correctly when Tauri events are emitted.

### End-to-End

- Use Tauri's `tauri-driver` (WebDriver-based) for E2E tests against a running app with a real PostgreSQL database.
- Priority E2E scenarios: add connection → see dashboard → trigger dead letter → see alert → replay.

## Key Dependencies

### Rust (Cargo)

- `tauri` 2.x — desktop shell
- `tokio-postgres` — async PostgreSQL client
- `deadpool-postgres` — connection pooling
- `serde` / `serde_json` — serialization
- `tauri-plugin-notification` — system notifications
- `tauri-plugin-store` — encrypted persistent storage
- `tauri-plugin-log` — structured logging (log levels: error, warn, info, debug; output to app_log_dir)
- `tracing` / `tracing-subscriber` — Rust-side structured logging
- `testcontainers` — integration test infrastructure

### Frontend (npm)

- `svelte` 5 — UI framework
- `typescript` — type safety
- `tailwindcss` 4 — styling
- `@tauri-apps/api` — Tauri bridge
- `layerchart` — charting
- (no routing library — simple reactive store-based navigation, see Frontend State Management)

## Styling

- **Dark theme** by default (ops tools convention)
- **Tailwind CSS** for utility-first styling
- Status color coding:
  - Green: handled/success
  - Blue: incoming/scheduled
  - Orange: outgoing/pending
  - Red: dead letter/error

## Scope Boundaries

**In scope for v1:**
- All five views (Dashboard, Message Explorer, Dead Letter Queue, Nodes, Connections)
- Multi-database support
- LISTEN/NOTIFY real-time updates
- Dead letter replay (single + bulk)
- System + in-app alerts for dead letters
- Dark theme
- SSL/TLS support

**Out of scope for v1:**
- Light theme toggle
- Message body editing
- Custom dashboard layouts
- Historical analytics / data export
- Multi-tenancy (per-tenant databases)
- Wolverine queue transport tables (`wolverine_queues` schema)
