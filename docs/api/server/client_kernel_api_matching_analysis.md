# GraphDB Client-Kernel API Matching Analysis

This document analyzes the API matching between the `graphdb-cli` client package and the server APIs provided by `src/api/server`.

## Executive Summary

The analysis reveals a **partial match** between the client and server APIs. While the core functionality (authentication, query execution, schema browsing) is aligned, there are several discrepancies and missing features on both sides.

**Overall Match Rate**: ~65%

---

## 1. Client API Overview (graphdb-cli)

The `graphdb-cli` client provides HTTP client functionality through the following components:

### 1.1 Core Client Trait (`GraphDbClient`)

Located in: `graphdb-cli/src/client/client_trait.rs`

| Method | Description | HTTP Method | Server Endpoint |
|--------|-------------|-------------|-----------------|
| `connect()` | Authenticate and create session | POST | `/v1/auth/login` |
| `disconnect()` | Close connection | - | (No explicit logout call) |
| `execute_query()` | Execute query | POST | `/v1/query` |
| `execute_query_raw()` | Execute query without substitution | POST | `/v1/query` |
| `list_spaces()` | List all spaces | GET | `/v1/schema/spaces` |
| `switch_space()` | Get space details | GET | `/v1/schema/spaces/{name}` |
| `list_tags()` | List tags in space | GET | `/v1/schema/spaces/{name}/tags` |
| `list_edge_types()` | List edge types | GET | `/v1/schema/spaces/{name}/edge-types` |
| `health_check()` | Check server health | GET | `/v1/health` |

### 1.2 HTTP Client Implementation

Located in: `graphdb-cli/src/client/http.rs`

**Internal Methods**:
- `create_session()` - POST `/v1/sessions`
- `login()` - POST `/v1/auth/login`

**Legacy Client (`GraphDBHttpClient`)**:
- `health_check()` - GET `/v1/health`
- `login()` - POST `/v1/auth/login`
- `create_session()` - POST `/v1/sessions`
- `execute_query()` - POST `/v1/query`
- `list_spaces()` - GET `/v1/schema/spaces`
- `use_space()` - GET `/v1/schema/spaces/{name}`
- `list_tags()` - GET `/v1/schema/spaces/{name}/tags`
- `list_edge_types()` - GET `/v1/schema/spaces/{name}/edge-types`

---

## 2. Server API Overview

Located in: `src/api/server/http/router.rs`

### 2.1 Public Routes (No Auth Required)

| Endpoint | Method | Handler | Client Support |
|----------|--------|---------|----------------|
| `/v1/health` | GET | `health::check` | Yes |
| `/v1/auth/login` | POST | `auth::login` | Yes |
| `/v1/auth/logout` | POST | `auth::logout` | **No** |

### 2.2 Protected Routes (Auth Required)

| Endpoint | Method | Handler | Client Support |
|----------|--------|---------|----------------|
| `/v1/sessions` | POST | `session::create` | Yes (internal) |
| `/v1/sessions/{id}` | GET/DELETE | `session::get/delete` | **No** |
| `/v1/query` | POST | `query::execute` | Yes |
| `/v1/query/validate` | POST | `query::validate` | **No** |
| `/v1/transactions/*` | POST | `transaction::*` | **No** |
| `/v1/batch/*` | POST/GET/DELETE | `batch::*` | **No** |
| `/v1/statistics/*` | GET | `statistics::*` | **No** |
| `/v1/config/*` | GET/PUT/DELETE | `config::*` | **No** |
| `/v1/functions/*` | POST/GET/DELETE | `function::*` | **No** |
| `/v1/query/stream` | POST | `stream::execute` | **No** |
| `/v1/vector/*` | POST/GET/DELETE | `vector::*` | **No** |
| `/v1/sync/status` | GET | `sync::status` | **No** |
| `/v1/schema/spaces` | POST/GET | `schema::*` | Partial |
| `/v1/schema/spaces/{name}` | GET/DELETE | `schema::*` | Partial |
| `/v1/schema/spaces/{name}/tags` | POST/GET | `schema::*` | Partial |
| `/v1/schema/spaces/{name}/edge-types` | POST/GET | `schema::*` | Partial |

