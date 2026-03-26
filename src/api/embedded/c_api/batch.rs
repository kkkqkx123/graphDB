//! C API Batch Operation Module
//!
//! Provides batch operation functions, supporting batch insert, batch update, and batch delete

use crate::api::embedded::c_api::error::{graphdb_error_code_t, set_last_error_message};
use crate::api::embedded::c_api::session::GraphDbSessionHandle;
use crate::api::embedded::c_api::types::graphdb_value_type_t;
use crate::api::embedded::c_api::types::{graphdb_batch_t, graphdb_session_t, graphdb_value_t};
use crate::core::{Edge, Value, Vertex};
use crate::storage::StorageClient;
use std::collections::HashMap;
use std::ffi::{c_char, c_int, CStr, CString};

// Batch action item type
enum BatchItem {
    Vertex(Vertex),
    Edge(Edge),
}

/// Internal Structure of Batch Operation Handles
///
/// Note: This structure holds the session pointer, but does not own the session.
/// The caller must ensure that the session is not closed until the batch operation handle is released.
pub struct GraphDbBatchHandle {
    /// Associated session pointer (used to verify session validity)
    session_ptr: *mut GraphDbSessionHandle,
    /// Batch size
    batch_size: usize,
    /// Buffer
    buffer: Vec<BatchItem>,
    /// Number of inserted vertices
    vertices_inserted: usize,
    /// Number of inserted edges
    edges_inserted: usize,
    /// Error messages
    errors: Vec<String>,
    /// Final error
    pub(crate) last_error: Option<CString>,
}

impl GraphDbBatchHandle {
    /// Check if the session is still active
    fn is_session_valid(&self) -> bool {
        !self.session_ptr.is_null()
    }

    /// Get session reference (if valid)
    fn get_session(&self) -> Option<&GraphDbSessionHandle> {
        if self.is_session_valid() {
            Some(unsafe { &*self.session_ptr })
        } else {
            None
        }
    }

    /// Flush vertex buffer
    fn flush_vertices(&mut self) -> Result<(), String> {
        // Separate vertices and edges
        let mut vertices = Vec::new();
        let mut remaining = Vec::new();

        for item in self.buffer.drain(..) {
            match item {
                BatchItem::Vertex(v) => vertices.push(v),
                _ => remaining.push(item),
            }
        }

        // Put the edge back into the buffer
        self.buffer.extend(remaining);

        if vertices.is_empty() {
            return Ok(());
        }

        let vertex_count = vertices.len();

        // Avoid borrowing conflicts by placing session-related operations in separate scopes
        let result = {
            let session = self
                .get_session()
                .ok_or_else(|| "Session invalid or closed".to_string())?;

            let space_name = session
                .inner
                .space_name()
                .ok_or_else(|| "No graph space selected".to_string())?;

            // Calling the storage layer's batch insertion interface
            let mut storage = session.inner.storage();
            storage.batch_insert_vertices(space_name, vertices)
        };

        match result {
            Ok(_) => {
                self.vertices_inserted += vertex_count;
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Batch insert vertices failed: {}", e);
                self.errors.push(err_msg.clone());
                Err(err_msg)
            }
        }
    }

    /// Flush edge buffer
    fn flush_edges(&mut self) -> Result<(), String> {
        // Separate edges and vertices
        let mut edges = Vec::new();
        let mut remaining = Vec::new();

        for item in self.buffer.drain(..) {
            match item {
                BatchItem::Edge(e) => edges.push(e),
                _ => remaining.push(item),
            }
        }

        // Put vertices back into the buffer
        self.buffer.extend(remaining);

        if edges.is_empty() {
            return Ok(());
        }

        let edge_count = edges.len();

        // Avoid borrowing conflicts by placing session-related operations in separate scopes
        let result = {
            let session = self
                .get_session()
                .ok_or_else(|| "Session invalid or closed".to_string())?;

            let space_name = session
                .inner
                .space_name()
                .ok_or_else(|| "No graph space selected".to_string())?;

            // Calling the storage layer's batch insertion interface
            let mut storage = session.inner.storage();
            storage.batch_insert_edges(space_name, edges)
        };

        match result {
            Ok(_) => {
                self.edges_inserted += edge_count;
                Ok(())
            }
            Err(e) => {
                let err_msg = format!("Batch insert edges failed: {}", e);
                self.errors.push(err_msg.clone());
                Err(err_msg)
            }
        }
    }

