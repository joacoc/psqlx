use psqlx_utils::{
    ask_to_continue,
    bindings::{
        PQExpBuffer, PQerrorMessage, PsqlSettings, _backslashResult,
        _backslashResult_PSQL_CMD_ERROR, _backslashResult_PSQL_CMD_NEWEDIT,
        _backslashResult_PSQL_CMD_SKIP_LINE, appendPQExpBufferStr, puts, resetPQExpBuffer,
    },
    get_schema, pqexpbuffer_to_string,
    spinner::Spinner,
};
use ureq::json;

use crate::ai::completion;
use std::{
    error::Error,
    ffi::{CStr, CString},
};

/// Executes the "fix" command to generate a fix for a previously encountered error.
///
/// This function retrieves the last error message from the PostgreSQL session, attempts to generate a fix
/// for the associated code using the error message and schema information, and presents the fixed code to the user.
/// The user is then asked whether they want to apply the fix to the query buffer. If the user agrees,
/// the fixed code is added to the query buffer for execution.
///
/// Example:
/// ```psql
/// postgres=# SELEC 2;
/// ERROR:  syntax error at or near "SELEC"
/// LINE 1: SELEC 2;
///         ^
/// postgres=# \fix
/// SELECT 2;
///
/// Run fix? [Y/n]: Y
///
///  ?column?
/// ----------
///         2
/// (1 row)
///
/// postgres=#
/// ```
///
/// # Returns
/// - `Ok(_backslashResult_PSQL_CMD_NEWEDIT)`: If the user agrees to apply the fix, the fixed code is added
///   to the query buffer, and a new edit result is returned.
/// - `Ok(_backslashResult_PSQL_CMD_SKIP_LINE)`: If the user opts not to apply the fix, this result is returned.
/// - `Ok(_backslashResult_PSQL_CMD_ERROR)`: If any errors occur during processing, such as issues retrieving the
///   error message, generating the fix, or handling the output buffers, an error result is returned.
///
/// # Errors
/// - Returns an error if any part of the function fails, including fetching the error message, generating the fix,
///   or handling the buffers.
pub fn execute_command_fix(
    query_buf: PQExpBuffer,
    previous_buf: PQExpBuffer,
    pset: PsqlSettings,
) -> Result<_backslashResult, Box<dyn Error>> {
    let mut spinner = Spinner::start();
    let code_str = pqexpbuffer_to_string(previous_buf)?;
    let error_msg = unsafe { PQerrorMessage(pset.db) };
    let error_msg_c_str = unsafe { CStr::from_ptr(error_msg) };
    let err_msg_str = match error_msg_c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            spinner.stop();
            return Ok(_backslashResult_PSQL_CMD_ERROR);
        }
    };
    let schema_str = &get_schema(pset);

    if err_msg_str.trim().is_empty() {
        spinner.stop();
        println!("No previous error found.");
        return Ok(_backslashResult_PSQL_CMD_SKIP_LINE);
    }

    // Call the fix function
    match fix_code(&code_str, err_msg_str, schema_str) {
        Ok(fixed_code) => {
            let modified = format!("{}", fixed_code);
            spinner.stop();

            // Convert Rust string back to C string
            let fixed_c_code = match CString::new(modified) {
                Ok(c_string) => c_string.into_raw(), // Transfer ownership
                Err(_) => return Ok(_backslashResult_PSQL_CMD_ERROR),
            };

            unsafe {
                puts(fixed_c_code);
            }

            match ask_to_continue("Run fix?") {
                true => {
                    unsafe {
                        resetPQExpBuffer(query_buf);
                        appendPQExpBufferStr(query_buf, fixed_c_code);
                    }

                    return Ok(_backslashResult_PSQL_CMD_NEWEDIT);
                }
                false => {
                    println!("Fix not applied.");
                    return Ok(_backslashResult_PSQL_CMD_SKIP_LINE);
                }
            }
        }
        Err(e) => {
            spinner.stop();
            println!("Error: {}", e);
            return Ok(_backslashResult_PSQL_CMD_ERROR);
        }
    }
}

pub fn fix_code(code: &str, error_msg: &str, schema: &str) -> Result<String, Box<dyn Error>> {
    // Build the request payload
    let payload = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": format!(
                "
                    You are an expert IC engineer code assistant for PSQL.
                    Generate and return only the exact SQL it will be used as input to run again, do not use markdown and return the code formatted, nothing else.
                    Current schema of the database is:
                    {:?}
                ", schema)
            },
            {
                "role": "user",
                "content": format!("Code: {:?} {:?}", code, error_msg)
            }
        ],
        "temperature": 0.0
    });

    completion(payload)
}
