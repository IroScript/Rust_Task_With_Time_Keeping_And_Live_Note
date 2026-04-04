# 🦀 Pure Rust Backend — Complete Development Guideline
### Real-Time, Local-First, Massive-Scale Notes & Documents Platform

> **Scope**: A production-grade Rust backend serving trillions of notes/documents (~50,000 TB), with instant access, real-time mobile sync (WebSocket), secure cloud storage (PostgreSQL), local caching (SQLite/JSON), and automatic local eviction.

> **Date**: February 2026 | **Status**: Verified against latest ecosystem (Axum 0.8.x, Tokio 1.42+, Automerge 3.x, Yrs)

---

## Table of Contents

1. [Core Philosophy & Goals](#1-core-philosophy--goals)
2. [System Architecture Overview](#2-system-architecture-overview)
3. [Technology Stack (Verified 2026)](#3-technology-stack-verified-2026)
4. [Cloud Storage — PostgreSQL & Distributed SQL](#4-cloud-storage--postgresql--distributed-sql)
5. [Local Storage — SQLite / JSON with Auto-Eviction](#5-local-storage--sqlite--json-with-auto-eviction)
6. [Real-Time Sync — WebSocket & Live Presence](#6-real-time-sync--websocket--live-presence)
7. [CRDT — Conflict-Free Offline Merge](#7-crdt--conflict-free-offline-merge)
8. [Security — Eliminating Password Leak Risks](#8-security--eliminating-password-leak-risks)
9. [Data Partitioning & Tiered Storage](#9-data-partitioning--tiered-storage)
10. [Search & Indexing at Scale](#10-search--indexing-at-scale)
11. [Mobile Readiness & Future-Proofing](#11-mobile-readiness--future-proofing)
12. [Horizontal Scaling & Deployment](#12-horizontal-scaling--deployment)
13. [Observability & Reliability](#13-observability--reliability)
14. [Project Structure & Code Examples](#14-project-structure--code-examples)
15. [Development Phases & Roadmap](#15-development-phases--roadmap)

---

## 1. Core Philosophy & Goals

### Why Pure Rust?

| Concern | Rust Advantage |
|---|---|
| **Memory safety** | Zero-cost abstractions, no GC pauses — critical for real-time sync |
| **Performance** | Near-C speed; predictable latency for sub-second reads |
| **Concurrency** | Tokio async runtime handles millions of concurrent WebSocket connections |
| **Cross-platform** | Single codebase compiles to server, WASM, mobile (via UniFFI/C bindings) |
| **Reliability** | Compiler catches data races at compile time |

### Non-Negotiable Goals

1. **Sub-second read latency** — users click, content appears instantly
2. **Real-time sync** — "someone is typing..." presence on mobile/desktop
3. **Offline-first** — full local functionality, sync when online
4. **Zero credential exposure** — no DB passwords in client apps, ever
5. **Auto-eviction** — local cache stays lean (≤1 GB), cloud is authoritative
6. **Future mobile** — architecture ready for iOS/Android apps from day one

---

## 2. System Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    CLIENT DEVICES                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐   │
│  │ Desktop  │  │  Mobile  │  │  Web (future WASM)   │   │
│  │ (egui)   │  │ (future) │  │                      │   │
│  └────┬─────┘  └────┬─────┘  └──────────┬───────────┘   │
│       │              │                   │               │
│  ┌────┴──────────────┴───────────────────┴────┐          │
│  │         LOCAL LAYER (per device)            │          │
│  │  SQLite cache + JSON snapshots + CRDT state │          │
│  │  Auto-eviction manager (≤1 GB threshold)    │          │
│  └────────────────────┬───────────────────────┘          │
└───────────────────────┼──────────────────────────────────┘
                        │ WebSocket / gRPC (TLS 1.3)
                        ▼
┌───────────────────────────────────────────────────────────┐
│                 RUST BACKEND CLUSTER                       │
│                                                           │
│  ┌─────────────────┐  ┌──────────────────┐                │
│  │  API Gateway     │  │  Auth Service    │                │
│  │  (axum 0.8.x)   │  │  (JWT/OAuth)     │                │
│  │  + WebSocket     │  │  + Vault client  │                │
│  └────────┬────────┘  └────────┬─────────┘                │
│           │                    │                          │
│  ┌────────┴────────────────────┴─────────┐                │
│  │         SYNC ENGINE                    │                │
│  │  CRDT merge (yrs/automerge)            │                │
│  │  Delta compression + sequence tracking │                │
│  │  Presence broadcast (typing, cursors)  │                │
│  └────────┬──────────────────────────────┘                │
│           │                                               │
│  ┌────────┴──────────────────────────────────────┐        │
│  │              STORAGE LAYER                     │        │
│  │                                                │        │
│  │  ┌──────────────┐  ┌────────────┐  ┌────────┐ │        │
│  │  │ PostgreSQL   │  │ Object     │  │ Search │ │        │
│  │  │ (metadata +  │  │ Storage    │  │ Index  │ │        │
│  │  │  pointers)   │  │ (S3/MinIO) │  │(Meili/ │ │        │
│  │  │ CockroachDB  │  │ (blobs)    │  │ Tantivy│ │        │
│  │  └──────────────┘  └────────────┘  └────────┘ │        │
│  └───────────────────────────────────────────────┘        │
│                                                           │
│  ┌────────────────────────────────────┐                   │
│  │  Message Bus (NATS / Kafka)        │                   │
│  │  Change-log, event sourcing, fanout│                   │
│  └────────────────────────────────────┘                   │
│                                                           │
│  ┌────────────────────────────────────┐                   │
│  │  Observability (OpenTelemetry,     │                   │
│  │  Prometheus, Grafana, Loki)        │                   │
│  └────────────────────────────────────┘                   │
└───────────────────────────────────────────────────────────┘
```

---

## 3. Technology Stack (Verified 2026)

### Core Rust Crates

| Layer | Crate | Version / Notes |
|---|---|---|
| **Async Runtime** | `tokio` | 1.42+ with `full` feature |
| **HTTP/WS Gateway** | `axum` | 0.8.x (built by Tokio team, Tower middleware) |
| **WebSocket** | `tokio-tungstenite` | via axum's built-in WS support |
| **DB Access** | `sqlx` | Async, compile-time checked queries, PostgreSQL driver |
| **ORM (optional)** | `sea-orm` | If you prefer ORM patterns over raw SQL |
| **CRDT** | `yrs` (Yjs Rust port) | Battle-tested, JS/Swift/Kotlin bindings via WASM/UniFFI |
| **CRDT (alt)** | `automerge` | v3.x — reduced memory, JSON-like model, C/Swift bindings |
| **Serialization** | `serde` + `serde_json` | Universal Rust serialization |
| **Message Bus** | `async-nats` | NATS client; or `rdkafka` for Kafka |
| **Object Storage** | `aws-sdk-s3` | Official AWS SDK for Rust; works with MinIO |
| **Auth** | `jsonwebtoken` + `openidconnect` | JWT validation, OAuth flows |
| **Secrets** | `vaultrs` | HashiCorp Vault client |
| **Search** | `tantivy` | Pure Rust full-text search engine |
| **Tracing** | `tracing` + `tracing-opentelemetry` | Structured logging + distributed traces |
| **Local DB** | `rusqlite` / `sqlx` (SQLite) | For client-side local cache |
| **Compression** | `lz4_flex` | Fast compression for JSON snapshots |

### Infrastructure

| Component | Choice | Rationale |
|---|---|---|
| **Cloud DB** | PostgreSQL 17+ / CockroachDB / YugabyteDB | Pg-compatible, horizontally scalable |
| **Object Store** | S3 / MinIO | Durable blob storage with lifecycle policies |
| **Secrets Vault** | HashiCorp Vault | Ephemeral DB credentials, auto-rotation |
| **Container** | Docker (distroless base) | Minimal attack surface |
| **Orchestration** | Kubernetes | HPA for stateless, StatefulSets for DBs |
| **CDN** | CloudFront / Cloudflare | Cached delivery of frequently accessed blobs |

---

## 4. Cloud Storage — PostgreSQL & Distributed SQL

### Why PostgreSQL (and When to Scale Beyond)

- **Start with PostgreSQL 17+**: mature, trusted, SCRAM-SHA-256 auth, excellent Rust support via `sqlx`.
- **Scale to CockroachDB/YugabyteDB** when single-node Postgres can't handle the write throughput or you need multi-region.

### Schema Design Principles

```sql
-- Metadata only in SQL — blobs go to object storage
CREATE TABLE documents (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id    UUID NOT NULL,
    owner_id        UUID NOT NULL,
    title           TEXT NOT NULL,
    content_hash    BYTEA NOT NULL,          -- SHA-256 of blob
    blob_ref        TEXT NOT NULL,            -- S3 key / object path
    crdt_version    BIGINT NOT NULL DEFAULT 0,
    size_bytes      BIGINT NOT NULL,
    mime_type       TEXT NOT NULL DEFAULT 'text/plain',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ,             -- soft delete
    
    -- Partition key for sharding
    CONSTRAINT fk_workspace FOREIGN KEY (workspace_id) REFERENCES workspaces(id)
);

-- Partition by workspace_id (range or hash) for horizontal scale
-- Index on (workspace_id, updated_at DESC) for fast recent-docs queries
CREATE INDEX idx_docs_workspace_updated ON documents (workspace_id, updated_at DESC);

-- Revision history (append-only, time-partitioned)
CREATE TABLE revisions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    document_id     UUID NOT NULL REFERENCES documents(id),
    version         BIGINT NOT NULL,
    delta_blob_ref  TEXT NOT NULL,            -- compressed CRDT delta in S3
    author_id       UUID NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (created_at);
```

### Connection Management

```rust
use sqlx::postgres::PgPoolOptions;

// Use connection pooling — never open raw connections
let pool = PgPoolOptions::new()
    .max_connections(50)           // tune per instance
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .idle_timeout(Duration::from_secs(600))
    .connect(&database_url)        // URL from Vault, not hardcoded!
    .await?;
```

---

## 5. Local Storage — SQLite / JSON with Auto-Eviction

### Architecture

```
┌─────────────────────────────────────────┐
│             LOCAL CACHE LAYER            │
│                                          │
│  ┌──────────────────────────────────┐    │
│  │  SQLite Database (structured)    │    │
│  │  - Document metadata index       │    │
│  │  - CRDT state snapshots          │    │
│  │  - Sync queue (pending uploads)  │    │
│  │  - Access timestamps (for LRU)   │    │
│  └──────────────────────────────────┘    │
│                                          │
│  ┌──────────────────────────────────┐    │
│  │  JSON + LZ4 Snapshots (blobs)   │    │
│  │  - Fast cold-start hydration     │    │
│  │  - Compressed document content   │    │
│  └──────────────────────────────────┘    │
│                                          │
│  ┌──────────────────────────────────┐    │
│  │  EVICTION MANAGER                │    │
│  │  - Monitors total size           │    │
│  │  - Threshold: 1 GB (configurable)│    │
│  │  - Policy: LRU + cloud-verified  │    │
│  │  - Keeps metadata, deletes blobs │    │
│  └──────────────────────────────────┘    │
└─────────────────────────────────────────┘
```

### Eviction Algorithm (Rust Pseudocode)

```rust
/// Called after every local write or on a periodic timer
async fn enforce_local_cache_limit(db: &SqlitePool, config: &CacheConfig) -> Result<()> {
    let total_size = query_total_local_size(db).await?;
    
    if total_size <= config.max_local_bytes {  // e.g., 1 GB
        return Ok(());
    }
    
    // Get candidates: synced items, oldest access first (LRU)
    let candidates = sqlx::query_as!(
        CacheEntry,
        r#"SELECT id, blob_path, size_bytes, cloud_version, cloud_checksum
           FROM local_cache
           WHERE is_synced = true
           ORDER BY last_accessed_at ASC
           LIMIT 100"#
    )
    .fetch_all(db)
    .await?;
    
    let mut freed: u64 = 0;
    let target_free = total_size - config.max_local_bytes;
    
    for item in candidates {
        // CRITICAL: verify cloud has durable copy before local deletion
        if verify_cloud_copy(&item).await? {
            // Delete local blob file, keep metadata row
            tokio::fs::remove_file(&item.blob_path).await?;
            sqlx::query!(
                "UPDATE local_cache SET blob_path = NULL, evicted = true WHERE id = ?",
                item.id
            )
            .execute(db)
            .await?;
            
            freed += item.size_bytes;
            if freed >= target_free { break; }
        }
    }
    
    Ok(())
}

/// When user opens an evicted document — fetch from cloud
async fn on_document_open(doc_id: Uuid, local_db: &SqlitePool) -> Result<Document> {
    let entry = get_local_entry(local_db, doc_id).await?;
    
    if entry.evicted {
        // Show cached preview (title + first lines) immediately
        // Fetch full content from cloud in background
        let blob = fetch_from_cloud(entry.cloud_ref).await?;
        restore_local_blob(local_db, doc_id, &blob).await?;
        Ok(Document::from_blob(entry.metadata, blob))
    } else {
        Ok(Document::from_local(entry))
    }
}
```

### Key Rules

| Rule | Detail |
|---|---|
| **Threshold** | Default 1 GB; configurable per device |
| **Policy** | LRU (least recently used) — evict oldest-accessed first |
| **Safety** | Never delete locally until cloud copy is verified (checksum + version match) |
| **Metadata preserved** | Always keep title, dates, tags locally — only evict blob content |
| **Re-fetch** | On-demand with progress indicator when user opens evicted doc |
| **Sync queue** | Pending uploads are NEVER evicted |

---

## 6. Real-Time Sync — WebSocket & Live Presence

### WebSocket Gateway (axum)

```rust
use axum::{
    extract::{ws::{WebSocket, Message, WebSocketUpgrade}, State},
    routing::get,
    Router,
};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use dashmap::DashMap;

#[derive(Clone)]
struct AppState {
    // Active connections per document
    doc_channels: Arc<DashMap<Uuid, broadcast::Sender<SyncMessage>>>,
    // Presence: who's editing what
    presence: Arc<DashMap<Uuid, Vec<PresenceInfo>>>,
    db_pool: sqlx::PgPool,
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl axum::response::IntoResponse {
    ws.max_message_size(2 * 1024 * 1024)  // 2 MB max from server
      .on_upgrade(move |socket| handle_ws(socket, state))
}

async fn handle_ws(mut socket: WebSocket, state: Arc<AppState>) {
    // 1. Authenticate (first message must be auth token)
    // 2. Subscribe to document channel
    // 3. Send current state + presence list
    // 4. Loop: receive edits → apply CRDT → broadcast deltas
    
    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                let edit: EditMessage = serde_json::from_str(&text).unwrap();
                
                // Apply CRDT merge
                let delta = apply_crdt_edit(&state, &edit).await;
                
                // Persist to WAL / change-log
                persist_change(&state.db_pool, &delta).await;
                
                // Broadcast to all connected clients on this document
                if let Some(tx) = state.doc_channels.get(&edit.doc_id) {
                    let _ = tx.send(SyncMessage::Delta(delta));
                }
            }
            Message::Ping(data) => {
                let _ = socket.send(Message::Pong(data)).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
    
    // Cleanup: remove from presence, unsubscribe
}

// Presence: "someone is typing..."
#[derive(Clone, serde::Serialize)]
struct PresenceInfo {
    user_id: Uuid,
    user_name: String,
    cursor_position: Option<usize>,
    is_typing: bool,
    last_active: chrono::DateTime<chrono::Utc>,
}
```

### Sync Protocol Design

| Feature | Implementation |
|---|---|
| **Delta sync** | Send only changed fields/ops, not full documents |
| **Sequence numbers** | Monotonic per-client; server merges in causal order |
| **Heartbeat** | Client sends ping every 30s; server drops after 90s silence |
| **Reconnect** | Client stores last sequence number; resumes from that point |
| **Compression** | LZ4 compress deltas > 1 KB before sending |
| **Typing indicator** | Debounced presence updates (every 500ms while typing) |
| **Cursor position** | Broadcast cursor location for collaborative editing |

---

## 7. CRDT — Conflict-Free Offline Merge

### Recommended: `yrs` (Yjs Rust Port)

```rust
use yrs::{Doc, Text, Transact, ReadTxn, updates::decoder::Decode, Update};

// Create a collaborative document
fn create_collaborative_doc() -> Doc {
    let doc = Doc::new();
    // Text CRDT for note content
    let text = doc.get_or_insert_text("content");
    
    // Apply local edit
    let mut txn = doc.transact_mut();
    text.insert(&mut txn, 0, "Hello from device A");
    drop(txn);
    
    doc
}

// Merge remote changes (from another device)
fn merge_remote_update(doc: &Doc, remote_update: &[u8]) -> Result<()> {
    let update = Update::decode_v1(remote_update)?;
    let mut txn = doc.transact_mut();
    txn.apply_update(update)?;
    Ok(())
}

// Extract delta to send to other devices
fn get_state_update(doc: &Doc) -> Vec<u8> {
    let txn = doc.transact();
    txn.encode_state_as_update_v1(&Default::default())
}
```

### Why CRDTs Over OT?

| Aspect | CRDT (Recommended) | OT |
|---|---|---|
| **Offline edits** | ✅ Automatic merge, no conflicts | ❌ Requires server coordination |
| **Server complexity** | Lower — merge is deterministic | Higher — transform functions |
| **Mobile-friendly** | ✅ Works fully offline | Needs constant connection |
| **Libraries** | `yrs`, `automerge` (mature) | Custom implementation needed |

---

## 8. Security — Eliminating Password Leak Risks

> ⚠️ **This is your primary security concern.** PostgreSQL and other databases ARE vulnerable to credential leaks. Here's how to make it nearly impossible.

### The Problem

Traditional DB access: `app → hardcoded password → database` — if the password leaks (via code repo, logs, config file, memory dump), attacker gets full DB access.

### The Solution: Zero Static Credentials

```
┌──────────┐     ┌──────────────┐     ┌──────────────┐
│  Client   │────→│  Rust Backend │────→│  Vault       │
│  (no DB   │     │  (no static  │     │  (ephemeral  │
│  creds)   │     │   password)  │     │   DB creds)  │
└──────────┘     └──────┬───────┘     └──────┬───────┘
                        │                     │
                        │  short-lived cred    │
                        │◄────────────────────┘
                        │
                        ▼
                 ┌──────────────┐
                 │  PostgreSQL  │
                 │  (SCRAM-SHA- │
                 │   256 auth)  │
                 └──────────────┘
```

### Implementation Layers

#### Layer 1: No Credentials in Client Apps — EVER

```rust
// ❌ NEVER DO THIS (client side)
let db_url = "postgres://user:password@host/db";

// ✅ CORRECT: Client authenticates to YOUR backend via JWT/OAuth
// Backend holds DB access; client never sees DB credentials
let api_token = authenticate_user(email, password).await?;
let notes = api_client.get_notes(&api_token).await?;
```

#### Layer 2: Ephemeral DB Credentials via Vault

```rust
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};

async fn get_db_credentials(vault: &VaultClient) -> Result<DatabaseCredentials> {
    // Vault generates a NEW username/password pair
    // that expires in 1 hour (configurable)
    let creds = vaultrs::database::roles::creds(
        vault,
        "database",          // mount path
        "app-readonly",      // role name — least privilege!
    ).await?;
    
    // creds.username = "v-app-readonly-abc123" (temporary)
    // creds.password = "random-generated-password" (temporary)
    // Auto-revoked after TTL expires
    
    Ok(DatabaseCredentials {
        username: creds.username,
        password: creds.password,
        ttl: creds.lease_duration,
    })
}
```

#### Layer 3: PostgreSQL Hardening

```sql
-- Use SCRAM-SHA-256 (NOT md5) — pg_hba.conf
hostssl all all 0.0.0.0/0 scram-sha-256

-- Set in postgresql.conf
password_encryption = 'scram-sha-256'
ssl = on
ssl_min_protocol_version = 'TLSv1.3'

-- Least-privilege roles
CREATE ROLE app_reader WITH LOGIN;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO app_reader;

CREATE ROLE app_writer WITH LOGIN;
GRANT SELECT, INSERT, UPDATE ON ALL TABLES IN SCHEMA public TO app_writer;
-- NO DELETE, NO DROP, NO ALTER for application roles
```

#### Layer 4: Encryption Everywhere

| What | How |
|---|---|
| **In transit** | TLS 1.3 mandatory (client↔backend, backend↔DB, internal RPC) |
| **At rest (DB)** | OS-level disk encryption + `pgcrypto` for sensitive columns |
| **At rest (blobs)** | S3 server-side encryption (SSE-S3 or SSE-KMS) |
| **Application-level** | Encrypt note content before storing (AES-256-GCM via `ring` crate) |
| **Local cache** | SQLCipher or application-level encryption for local SQLite |

#### Layer 5: Audit & Monitoring

```rust
// Log all access events (but NEVER log credentials)
tracing::info!(
    user_id = %user_id,
    action = "document_read",
    doc_id = %doc_id,
    // password = secret,  // ❌ NEVER
);

// Monitor for anomalies
// - Unusual access patterns (bulk downloads)
// - Failed auth attempts (rate limit + alert)
// - Credential rotation failures (alert immediately)
```

### Security Checklist

- [ ] No DB credentials in client code — ever
- [ ] Vault issues ephemeral credentials (TTL ≤ 1 hour)
- [ ] SCRAM-SHA-256 authentication on PostgreSQL
- [ ] TLS 1.3 on all connections
- [ ] Least-privilege DB roles (separate read/write)
- [ ] Application-level encryption for sensitive content
- [ ] SQLCipher for local SQLite (mobile)
- [ ] Audit logging (redacted, no PII/secrets)
- [ ] Automated credential rotation monitoring
- [ ] CI/CD secret scanning (prevent repo leaks)

---

## 9. Data Partitioning & Tiered Storage

### Storage Tiers

| Tier | Storage | Data | Latency | Cost |
|---|---|---|---|---|
| **Hot** | PostgreSQL + Redis cache | Active documents, metadata | <10ms | $$$ |
| **Warm** | S3 Standard | Recent blobs, attachments | <100ms | $$ |
| **Cold** | S3 Glacier / Deep Archive | Old revisions, archived docs | minutes | $ |

### Partitioning Strategy

```
50,000 TB ÷ across workspaces
  → Partition by workspace_id (consistent hash)
  → Sub-partition revisions by time (monthly)
  → Hot shards on NVMe SSDs
  → Automatic tier migration via lifecycle policies
```

### Object Storage Lifecycle

```json
{
  "Rules": [
    {
      "ID": "warm-to-cold",
      "Status": "Enabled",
      "Transitions": [
        { "Days": 90, "StorageClass": "STANDARD_IA" },
        { "Days": 365, "StorageClass": "GLACIER" },
        { "Days": 1825, "StorageClass": "DEEP_ARCHIVE" }
      ]
    }
  ]
}
```

---

## 10. Search & Indexing at Scale

### Recommended: Tantivy (Pure Rust) + Sharded

```rust
use tantivy::{schema::*, Index, IndexWriter};

fn build_search_index() -> Result<Index> {
    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("content", TEXT);
    schema_builder.add_u64_field("workspace_id", INDEXED);
    schema_builder.add_date_field("updated_at", INDEXED | STORED);
    
    let schema = schema_builder.build();
    let index = Index::create_in_dir("/data/search-index", schema)?;
    Ok(index)
}
```

For trillions of documents, shard search indexes by workspace/user group and run distributed Tantivy or use Meilisearch/OpenSearch.

---

## 11. Mobile Readiness & Future-Proofing

### Cross-Platform Core Library

```
┌─────────────────────────────────┐
│     Shared Rust Core Library     │
│  - CRDT engine (yrs/automerge)  │
│  - Sync protocol client         │
│  - Local cache manager          │
│  - Encryption layer             │
│  - API client                   │
└──────────┬──────────────────────┘
           │
    ┌──────┼──────────┐
    ▼      ▼          ▼
 UniFFI   WASM      C-ABI
 (Swift/  (Web)     (any
  Kotlin)            native)
```

### UniFFI for Mobile Bindings

```rust
// Define interface once in Rust, generate Swift + Kotlin bindings
#[uniffi::export]
fn sync_document(doc_id: String, server_url: String) -> Result<SyncResult, SyncError> {
    // Shared Rust logic for all platforms
    let result = runtime.block_on(async {
        let client = create_ws_client(&server_url).await?;
        client.sync(&doc_id).await
    })?;
    Ok(result)
}
```

---

## 12. Horizontal Scaling & Deployment

### Docker Multi-Stage Build

```dockerfile
# Build stage
FROM rust:1.85 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage (minimal)
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/backend /
EXPOSE 3000
CMD ["/backend"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-backend
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: backend
        image: your-registry/rust-backend:latest
        ports:
        - containerPort: 3000
        resources:
          requests: { memory: "256Mi", cpu: "500m" }
          limits:   { memory: "1Gi",   cpu: "2000m" }
        env:
        - name: VAULT_ADDR
          valueFrom:
            secretKeyRef:
              name: vault-config
              key: addr
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: rust-backend-hpa
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: rust-backend
  minReplicas: 3
  maxReplicas: 100
  metrics:
  - type: Resource
    resource:
      name: cpu
      target: { type: Utilization, averageUtilization: 70 }
```

---

## 13. Observability & Reliability

| Tool | Purpose |
|---|---|
| **OpenTelemetry** | Distributed tracing (request flow across services) |
| **Prometheus** | Metrics (latency percentiles, connection counts, cache hit rates) |
| **Grafana** | Dashboards and alerting |
| **Loki** | Centralized log aggregation |

### Key SLOs

| Metric | Target |
|---|---|
| Read latency (p95) | < 100ms |
| Write latency (p95) | < 200ms |
| WebSocket message delivery | < 50ms |
| Availability | 99.95% |
| Sync lag (cloud ↔ local) | < 5 seconds |

---

## 14. Project Structure & Code Examples

```
rust-backend/
├── Cargo.toml
├── crates/
│   ├── gateway/           # axum HTTP/WS server
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── routes/
│   │   │   ├── ws/        # WebSocket handlers
│   │   │   ├── middleware/ # auth, rate-limit, tracing
│   │   │   └── error.rs
│   │   └── Cargo.toml
│   ├── sync-engine/       # CRDT merge, delta generation
│   │   ├── src/
│   │   │   ├── crdt.rs
│   │   │   ├── presence.rs
│   │   │   └── protocol.rs
│   │   └── Cargo.toml
│   ├── storage/           # DB access, S3 client
│   │   ├── src/
│   │   │   ├── postgres.rs
│   │   │   ├── s3.rs
│   │   │   └── migrations/
│   │   └── Cargo.toml
│   ├── local-cache/       # SQLite, eviction, offline
│   │   ├── src/
│   │   │   ├── sqlite.rs
│   │   │   ├── eviction.rs
│   │   │   └── sync_queue.rs
│   │   └── Cargo.toml
│   ├── auth/              # JWT, OAuth, Vault integration
│   │   └── ...
│   └── shared/            # Common types, error types
│       └── ...
├── mobile-core/           # Shared library for future mobile
│   ├── src/lib.rs
│   └── uniffi.toml
└── deploy/
    ├── Dockerfile
    ├── k8s/
    └── vault-config/
```

---

## 15. Development Phases & Roadmap

### Phase 1: Foundation (Weeks 1–4)
- [ ] Set up Cargo workspace with crate structure
- [ ] Implement axum gateway with health check routes
- [ ] PostgreSQL connection via sqlx + Vault ephemeral creds
- [ ] Basic REST API: CRUD for documents (metadata in Pg, blobs in S3)
- [ ] Authentication (JWT middleware)

### Phase 2: Real-Time (Weeks 5–8)
- [ ] WebSocket endpoint with connection management
- [ ] Integrate `yrs` CRDT for document editing
- [ ] Delta sync protocol (sequence numbers, reconnect)
- [ ] Presence system ("user X is typing...")
- [ ] Broadcast engine (tokio broadcast channels + NATS for multi-instance)

### Phase 3: Local-First (Weeks 9–12)
- [ ] SQLite local cache with `rusqlite`
- [ ] JSON+LZ4 snapshot for fast cold start
- [ ] Bi-directional sync (local ↔ cloud)
- [ ] Auto-eviction manager (1 GB threshold, LRU, cloud-verified)
- [ ] Offline queue (pending changes stored locally, replayed on reconnect)

### Phase 4: Scale & Harden (Weeks 13–16)
- [ ] Search indexing (Tantivy or Meilisearch)
- [ ] Horizontal scaling (k8s HPA, connection-aware load balancing)
- [ ] Tiered storage lifecycle (hot → warm → cold)
- [ ] Security audit (SCRAM-SHA-256, TLS 1.3 everywhere, secrets rotation)
- [ ] Observability (OpenTelemetry, Prometheus, Grafana dashboards)

### Phase 5: Mobile Preparation (Weeks 17–20)
- [ ] Extract shared core library (`mobile-core` crate)
- [ ] UniFFI bindings for Swift/Kotlin
- [ ] WASM compilation for web client
- [ ] Integration testing across platforms
- [ ] Performance benchmarking and optimization

---

## Summary — Key Decisions at a Glance

| Decision | Choice | Why |
|---|---|---|
| **Language** | Pure Rust | Performance, safety, cross-platform |
| **Web Framework** | axum 0.8.x | Tokio-native, Tower middleware, mature |
| **Cloud DB** | PostgreSQL → CockroachDB | Start simple, scale horizontally |
| **Local DB** | SQLite + JSON/LZ4 | Fast, embedded, universal |
| **Sync** | WebSocket + CRDT (yrs) | Real-time, offline-merge, mobile-ready |
| **Secrets** | HashiCorp Vault (ephemeral) | No static passwords, auto-rotation |
| **Auth** | JWT + OAuth 2.0 | Standard, mobile-compatible |
| **Blobs** | S3-compatible object storage | Durable, lifecycle policies, cheap at scale |
| **Search** | Tantivy (Rust) / Meilisearch | Full-text, fast, scalable |
| **Eviction** | LRU @ 1 GB, cloud-verified | Device stays lean, data is safe |

---

> **Start small, scale incrementally.** Begin with single-instance PostgreSQL + S3 + axum + SQLite local cache + yrs CRDT sync. Add distributed SQL, sharding, and multi-region only when your data and traffic demand it.

---

*Generated: 2026-02-27 | Verified against: Axum 0.8.x, Tokio 1.42+, sqlx 0.8+, yrs/Yjs, Automerge 3.x, PostgreSQL 17, HashiCorp Vault*
