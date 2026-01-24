use crate::config::Manifest;
use crate::user_error::UserError;

pub fn selected_targets(manifest: &Manifest, target_filter: &str) -> anyhow::Result<Vec<String>> {
    let mut known: Vec<String> = manifest.targets.keys().cloned().collect();
    known.sort();

    match target_filter {
        "all" => {
            let missing: Vec<String> = known
                .iter()
                .filter(|t| !crate::target_registry::is_compiled_target(t))
                .cloned()
                .collect();
            if !missing.is_empty() {
                return Err(anyhow::Error::new(
                    UserError::new(
                        "E_TARGET_UNSUPPORTED",
                        "one or more configured targets are not compiled into this agentpack build",
                    )
                    .with_details(serde_json::json!({
                        "target": "all",
                        "missing": missing,
                        "compiled": crate::target_registry::COMPILED_TARGETS,
                        "reason_code": "target_not_compiled",
                        "next_actions": [
                            "inspect_help_json",
                            "edit_manifest_targets",
                            "rebuild_with_target_feature",
                            "retry_command",
                        ],
                    })),
                ));
            }
            Ok(known)
        }
        t if crate::target_registry::is_compiled_target(t) => {
            if !manifest.targets.contains_key(t) {
                return Err(anyhow::Error::new(
                    UserError::new("E_CONFIG_INVALID", format!("target not configured: {t}"))
                        .with_details(serde_json::json!({
                            "target": t,
                            "hint": "add the target under `targets:` in agentpack.yaml",
                        })),
                ));
            }
            Ok(vec![t.to_string()])
        }
        other => Err(anyhow::Error::new(
            UserError::new(
                "E_TARGET_UNSUPPORTED",
                format!("unsupported --target: {other}"),
            )
            .with_details(serde_json::json!({
                "target": other,
                "allowed": crate::target_registry::allowed_target_filters(),
                "reason_code": "target_filter_unsupported",
                "next_actions": ["inspect_help_json", "retry_with_supported_target"],
            })),
        )),
    }
}