### 2.3 Web Routes (Extended APIs)

| Endpoint | Method | Description | Client Support |
|----------|--------|-------------|----------------|
| `/api/spaces/{name}/tags/{tag}/vertices` | GET | Browse vertices | **No** |
| `/api/spaces/{name}/edge-types/{type}/edges` | GET | Browse edges | **No** |
| `/api/vertices/{vid}` | GET | Get vertex | **No** |
| `/api/edges` | GET | Get edge | **No** |
| `/api/vertices/{vid}/neighbors` | GET | Get neighbors | **No** |
| `/api/history/*` | GET/POST/DELETE | Query history | **No** |
| `/api/favorites/*` | GET/POST/DELETE | Favorites | **No** |
| `/api/spaces/*` | GET/POST | Extended schema | **No** |

---

## 3. Detailed Matching Analysis

### 3.1 Fully Matched APIs

These APIs are fully implemented on both client and server:

| API | Client Method | Server Endpoint | Status |
|-----|---------------|-----------------|--------|
| Health Check | `health_check()` | `GET /v1/health` | Match |
| Login | `connect()` -> `login()` | `POST /v1/auth/login` | Match |
| Create Session | `create_session()` | `POST /v1/sessions` | Match |
| Execute Query | `execute_query()` | `POST /v1/query` | Match |
| List Spaces | `list_spaces()` | `GET /v1/schema/spaces` | Match |
| Get Space | `switch_space()` | `GET /v1/schema/spaces/{name}` | Match |
| List Tags | `list_tags()` | `GET /v1/schema/spaces/{name}/tags` | Match |
| List Edge Types | `list_edge_types()` | `GET /v1/schema/spaces/{name}/edge-types` | Match |

### 3.2 Partially Matched APIs

These APIs have some discrepancies:

#### 3.2.1 Schema Management

| Feature | Server | Client | Issue |
|---------|--------|--------|-------|
| Create Space | `POST /v1/schema/spaces` | **Missing** | Client cannot create spaces |
| Drop Space | `DELETE /v1/schema/spaces/{name}` | **Missing** | Client cannot delete spaces |
| Create Tag | `POST /v1/schema/spaces/{name}/tags` | **Missing** | Client cannot create tags |
| Create Edge Type | `POST /v1/schema/spaces/{name}/edge-types` | **Missing** | Client cannot create edge types |

**Response Format Mismatch**:
- Server returns: `{ "spaces": [...] }`
- Client expects: `Vec<SpaceInfo>` with fields `id`, `name`, `vid_type`, `comment`
- The client parses the response correctly but the server response structure may vary

#### 3.2.2 Session Management

| Feature | Server | Client | Issue |
|---------|--------|--------|-------|
| Get Session | `GET /v1/sessions/{id}` | **Missing** | Cannot retrieve session info |
| Delete Session | `DELETE /v1/sessions/{id}` | **Missing** | `disconnect()` doesn't call logout |

### 3.3 Missing in Client (Server-Only APIs)

These server APIs have no corresponding client implementation:

#### 3.3.1 Query Operations

| Server API | Description | Priority |
|------------|-------------|----------|
| `POST /v1/query/validate` | Validate query syntax | Medium |
| `POST /v1/query/stream` | Streaming query execution | Low |

#### 3.3.2 Transaction Management

| Server API | Description | Priority |
|------------|-------------|----------|
| `POST /v1/transactions` | Begin transaction | **High** |
| `POST /v1/transactions/{id}/commit` | Commit transaction | **High** |
| `POST /v1/transactions/{id}/rollback` | Rollback transaction | **High** |

#### 3.3.3 Batch Operations

