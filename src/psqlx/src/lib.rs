use plugins::{PLUGIN_REGISTRY, initialize_plugins};
use psqlx_utils::to_c_str;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

mod plugins;

use psqlx_utils::bindings::{
    _backslashResult, _backslashResult_PSQL_CMD_UNKNOWN,
    PQExpBuffer, PsqlScanState, PsqlSettings,
};

// Updated external functions to use the new plugin system
#[unsafe(no_mangle)]
pub extern "C" fn has_command_ext(cmd: *const c_char) -> c_int {
    if cmd.is_null() {
        return 0;
    }

    // TODO: Initialize once.
    if let Err(_e) = initialize_plugins() {
        return 0;
    }

    let cmd = unsafe { CStr::from_ptr(cmd) };
    if let Ok(cmd_str) = cmd.to_str() {
        if let Ok(registry) = PLUGIN_REGISTRY.read() {
            for plugin in registry.values() {
                if plugin.commands.contains(cmd_str) {
                    return 1;
                }
            }
        }
        return 0;
    }
    return 0;
}

#[unsafe(no_mangle)]
pub extern "C" fn exec_command_ext(
    cmd: *const c_char,
    scan_state: PsqlScanState,
    active_branch: bool,
    query_buf: PQExpBuffer,
    previous_buf: PQExpBuffer,
    pset: PsqlSettings,
) -> _backslashResult {
    if cmd.is_null() {
        return _backslashResult_PSQL_CMD_UNKNOWN;
    }

    let cmd = unsafe { CStr::from_ptr(cmd) };
    if let Ok(cmd_str) = cmd.to_str() {
        match cmd_str {
            _ => {
                if let Ok(registry) = PLUGIN_REGISTRY.read() {
                    for plugin in registry.values() {
                        if plugin.commands.contains(cmd_str) {
                            return unsafe {
                                (plugin.execute)(
                                    to_c_str(cmd_str),
                                    scan_state,
                                    active_branch,
                                    query_buf,
                                    previous_buf,
                                    pset,
                                )
                            };
                        }
                    }
                }
                _backslashResult_PSQL_CMD_UNKNOWN
            }
        }
    } else {
        _backslashResult_PSQL_CMD_UNKNOWN
    }
}
