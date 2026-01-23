#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct CommonArgs {
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub machine: Option<String>,
    #[serde(default)]
    pub dry_run: Option<bool>,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct StatusArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub only: Option<Vec<StatusOnly>>,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(in crate::mcp) enum StatusOnly {
    Missing,
    Modified,
    Extra,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct DoctorArgs {
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct DeployApplyArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub adopt: Option<bool>,
    #[serde(default)]
    pub confirm_token: Option<String>,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct RollbackArgs {
    #[serde(default)]
    pub repo: Option<String>,
    pub to: String,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct PreviewArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub diff: bool,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(in crate::mcp) enum EvolveScopeArg {
    Global,
    Machine,
    Project,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct EvolveProposeArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub module_id: Option<String>,
    #[serde(default)]
    pub scope: Option<EvolveScopeArg>,
    #[serde(default)]
    pub branch: Option<String>,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Default, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct EvolveRestoreArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    #[serde(default)]
    pub module_id: Option<String>,
    #[serde(default)]
    pub yes: bool,
}

#[derive(Debug, Clone, Copy, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub(in crate::mcp) enum ExplainKindArg {
    Plan,
    Diff,
    Status,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(in crate::mcp) struct ExplainArgs {
    #[serde(flatten)]
    pub common: CommonArgs,
    pub kind: ExplainKindArg,
}