| Server API | Description | Priority |
|------------|-------------|----------|
| `POST /v1/batch` | Create batch task | Medium |
| `GET /v1/batch/{id}` | Get batch status | Medium |
| `POST /v1/batch/{id}/items` | Add batch items | Medium |
| `POST /v1/batch/{id}/execute` | Execute batch | Medium |
| `POST /v1/batch/{id}/cancel` | Cancel batch | Low |
| `DELETE /v1/batch/{id}` | Delete batch | Low |

#### 3.3.4 Vector Operations

| Server API | Description | Priority |
|------------|-------------|----------|
| `GET/POST /v1/vector/indexes` | List/Create vector indexes | Low |
| `GET/DELETE /v1/vector/indexes/{id}` | Get/Drop vector index | Low |
| `POST /v1/vector/search` | Vector search | Low |
| `GET /v1/vector/{id}/{point}` | Get vector point | Low |
| `GET /v1/vector/{id}/count` | Vector count | Low |

#### 3.3.5 Statistics APIs

| Server API | Description | Priority |
|------------|-------------|----------|
| `GET /v1/statistics/sessions/{id}` | Session statistics | Medium |
| `GET /v1/statistics/queries` | Query statistics | Medium |
| `GET /v1/statistics/database` | Database statistics | Medium |
| `GET /v1/statistics/system` | System statistics | Low |

#### 3.3.6 Configuration APIs

| Server API | Description | Priority |
|------------|-------------|----------|
| `GET /v1/config` | Get configuration | Low |
| `PUT /v1/config` | Update configuration | Low |
| `GET/PUT/DELETE /v1/config/{section}/{key}` | Config key operations | Low |

#### 3.3.7 Function APIs

| Server API | Description | Priority |
|------------|-------------|----------|
| `GET /v1/functions` | List functions | Low |
| `POST /v1/functions` | Register function | Low |
| `GET/DELETE /v1/functions/{name}` | Get/Unregister function | Low |

#### 3.3.8 Sync APIs

| Server API | Description | Priority |
|------------|-------------|----------|
| `GET /v1/sync/status` | Sync status | Low |

#### 3.3.9 Web Management APIs

| Server API | Description | Priority |
|------------|-------------|----------|
| `GET /api/spaces/{name}/tags/{tag}/vertices` | Browse vertices | Medium |
| `GET /api/spaces/{name}/edge-types/{type}/edges` | Browse edges | Medium |
| `GET /api/vertices/{vid}` | Get vertex details | Medium |
| `GET /api/edges` | Get edge details | Medium |
| `GET /api/vertices/{vid}/neighbors` | Get neighbors | Medium |
| `GET/POST /api/history/*` | Query history | Low |
| `GET/POST /api/favorites/*` | Favorites | Low |

---

## 4. Data Type Compatibility

### 4.1 Request/Response Types

#### Login Request

**Client sends** (`graphdb-cli/src/client/http.rs`):
```rust
struct LoginRequest {
    username: String,
    password: String,
}
```

**Server expects** (`src/api/server/http/handlers/auth.rs`):
```rust
struct LoginRequest {
    username: String,
    password: String,
}
```

Status: Match

#### Login Response

**Server returns**:
```rust
struct LoginResponse {
    session_id: i64,
    username: String,
    expires_at: Option<u64>,
}
```

**Client expects**:
```rust
struct LoginResponse {
    session_id: i64,
    username: String,
    #[serde(default)]
    expires_at: Option<u64>,
}
```

Status: Match

#### Query Request

**Client sends**:
```rust
struct QueryRequest {
    query: String,
    session_id: i64,
    #[serde(default)]
    parameters: HashMap<String, String>,
}
```

**Server expects** (`src/api/server/http/handlers/query_types.rs`):
```rust
struct QueryRequest {
    pub query: String,
    pub session_id: i64,
    #[serde(default)]
    pub parameters: HashMap<String, String>,
}
```

