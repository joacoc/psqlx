use std::{error::Error, ffi::c_char};

use psqlx_utils::{
    bindings::{
        PQExpBuffer, PsqlScanState, PsqlSettings, _backslashResult,
        _backslashResult_PSQL_CMD_ERROR, _backslashResult_PSQL_CMD_SKIP_LINE,
    },
    to_c_str, to_rust_string, MetaCommand, Plugin,
};

// Example meta-command implementation
struct ExampleCommand;

impl MetaCommand for ExampleCommand {
    fn name(&self) -> &str {
        "example"
    }

    fn execute(
        &self,
        _scan_state: PsqlScanState,
        _active_branch: bool,
        _query_buf: PQExpBuffer,
        _previous_buf: PQExpBuffer,
        _pset: PsqlSettings,
    ) -> Result<_backslashResult, Box<dyn Error>> {
        println!("Executing example command");
        Ok(_backslashResult_PSQL_CMD_SKIP_LINE)
    }
}

// Plugin implementation
struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        "example"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn meta_commands(&self) -> Vec<Box<dyn MetaCommand>> {
        vec![Box::new(ExampleCommand)]
    }
}

// The required export function that will be called by the plugin manager
#[unsafe(no_mangle)]
pub extern "C" fn name() -> *const c_char {
    to_c_str(ExamplePlugin.name())
}

pub extern "C" fn version() -> *const c_char {
    to_c_str(ExamplePlugin.version())
}

#[unsafe(no_mangle)]
pub extern "C" fn meta_commands() -> *const c_char {
    let commands = ExamplePlugin
        .meta_commands()
        .iter()
        .map(|x| x.name())
        .collect::<Vec<&str>>()
        .join(",");
    to_c_str(&commands)
}

#[unsafe(no_mangle)]
pub extern "C" fn execute_command(
    cmd: *const c_char,
    scan_state: PsqlScanState,
    active_branch: bool,
    query_buf: PQExpBuffer,
    previous_buf: PQExpBuffer,
    pset: PsqlSettings,
) -> _backslashResult {
    let cmd_str = match to_rust_string(cmd) {
        Ok(s) => s,
        Err(_) => return _backslashResult_PSQL_CMD_ERROR,
    };

    let result = ExamplePlugin.execute_command(
        cmd_str.as_str(),
        scan_state,
        active_branch,
        query_buf,
        previous_buf,
        pset,
    );

    result
}
