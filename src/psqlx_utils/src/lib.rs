pub mod bindings;
pub mod spinner;

// Trait that all PSQL meta-commands must implement
pub trait MetaCommand: Send + Sync {
    fn name(&self) -> &str;
    fn execute(
        &self,
        scan_state: PsqlScanState,
        active_branch: bool,
        query_buf: PQExpBuffer,
        previous_buf: PQExpBuffer,
        pset: PsqlSettings,
    ) -> Result<_backslashResult, Box<dyn std::error::Error>>;
}

// Trait that all plugins must implement
pub trait Plugin {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn meta_commands(&self) -> Vec<Box<dyn MetaCommand>>;
    fn execute_command(
        &self,
        cmd: &str,
        scan_state: PsqlScanState,
        active_branch: bool,
        query_buf: PQExpBuffer,
        previous_buf: PQExpBuffer,
        pset: PsqlSettings,
    ) -> _backslashResult {
        self.meta_commands()
            .iter()
            .find(|meta_cmd| meta_cmd.name() == cmd)
            .unwrap()
            .execute(scan_state, active_branch, query_buf, previous_buf, pset)
            .unwrap_or(_backslashResult_PSQL_CMD_ERROR)
    }
}

use std::{
    error::Error,
    ffi::{CStr, CString},
    io::{self, Write},
};

use bindings::{PsqlScanState, _backslashResult, _backslashResult_PSQL_CMD_ERROR};

use crate::bindings::{
    ExecStatusType_PGRES_TUPLES_OK, PQExpBuffer, PQclear, PQexec, PQgetvalue, PQntuples,
    PQresultStatus, PsqlSettings, strdup,
};

pub fn to_c_str(string: &str) -> *const i8 {
    return match CString::new(string) {
        Ok(c_string) => c_string.into_raw(),
        Err(_) => std::ptr::null_mut(),
    };
}

pub fn ask_to_continue(question: &str) -> bool {
    print!("\n{} [Y/n]: ", question);
    io::stdout().flush().unwrap();

    let mut response = String::new();
    io::stdin().read_line(&mut response).unwrap();
    println!();

    return response.trim().to_lowercase() == "y" || response.trim().is_empty();
}

/// Converts a PQExpBuffer to a Rust String
///
/// # Arguments
/// * `buffer` - A pointer to a PQExpBufferData structure
///
/// # Returns
/// * `Result<String, Box<dyn Error>>` - The string content of the buffer or an error
///
/// # Safety
/// This function handles raw pointers and assumes the PQExpBuffer is valid
pub fn pqexpbuffer_to_string(buffer: PQExpBuffer) -> Result<String, Box<dyn Error>> {
    // Check if buffer is null
    if buffer.is_null() {
        return Err("Null PQExpBuffer pointer".into());
    }

    unsafe {
        // Get reference to buffer data
        let buf = &*buffer;

        // Check if data pointer is null or length is 0
        if buf.data.is_null() || buf.len == 0 {
            return Ok(String::new());
        }

        // Create a slice from the raw char pointer
        let slice = std::slice::from_raw_parts(buf.data as *const u8, buf.len);

        // Try to convert the byte slice to a String
        match std::str::from_utf8(slice) {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(format!("Invalid UTF-8 sequence in PQExpBuffer: {}", e).into()),
        }
    }
}

/// SQL query to retrieve tables, views, and their columns in the current session context.
pub const SCHEMA_QUERY: &str = r#"
SELECT json_agg(row_to_json(schema_info))
FROM (
  SELECT t.table_name, t.table_type, t.table_schema,
         c.column_name, c.data_type
  FROM information_schema.tables t
  JOIN information_schema.columns c ON t.table_name = c.table_name
  WHERE t.table_schema NOT IN ('pg_catalog', 'information_schema')
  ORDER BY t.table_schema, t.table_name, c.ordinal_position
) AS schema_info;
"#;

/// Retrieves the schema information (tables, views, and columns) for the current session context in a JSON string (`SELECT json_agg(row_to_json(schema_info))`).
///
/// This function executes a predefined SQL query (`SCHEMA_QUERY`) against the PostgreSQL database
/// to fetch details about tables, views, and columns. The query returns the schema information
/// in JSON format, which is then extracted and returned as a string.
///
/// # Arguments
/// - `pset`: The current PostgreSQL session settings (`PsqlSettings`), which contains the database connection.
///
/// # Returns
/// - A `String` containing the JSON-encoded schema information if the query is successful.
/// - An empty string if the query fails or no schema information is available.
///
/// # Example
/// ```rust
/// let schema_info = get_schema(pset);
/// println!("Schema: {}", schema_info);
/// ```
pub fn get_schema(pset: PsqlSettings) -> String {
    let schema_res = unsafe { PQexec(pset.db, to_c_str(SCHEMA_QUERY)) };

    if (unsafe { PQresultStatus(schema_res) } == ExecStatusType_PGRES_TUPLES_OK
        && unsafe { PQntuples(schema_res) } > 0)
    {
        let cached_schema = unsafe { strdup(PQgetvalue(schema_res, 0, 0)) };
        let cached_schema = unsafe { CStr::from_ptr(cached_schema) };
        let cached_schema_str = match cached_schema.to_str() {
            Ok(s) => s,
            Err(_) => "",
        };
        unsafe { PQclear(schema_res) };
        return cached_schema_str.to_owned();
    }

    unsafe { PQclear(schema_res) };
    return "".to_owned();
}
