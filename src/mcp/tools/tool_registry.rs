use rmcp::model::Tool;

use super::{
    CommonArgs, DeployApplyArgs, DoctorArgs, EvolveProposeArgs, EvolveRestoreArgs, ExplainArgs,
    PreviewArgs, RollbackArgs, StatusArgs, tool, tool_input_schema,
};

pub(super) fn tools() -> Vec<Tool> {
    vec![
        tool(
            "plan",
            "Compute plan (returns Agentpack JSON envelope).",
            tool_input_schema::<CommonArgs>(),
            true,
        ),
        tool(
            "diff",
            "Compute diff (returns Agentpack JSON envelope).",
            tool_input_schema::<CommonArgs>(),
            true,
        ),
        tool(
            "preview",
            "Preview plan (optionally include diff; returns Agentpack JSON envelope).",
            tool_input_schema::<PreviewArgs>(),
            true,
        ),
        tool(
            "status",
            "Compute drift/status (returns Agentpack JSON envelope).",
            tool_input_schema::<StatusArgs>(),
            true,
        ),
        tool(
            "doctor",
            "Run doctor checks (returns Agentpack JSON envelope; read-only).",
            tool_input_schema::<DoctorArgs>(),
            true,
        ),
        tool(
            "deploy",
            "Plan+diff (read-only; returns Agentpack JSON envelope).",
            tool_input_schema::<CommonArgs>(),
            true,
        ),
        tool(
            "deploy_apply",
            "Deploy with apply (requires yes=true).",
            tool_input_schema::<DeployApplyArgs>(),
            false,
        ),
        tool(
            "rollback",
            "Rollback to a snapshot id (requires yes=true).",
            tool_input_schema::<RollbackArgs>(),
            false,
        ),
        tool(
            "evolve_propose",
            "Propose overlay updates by capturing drifted outputs (requires yes=true when not dry_run).",
            tool_input_schema::<EvolveProposeArgs>(),
            false,
        ),
        tool(
            "evolve_restore",
            "Restore missing desired outputs (requires yes=true when not dry_run).",
            tool_input_schema::<EvolveRestoreArgs>(),
            false,
        ),
        tool(
            "explain",
            "Explain plan/diff/status provenance (returns Agentpack JSON envelope).",
            tool_input_schema::<ExplainArgs>(),
            true,
        ),
    ]
}
