use std::{
    error::Error,
    ffi::{CStr, CString},
    ptr::null_mut,
};

use psqlx_utils::{
    ask_to_continue,
    bindings::{
        PQExpBuffer, PsqlScanState, PsqlSettings, _backslashResult,
        _backslashResult_PSQL_CMD_ERROR, _backslashResult_PSQL_CMD_NEWEDIT,
        _backslashResult_PSQL_CMD_SKIP_LINE, appendPQExpBufferStr, psql_scan_slash_option, puts,
        resetPQExpBuffer, slash_option_type_OT_NORMAL,
    },
    get_schema,
    spinner::Spinner,
};
use ureq::json;

use crate::ai::completion;

/// Executes the "generate" command for generating code based on schema information.
///
/// This function processes a given command option, generates code based on the schema, and
/// presents the generated code to the user. The user is then asked whether they want to run the generated code.
/// If the user agrees, the generated code is appended to the query buffer for execution.
///
/// Example:
/// ```psql
/// \generate "a query to get all users"
///
/// SELECT * FROM users;
///
/// Run code? [Y/n]: y
/// ...
/// ```
///
/// # Returns
/// - `Ok(_backslashResult_PSQL_CMD_NEWEDIT)`: If the user agrees to run the generated code, it is added
///   to the query buffer, and the result indicates that a new edit is required.
/// - `Ok(_backslashResult_PSQL_CMD_SKIP_LINE)`: If the user opts not to run the code, this result is returned.
/// - `Ok(_backslashResult_PSQL_CMD_ERROR)`: If any errors occur during processing, such as issues parsing the input
///   or generating the code, an error result is returned.
///
/// # Errors
/// - Returns an error if any part of the function fails, including processing the command input,
///   generating code, or handling the output buffers.
///
pub fn execute_command_generate(
    scan_state: PsqlScanState,
    query_buf: PQExpBuffer,
    pset: PsqlSettings,
) -> Result<_backslashResult, Box<dyn Error>> {
    let mut spinner = Spinner::start();
    let arg_text = unsafe {
        psql_scan_slash_option(scan_state, slash_option_type_OT_NORMAL, null_mut(), true)
    };

    let arg_text_c_str = unsafe { CStr::from_ptr(arg_text) };
    let arg_text_str = match arg_text_c_str.to_str() {
        Ok(s) => s,
        Err(_) => {
            spinner.stop();
            return Ok(_backslashResult_PSQL_CMD_ERROR);
        }
    };
    let schema = get_schema(pset);

    // Call the generate function
    match generate_code(arg_text_str, &schema) {
        Ok(generated_code) => {
            let modified = format!("{}", generated_code);
            spinner.stop();

            let generated_c_code = match CString::new(modified) {
                Ok(c_string) => c_string.into_raw(), // Transfer ownership
                Err(_) => return Ok(_backslashResult_PSQL_CMD_ERROR),
            };

            unsafe {
                puts(generated_c_code);
            }

            match ask_to_continue("Run code?") {
                true => {
                    unsafe {
                        resetPQExpBuffer(query_buf);
                        appendPQExpBufferStr(query_buf, generated_c_code);
                    }

                    return Ok(_backslashResult_PSQL_CMD_NEWEDIT);
                }
                false => Ok(_backslashResult_PSQL_CMD_SKIP_LINE),
            }
        }
        Err(e) => {
            spinner.stop();
            println!("Error: {}", e);
            Err(e)
        }
    }
}

fn generate_code(arg_text: &str, schema: &str) -> Result<String, Box<dyn Error>> {
    let payload = json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "system",
                "content": format!("
                    You are an expert IC engineer code assistant for PSQL.
                    Generate and return only the exact SQL it will be used as input to run again, do not use markdown and return the code formatted, nothing else.
                    Current schema of the database is:
                    {:?}
                ", schema)
            },
            {
                "role": "user",
                "content": format!("Code generation request: {:?}", arg_text)
            }
        ],
        "temperature": 0.0
    });

    completion(payload)
}
