use std::time::Instant;

use anyhow::Context as _;
use rmcp::model::CallToolResult;

use crate::user_error::UserError;

pub(super) async fn call_deploy_apply_tool(
    server: &super::AgentpackMcp,
    args: super::DeployApplyArgs,
) -> CallToolResult {
    let command_path = ["deploy", "--apply"];
    let meta = super::CommandMeta {
        command: "deploy",
        command_id: "deploy --apply",
        command_path: &command_path,
    };

    if !args.yes || args.common.dry_run.unwrap_or(false) {
        match call_deploy_apply_in_process(args).await {
            Ok((text, envelope)) => super::tool_result_from_envelope(text, envelope),
            Err(err) => super::tool_result_unexpected(meta, &err),
        }
    } else {
        let Some(token) = args
            .confirm_token
            .as_deref()
            .filter(|t| !t.is_empty())
            .map(ToOwned::to_owned)
        else {
            return super::tool_result_from_user_error(meta, UserError::confirm_token_required());
        };

        let binding = super::ConfirmTokenBinding::from(&args.common);
        let now = Instant::now();
        let stored_plan_hash = {
            let mut store = server
                .confirm_tokens
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            match super::confirm::validate_token(&mut store, token.as_str(), &binding, now) {
                Ok(v) => v,
                Err(err) => return super::tool_result_from_user_error(meta, err),
            }
        };

        let plan_env = match super::deploy_plan_envelope_in_process(super::CommonArgs {
            repo: args.common.repo.clone(),
            profile: args.common.profile.clone(),
            target: args.common.target.clone(),
            machine: args.common.machine.clone(),
            dry_run: args.common.dry_run,
        })
        .await
        {
            Ok(v) => v,
            Err(err) => {
                return super::tool_result_unexpected(meta, &err);
            }
        };
        let current_plan_hash = match super::confirm::compute_confirm_plan_hash(&binding, &plan_env)
        {
            Ok(v) => v,
            Err(err) => {
                return super::tool_result_unexpected(meta, &err);
            }
        };

        if current_plan_hash != stored_plan_hash {
            return super::tool_result_from_user_error(
                meta,
                UserError::confirm_token_mismatch().with_details(serde_json::json!({
                    "reason_code": "confirm_token_mismatch",
                    "next_actions": ["call_deploy", "retry_deploy_apply"],
                    "hint": "Re-run the deploy tool and ensure the apply uses the matching confirm_token.",
                    "confirm_plan_hash": current_plan_hash,
                    "expected_confirm_plan_hash": stored_plan_hash,
                })),
            );
        }

        match call_deploy_apply_in_process(args).await {
            Ok((text, envelope)) => {
                if envelope
                    .get("ok")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false)
                {
                    let mut store = server
                        .confirm_tokens
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    super::confirm::consume_token(&mut store, &token);
                }
                super::tool_result_from_envelope(text, envelope)
            }
            Err(err) => super::tool_result_unexpected(meta, &err),
        }
    }
}

async fn call_deploy_apply_in_process(
    args: super::DeployApplyArgs,
) -> anyhow::Result<(String, serde_json::Value)> {
    tokio::task::spawn_blocking(move || {
        let command_path = ["deploy", "--apply"];
        let meta = super::CommandMeta {
            command: "deploy",
            command_id: "deploy --apply",
            command_path: &command_path,
        };

        let repo_override = args.common.repo.as_ref().map(std::path::PathBuf::from);
        let profile = args.common.profile.as_deref().unwrap_or("default");
        let target = args.common.target.as_deref().unwrap_or("all");
        let machine_override = args.common.machine.as_deref();

        let result = (|| -> anyhow::Result<(String, serde_json::Value)> {
            let engine = crate::engine::Engine::load(repo_override.as_deref(), machine_override)?;
            let crate::handlers::read_only::ReadOnlyContext {
                targets,
                desired,
                plan,
                warnings,
                roots,
            } = crate::handlers::read_only::read_only_context_in(&engine, profile, target)?;

            let will_apply = !args.common.dry_run.unwrap_or(false);
            if !will_apply {
                let data =
                    crate::app::deploy_json::deploy_json_data_dry_run(profile, targets, plan);
                let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                    .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                envelope.warnings = warnings;

                let text = serde_json::to_string_pretty(&envelope)?;
                let envelope = serde_json::to_value(&envelope)?;
                return Ok((text, envelope));
            }

            let adopt = args.adopt.unwrap_or(false);
            let outcome = crate::handlers::deploy::deploy_apply_in(
                &engine,
                &plan,
                &desired,
                &roots,
                adopt,
                args.yes,
                crate::handlers::deploy::ConfirmationStyle::JsonYes {
                    command_id: "deploy --apply",
                },
            )?;

            match outcome {
                crate::handlers::deploy::DeployApplyOutcome::NoChanges => {
                    let data = crate::app::deploy_json::deploy_json_data_no_changes(
                        profile, targets, plan,
                    );
                    let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                        .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                    envelope.warnings = warnings;

                    let text = serde_json::to_string_pretty(&envelope)?;
                    let envelope = serde_json::to_value(&envelope)?;
                    Ok((text, envelope))
                }
                crate::handlers::deploy::DeployApplyOutcome::Applied { snapshot_id } => {
                    let data = crate::app::deploy_json::deploy_json_data_applied(
                        profile,
                        targets,
                        plan,
                        snapshot_id,
                    );
                    let mut envelope = crate::output::JsonEnvelope::ok(meta.command, data)
                        .with_command_meta(meta.command_id_string(), meta.command_path_vec());
                    envelope.warnings = warnings;

                    let text = serde_json::to_string_pretty(&envelope)?;
                    let envelope = serde_json::to_value(&envelope)?;
                    Ok((text, envelope))
                }
                crate::handlers::deploy::DeployApplyOutcome::NeedsConfirmation => {
                    anyhow::bail!(
                        "deploy apply requires confirmation, but confirmation was not provided"
                    )
                }
            }
        })();

        match result {
            Ok(v) => Ok(v),
            Err(err) => {
                let envelope = super::envelope_from_anyhow_error(meta, &err);
                let text = serde_json::to_string_pretty(&envelope)?;
                Ok((text, envelope))
            }
        }
    })
    .await
    .context("mcp deploy_apply handler task join")?
}