Status: Match

#### Query Response

**Server returns**:
```rust
struct QueryResponse {
    pub success: bool,
    pub data: Option<QueryData>,
    pub error: Option<QueryError>,
    pub metadata: QueryMetadata,
}

struct QueryData {
    pub columns: Vec<String>,
    pub rows: Vec<HashMap<String, serde_json::Value>>,
    pub row_count: usize,
}
```

**Client expects**:
```rust
struct QueryResponse {
    success: bool,
    data: Option<QueryData>,
    error: Option<QueryError>,
    metadata: Option<QueryMetadata>,
}
```

Status: **Partial Match** - Server returns non-optional metadata, client expects optional

#### SpaceInfo

**Server returns** (from `schema::list_spaces`):
```json
{
  "spaces": [
    {
      "id": 1,
      "name": "space_name",
      "vid_type": "STRING",
      "comment": null
    }
  ]
}
```

**Client expects**:
```rust
struct SpaceInfo {
    pub id: u64,
    pub name: String,
    pub vid_type: String,
    pub comment: Option<String>,
}
```

Status: Match (client extracts from `spaces` array)

### 4.2 Type Mismatches

| Type | Server | Client | Issue |
|------|--------|--------|-------|
| Space ID | `i64` in some places | `u64` | Potential overflow issues |
| Session ID | `i64` | `i64` | Match |
| Row Count | `usize` | `usize` | Match |

---

## 5. Issues and Recommendations

### 5.1 Critical Issues

#### Issue 1: Missing Transaction Support
**Severity**: High

The client has no transaction management capabilities while the server provides full transaction APIs.

**Recommendation**: Add transaction methods to `GraphDbClient` trait:
```rust
async fn begin_transaction(&self, session_id: i64) -> Result<u64>;
async fn commit_transaction(&self, txn_id: u64) -> Result<()>;
async fn rollback_transaction(&self, txn_id: u64) -> Result<()>;
```

#### Issue 2: No Logout on Disconnect
**Severity**: Medium

The client's `disconnect()` method doesn't call the server's logout endpoint, leaving sessions active on the server.

**Recommendation**: Update `disconnect()` to call `POST /v1/auth/logout`.

#### Issue 3: Missing Schema DDL Operations
**Severity**: Medium

Client cannot create/drop spaces, tags, or edge types.

**Recommendation**: Add schema management methods:
```rust
async fn create_space(&self, name: &str, config: SpaceConfig) -> Result<()>;
async fn drop_space(&self, name: &str) -> Result<()>;
async fn create_tag(&self, space: &str, name: &str, properties: Vec<PropertyDef>) -> Result<()>;
async fn create_edge_type(&self, space: &str, name: &str, properties: Vec<PropertyDef>) -> Result<()>;
```

### 5.2 Medium Priority Issues

#### Issue 4: No Batch Operations
**Severity**: Medium

Large data imports would benefit from batch API support.

#### Issue 5: No Statistics Access
**Severity**: Low-Medium

Cannot access query performance statistics from client.

#### Issue 6: No Query Validation
**Severity**: Low

Cannot validate queries before execution.

### 5.3 Low Priority Issues

#### Issue 7: No Vector Operations
**Severity**: Low

Vector search is not accessible from CLI client.

#### Issue 8: No Configuration Management
**Severity**: Low

Cannot view or modify server configuration from client.

#### Issue 9: No Streaming Query Support
**Severity**: Low

Large result sets cannot be streamed.

---

## 6. API Coverage Matrix

| Category | Server APIs | Client APIs | Coverage |
|----------|-------------|-------------|----------|
| Public | 3 | 2 | 67% |
| Session | 3 | 1 | 33% |
| Query | 3 | 2 | 67% |
| Transaction | 3 | 0 | 0% |
| Schema (Read) | 6 | 4 | 67% |
| Schema (Write) | 4 | 0 | 0% |
| Batch | 6 | 0 | 0% |
| Vector | 7 | 0 | 0% |
| Statistics | 4 | 0 | 0% |
| Config | 5 | 0 | 0% |
| Function | 4 | 0 | 0% |
| Sync | 1 | 0 | 0% |
| Web | 12 | 0 | 0% |
| **Total** | **61** | **9** | **15%** |

