use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "schema",
            serde_json::json!({
                "envelope": {
                    "schema_version": 1,
                    "fields": {
                        "schema_version": "number",
                        "ok": "boolean",
                        "command": "string",
                        "version": "string",
                        "data": "object",
                        "warnings": "array[string]",
                        "errors": "array[{code,message,details?}]",
                    },
                    "error_item": {
                        "code": "string",
                        "message": "string",
                        "details": "object|null"
                    }
                },
                "commands": {
                    "help": { "data_fields": ["global_args","commands","mutating_commands","notes"] },
                    "plan": { "data_fields": ["profile","targets","changes","summary"] },
                    "diff": { "data_fields": ["profile","targets","changes","summary"] },
                    "preview": { "data_fields": ["profile","targets","plan","diff?"] },
                    "status": { "data_fields": ["profile","targets","drift","summary"] }
                },
                "path_conventions": {
                    "native": "path-like fields use OS-native separators",
                    "posix": "many payloads also include *_posix companion fields using forward slashes"
                }
            }),
        );
        print_json(&envelope)?;
    } else {
        println!("JSON envelope schema_version=1");
        println!("- keys: schema_version, ok, command, version, data, warnings, errors");
        println!("- key commands: plan/diff/preview/status (use --json to inspect)");
    }

    Ok(())
}