    /// Performs a batch insert, flushing all buffered data
    fn execute(&mut self) -> Result<(), String> {
        // Flush vertices first.
        self.flush_vertices()?;

        // Flush edges
        self.flush_edges()?;

        Ok(())
    }
}

/// Create a batch inserter
///
/// # Parameters
/// - `session`: session handle
/// - `batch_size`: batch size
/// - `batch`: output parameter, batch operation handle
///
/// # Returns
/// Success: GRAPHDB_OK
/// Failure: Error code
///
/// # Safety
/// - `session` must be a valid session handle created by `graphdb_session_create`
/// - `batch_size` must be a positive integer (if <= 0, defaults to 100)
/// - `batch` must be a valid pointer to store the batch handle
/// - The created batch handle holds a session pointer but does not own the session
/// - The caller must ensure the session is not closed before the batch handle is freed
/// - The caller is responsible for freeing the batch handle using `graphdb_batch_free` when done
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_inserter_create(
    session: *mut graphdb_session_t,
    batch_size: c_int,
    batch: *mut *mut graphdb_batch_t,
) -> c_int {
    if session.is_null() || batch.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let size = if batch_size <= 0 {
        100
    } else {
        batch_size as usize
    };

    unsafe {
        let batch_handle = Box::new(GraphDbBatchHandle {
            session_ptr: session as *mut GraphDbSessionHandle,
            batch_size: size,
            buffer: Vec::with_capacity(size),
            vertices_inserted: 0,
            edges_inserted: 0,
            errors: Vec::new(),
            last_error: None,
        });
        *batch = Box::into_raw(batch_handle) as *mut graphdb_batch_t;
        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// Adding Vertices
///
/// # Parameters
/// - `batch`: A handle for batch operations
/// - `vid`: vertex ID
/// - `tag_name`: tag name (UTF-8 encoding)
/// - `properties`: An array of properties.
/// - `prop_count`: The number of properties
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - The `batch` must be a valid batch operation handle created using the `graphdb_batch_inserter_create` function.
/// - `tag_name` must be a valid pointer to a UTF-8 string ending in null
/// - If `properties` is not `null`, it must point to at least `prop_count` valid `graphdb_value_t` elements.
/// - The caller must ensure that the associated session is still valid when calling this function.
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_add_vertex(
    batch: *mut graphdb_batch_t,
    vid: i64,
    tag_name: *const c_char,
    properties: *const graphdb_value_t,
    prop_count: usize,
) -> c_int {
    if batch.is_null() || tag_name.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let tag_str = unsafe {
        match CStr::from_ptr(tag_name).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    let mut props = HashMap::new();

    if !properties.is_null() && prop_count > 0 {
        for i in 0..prop_count {
            unsafe {
                let prop = &*properties.add(i);
                let prop_name = format!("prop_{}", i);
                let value = convert_c_value_to_rust(prop);
                props.insert(prop_name, value);
            }
        }
    }

    unsafe {
        let handle = &mut *(batch as *mut GraphDbBatchHandle);

        // Check the validity of the session.
        if !handle.is_session_valid() {
            return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
        }

        let mut vertex = Vertex::with_vid(Value::Int(vid));
        let tag = crate::core::vertex_edge_path::Tag::new(tag_str.to_string(), props);
        vertex.add_tag(tag);

        handle.buffer.push(BatchItem::Vertex(vertex));

        // If the batch size is reached, the content will be automatically flushed.
        if handle.buffer.len() >= handle.batch_size {
            if let Err(e) = handle.flush_vertices() {
                let error_msg = e.to_string();
                set_last_error_message(error_msg.clone());
                handle.last_error = Some(CString::new(error_msg).unwrap_or_default());
                return graphdb_error_code_t::GRAPHDB_ERROR as c_int;
            }
        }

        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// Add edges
///
/// # Parameters
/// - `batch`: Batch operation handle
/// - `src_vid`: ID of the source vertex
/// - `dst_vid`: ID of the target vertex
/// - `edge_type`: The name of the edge type (encoded in UTF-8)
/// - `rank`: Ranking
/// - `properties`: Array of properties
/// - `prop_count`: Number of properties
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
/// - The `edge_type` must be a valid pointer to a UTF-8 string that ends with `null`.
/// - If `properties` is not null, it must point to at least `prop_count` valid `graphdb_value_t` elements
/// - Caller must ensure the associated session is still valid when calling this function
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_add_edge(
    batch: *mut graphdb_batch_t,
    src_vid: i64,
    dst_vid: i64,
    edge_type: *const c_char,
    rank: i64,
    properties: *const graphdb_value_t,
    prop_count: usize,
) -> c_int {
    if batch.is_null() || edge_type.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    let edge_type_str = unsafe {
        match CStr::from_ptr(edge_type).to_str() {
            Ok(s) => s,
            Err(_) => return graphdb_error_code_t::GRAPHDB_MISUSE as c_int,
        }
    };

    let mut props = HashMap::new();

    if !properties.is_null() && prop_count > 0 {
        for i in 0..prop_count {
            unsafe {
                let prop = &*properties.add(i);
                let prop_name = format!("prop_{}", i);
                let value = convert_c_value_to_rust(prop);
                props.insert(prop_name, value);
            }
        }
    }

    unsafe {
        let handle = &mut *(batch as *mut GraphDbBatchHandle);

        // Check the validity of the session.
        if !handle.is_session_valid() {
            return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
        }

        let edge = Edge::new(
            Value::Int(src_vid),
            Value::Int(dst_vid),
            edge_type_str.to_string(),
            rank,
            props,
        );

        handle.buffer.push(BatchItem::Edge(edge));

        // If the batch size is reached, the content will be automatically flushed.
        if handle.buffer.len() >= handle.batch_size {
            if let Err(e) = handle.flush_edges() {
                let error_msg = e.to_string();
                set_last_error_message(error_msg.clone());
                handle.last_error = Some(CString::new(error_msg).unwrap_or_default());
                return graphdb_error_code_t::GRAPHDB_ERROR as c_int;
            }
        }

        graphdb_error_code_t::GRAPHDB_OK as c_int
    }
}

/// Perform batch insert operations.
///
/// # Parameters
/// - `batch`: Batch operation handle
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
/// - Caller must ensure the associated session is still valid when calling this function
/// - This function triggers the actual database write operations, which may involve I/O (Input/Output) operations.
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_flush(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let handle = &mut *(batch as *mut GraphDbBatchHandle);

        // Check the validity of the session.
        if !handle.is_session_valid() {
            return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
        }

        match handle.execute() {
            Ok(_) => graphdb_error_code_t::GRAPHDB_OK as c_int,
            Err(e) => {
                let error_msg = e.to_string();
                set_last_error_message(error_msg.clone());
                handle.last_error = Some(CString::new(error_msg).unwrap_or_default());
                graphdb_error_code_t::GRAPHDB_ERROR as c_int
            }
        }
    }
}

/// Get the number of buffered vertices.
///
/// # Parameters
/// - `batch`: Batch operation handle
///
/// # Returns
/// Number of buffered vertices
///
/// # Safety
/// - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
/// - Caller must ensure the associated session is still valid when calling this function
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_buffered_vertices(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(batch as *mut GraphDbBatchHandle);
        handle
            .buffer
            .iter()
            .filter(|item| matches!(item, BatchItem::Vertex(_)))
            .count() as c_int
    }
}

/// Get the number of buffered edges.
///
/// # Arguments
/// - `batch`: Batch operation handle
///
/// # Returns
/// - Number of buffered edges
///
/// # Safety
/// - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
/// - Caller must ensure the associated session is still valid when calling this function
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_buffered_edges(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return -1;
    }

    unsafe {
        let handle = &*(batch as *mut GraphDbBatchHandle);
        handle
            .buffer
            .iter()
            .filter(|item| matches!(item, BatchItem::Edge(_)))
            .count() as c_int
    }
}

/// Free the batch operation handle
///
/// # Arguments
/// - `batch`: Batch operation handle
///
/// # Returns
/// - Success: GRAPHDB_OK
/// - Failure: Error code
///
/// # Safety
/// - `batch` must be a valid batch handle created by `graphdb_batch_inserter_create`
/// - After calling this function, the batch handle becomes invalid and must not be used
/// - This function does not close the associated session; the caller must close the session separately
#[no_mangle]
pub unsafe extern "C" fn graphdb_batch_free(batch: *mut graphdb_batch_t) -> c_int {
    if batch.is_null() {
        return graphdb_error_code_t::GRAPHDB_MISUSE as c_int;
    }

    unsafe {
        let _ = Box::from_raw(batch as *mut GraphDbBatchHandle);
    }

    graphdb_error_code_t::GRAPHDB_OK as c_int
}

/// Convert a C value to a Rust value.
unsafe fn convert_c_value_to_rust(c_value: &graphdb_value_t) -> Value {
    match c_value.type_ {
        graphdb_value_type_t::GRAPHDB_NULL => Value::Null(crate::core::value::NullType::Null),
        graphdb_value_type_t::GRAPHDB_BOOL => Value::Bool(c_value.data.boolean),
        graphdb_value_type_t::GRAPHDB_INT => Value::Int(c_value.data.integer),
        graphdb_value_type_t::GRAPHDB_FLOAT => Value::Float(c_value.data.floating),
        graphdb_value_type_t::GRAPHDB_STRING => {
            if c_value.data.string.data.is_null() || c_value.data.string.len == 0 {
                Value::String(String::new())
            } else {
                let slice = std::slice::from_raw_parts(
                    c_value.data.string.data as *const u8,
                    c_value.data.string.len,
                );
                let s = String::from_utf8_unchecked(slice.to_vec());
                Value::String(s)
            }
        }
        _ => Value::Null(crate::core::value::NullType::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::embedded::c_api::database::{graphdb_close, graphdb_open};
    use crate::api::embedded::c_api::session::{graphdb_session_close, graphdb_session_create};
    use crate::api::embedded::c_api::types::graphdb_t;
    use std::ptr;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn create_test_db() -> *mut graphdb_t {
        let counter = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        let temp_dir = std::env::temp_dir().join("graphdb_c_api_test");
        std::fs::create_dir_all(&temp_dir).ok();
        let db_path = temp_dir.join(format!("test_batch_{}_{}.db", std::process::id(), counter));

        // Make sure the database file does not exist.
        if db_path.exists() {
            std::fs::remove_file(&db_path).ok();
            // Wait for the file system to complete the deletion operation.
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        let path_cstring = CString::new(db_path.to_str().expect("Invalid path"))
            .expect("Failed to create CString");
        let mut db: *mut graphdb_t = ptr::null_mut();

        let rc = unsafe { graphdb_open(path_cstring.as_ptr(), &mut db) };
        if rc != graphdb_error_code_t::GRAPHDB_OK as c_int {
            panic!("Failed to open database, error code: {}, path: {:?}", rc, db_path);
        }
        assert!(!db.is_null());

        db
    }

    #[test]
    fn test_batch_inserter_create_null_params() {
        let rc = unsafe { graphdb_batch_inserter_create(ptr::null_mut(), 100, ptr::null_mut()) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_batch_free_null() {
        let rc = unsafe { graphdb_batch_free(ptr::null_mut()) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_MISUSE as c_int);
    }

    #[test]
    fn test_batch_inserter_create_and_free() {
        let db = create_test_db();
        let mut session: *mut graphdb_session_t = ptr::null_mut();

        let rc = unsafe { graphdb_session_create(db, &mut session) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        let mut batch: *mut graphdb_batch_t = ptr::null_mut();
        let rc = unsafe { graphdb_batch_inserter_create(session, 100, &mut batch) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);
        assert!(!batch.is_null());

        let rc = unsafe { graphdb_batch_free(batch) };
        assert_eq!(rc, graphdb_error_code_t::GRAPHDB_OK as c_int);

        unsafe { graphdb_session_close(session) };
        unsafe { graphdb_close(db) };
    }

    #[test]
    fn test_batch_buffered_counts_null() {
        let count = unsafe { graphdb_batch_buffered_vertices(ptr::null_mut()) };
        assert_eq!(count, -1);

        let count = unsafe { graphdb_batch_buffered_edges(ptr::null_mut()) };
        assert_eq!(count, -1);
    }
}
