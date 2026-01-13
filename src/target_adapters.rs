use crate::deploy::DesiredState;
use crate::engine::Engine;
use crate::targets::TargetRoot;

pub trait TargetAdapter {
    fn id(&self) -> &'static str;

    fn render(
        &self,
        engine: &Engine,
        modules: &[&crate::config::Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()>;
}

struct CodexAdapter;
struct ClaudeCodeAdapter;
struct CursorAdapter;
struct VscodeAdapter;

impl TargetAdapter for CodexAdapter {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn render(
        &self,
        engine: &Engine,
        modules: &[&crate::config::Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        engine.render_codex(modules, desired, warnings, roots)
    }
}

impl TargetAdapter for ClaudeCodeAdapter {
    fn id(&self) -> &'static str {
        "claude_code"
    }

    fn render(
        &self,
        engine: &Engine,
        modules: &[&crate::config::Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        engine.render_claude_code(modules, desired, warnings, roots)
    }
}

impl TargetAdapter for CursorAdapter {
    fn id(&self) -> &'static str {
        "cursor"
    }

    fn render(
        &self,
        engine: &Engine,
        modules: &[&crate::config::Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        engine.render_cursor(modules, desired, warnings, roots)
    }
}

impl TargetAdapter for VscodeAdapter {
    fn id(&self) -> &'static str {
        "vscode"
    }

    fn render(
        &self,
        engine: &Engine,
        modules: &[&crate::config::Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        engine.render_vscode(modules, desired, warnings, roots)
    }
}

pub fn adapter_for(target: &str) -> Option<&'static dyn TargetAdapter> {
    static CODEX: CodexAdapter = CodexAdapter;
    static CLAUDE: ClaudeCodeAdapter = ClaudeCodeAdapter;
    static CURSOR: CursorAdapter = CursorAdapter;
    static VSCODE: VscodeAdapter = VscodeAdapter;

    match target {
        "codex" => Some(&CODEX),
        "claude_code" => Some(&CLAUDE),
        "cursor" => Some(&CURSOR),
        "vscode" => Some(&VSCODE),
        _ => None,
    }
}
