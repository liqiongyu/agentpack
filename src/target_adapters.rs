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

#[cfg(feature = "target-codex")]
struct CodexAdapter;
#[cfg(feature = "target-claude-code")]
struct ClaudeCodeAdapter;
#[cfg(feature = "target-cursor")]
struct CursorAdapter;
#[cfg(feature = "target-vscode")]
struct VscodeAdapter;
#[cfg(feature = "target-jetbrains")]
struct JetbrainsAdapter;

#[cfg(feature = "target-codex")]
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

#[cfg(feature = "target-claude-code")]
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

#[cfg(feature = "target-cursor")]
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

#[cfg(feature = "target-vscode")]
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

#[cfg(feature = "target-jetbrains")]
impl TargetAdapter for JetbrainsAdapter {
    fn id(&self) -> &'static str {
        "jetbrains"
    }

    fn render(
        &self,
        engine: &Engine,
        modules: &[&crate::config::Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        engine.render_jetbrains(modules, desired, warnings, roots)
    }
}

pub fn adapter_for(target: &str) -> Option<&'static dyn TargetAdapter> {
    #[cfg(feature = "target-codex")]
    static CODEX: CodexAdapter = CodexAdapter;
    #[cfg(feature = "target-claude-code")]
    static CLAUDE: ClaudeCodeAdapter = ClaudeCodeAdapter;
    #[cfg(feature = "target-cursor")]
    static CURSOR: CursorAdapter = CursorAdapter;
    #[cfg(feature = "target-vscode")]
    static VSCODE: VscodeAdapter = VscodeAdapter;
    #[cfg(feature = "target-jetbrains")]
    static JETBRAINS: JetbrainsAdapter = JetbrainsAdapter;

    match target {
        #[cfg(feature = "target-codex")]
        "codex" => Some(&CODEX),
        #[cfg(feature = "target-claude-code")]
        "claude_code" => Some(&CLAUDE),
        #[cfg(feature = "target-cursor")]
        "cursor" => Some(&CURSOR),
        #[cfg(feature = "target-vscode")]
        "vscode" => Some(&VSCODE),
        #[cfg(feature = "target-jetbrains")]
        "jetbrains" => Some(&JETBRAINS),
        _ => None,
    }
}