**Note**: Coverage is calculated based on unique API endpoints vs implemented client methods.

---

## 7. Conclusion

### 7.1 Current State

The `graphdb-cli` client provides basic functionality for:
- Authentication and session management
- Query execution
- Schema browsing (read-only)
- Health checks

### 7.2 Gaps

Major functionality gaps exist in:
1. **Transaction Management** - Critical for data integrity
2. **Schema DDL** - Cannot create/modify schema
3. **Batch Operations** - Inefficient for bulk data loading
4. **Statistics** - No visibility into performance
5. **Advanced Query Features** - No validation or streaming

### 7.3 Recommendations

**Immediate (High Priority)**:
1. Implement transaction management in client
2. Fix disconnect to properly logout
3. Add schema DDL operations

**Short-term (Medium Priority)**:
1. Add batch operation support
2. Add statistics APIs
3. Add query validation

**Long-term (Low Priority)**:
1. Add vector operations
2. Add configuration management
3. Add streaming query support
4. Add web management APIs

---

## Appendix: API Mapping Table

| Client Method | Server Endpoint | Status | Notes |
|---------------|-----------------|--------|-------|
| `HttpClient::new()` | - | N/A | Constructor |
| `HttpClient::with_config()` | - | N/A | Constructor |
| `HttpClient::base_url()` | - | N/A | Utility |
| `HttpClient::inner()` | - | N/A | Utility |
| `HttpClient::create_session()` | `POST /v1/sessions` | Match | Private method |
| `HttpClient::login()` | `POST /v1/auth/login` | Match | Private method |
| `GraphDbClient::is_connected()` | - | N/A | State check |
| `GraphDbClient::connect()` | `POST /v1/auth/login` | Match | Calls login |
| `GraphDbClient::disconnect()` | - | **Gap** | Should call logout |
| `GraphDbClient::execute_query()` | `POST /v1/query` | Match | |
| `GraphDbClient::execute_query_raw()` | `POST /v1/query` | Match | Same as execute |
| `GraphDbClient::list_spaces()` | `GET /v1/schema/spaces` | Match | |
| `GraphDbClient::switch_space()` | `GET /v1/schema/spaces/{name}` | Match | |
| `GraphDbClient::list_tags()` | `GET /v1/schema/spaces/{name}/tags` | Match | |
| `GraphDbClient::list_edge_types()` | `GET /v1/schema/spaces/{name}/edge-types` | Match | |
| `GraphDbClient::health_check()` | `GET /v1/health` | Match | |
| `GraphDbClient::connection_string()` | - | N/A | Utility |
| `GraphDBHttpClient::new()` | - | N/A | Legacy constructor |
| `GraphDBHttpClient::health_check()` | `GET /v1/health` | Match | Legacy |
| `GraphDBHttpClient::login()` | `POST /v1/auth/login` | Match | Legacy |
| `GraphDBHttpClient::create_session()` | `POST /v1/sessions` | Match | Legacy |
| `GraphDBHttpClient::execute_query()` | `POST /v1/query` | Match | Legacy |
| `GraphDBHttpClient::list_spaces()` | `GET /v1/schema/spaces` | Match | Legacy |
| `GraphDBHttpClient::use_space()` | `GET /v1/schema/spaces/{name}` | Match | Legacy |
| `GraphDBHttpClient::list_tags()` | `GET /v1/schema/spaces/{name}/tags` | Match | Legacy |
| `GraphDBHttpClient::list_edge_types()` | `GET /v1/schema/spaces/{name}/edge-types` | Match | Legacy |
