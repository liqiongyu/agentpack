use anyhow::Context as _;

use crate::app::doctor_json::doctor_json_data;
use crate::app::doctor_next_actions::doctor_next_actions;

fn action_prefix(repo: Option<&str>, target: &str) -> String {
    let mut out = String::from("agentpack");
    if let Some(repo) = repo {
        out.push_str(&format!(" --repo {repo}"));
    }
    if target != "all" {
        out.push_str(&format!(" --target {target}"));
    }
    out
}

pub(super) async fn call_doctor_in_process(
    args: super::DoctorArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let command_path = ["doctor"];
        let meta = super::CommandMeta {
            command: "doctor",
            command_id: "doctor",
            command_path: &command_path,
        };

        let repo_override = args.repo.as_ref().map(std::path::PathBuf::from);
        let target = args.target.as_deref().unwrap_or("all");

        let engine = match crate::engine::Engine::load(repo_override.as_deref(), None) {
            Ok(v) => v,
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error(meta, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                return Ok((text, envelope));
            }
        };

        let report = crate::handlers::doctor::doctor_report_in(&engine, "default", target, false);
        let report = match report {
            Ok(v) => v,
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error(meta, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                return Ok((text, envelope));
            }
        };

        let prefix = action_prefix(args.repo.as_deref(), target);
        let next_actions =
            doctor_next_actions(&report.roots, report.needs_gitignore_fix, false, &prefix);

        let crate::handlers::doctor::DoctorReport {
            machine_id,
            roots,
            gitignore_fixes,
            warnings,
            ..
        } = report;
        let data = doctor_json_data(machine_id, roots, gitignore_fixes, &next_actions.json)?;

        let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
            .with_command_meta(meta.command_id_string(), meta.command_path_vec());
        envelope.warnings = warnings;

        let text = serde_json::to_string_pretty(&envelope)?;
        let envelope = serde_json::to_value(&envelope)?;
        Ok((text, envelope))
    })
    .await
    .context("mcp doctor handler task join")?
}
