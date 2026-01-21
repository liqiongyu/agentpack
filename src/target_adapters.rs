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
#[cfg(feature = "target-zed")]
struct ZedAdapter;
#[cfg(feature = "target-export-dir")]
struct ExportDirAdapter;

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
        crate::targets::codex::render(engine, modules, desired, warnings, roots)
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
        crate::targets::claude_code::render(engine, modules, desired, warnings, roots)
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
        crate::targets::cursor::render(engine, modules, desired, warnings, roots)
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
        crate::targets::vscode::render(engine, modules, desired, warnings, roots)
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
        crate::targets::jetbrains::render(engine, modules, desired, warnings, roots)
    }
}

#[cfg(feature = "target-zed")]
impl TargetAdapter for ZedAdapter {
    fn id(&self) -> &'static str {
        "zed"
    }

    fn render(
        &self,
        engine: &Engine,
        modules: &[&crate::config::Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        crate::targets::zed::render(engine, modules, desired, warnings, roots)
    }
}

#[cfg(feature = "target-export-dir")]
impl TargetAdapter for ExportDirAdapter {
    fn id(&self) -> &'static str {
        "export_dir"
    }

    fn render(
        &self,
        engine: &Engine,
        modules: &[&crate::config::Module],
        desired: &mut DesiredState,
        warnings: &mut Vec<String>,
        roots: &mut Vec<TargetRoot>,
    ) -> anyhow::Result<()> {
        crate::targets::export_dir::render(engine, modules, desired, warnings, roots)
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
    #[cfg(feature = "target-zed")]
    static ZED: ZedAdapter = ZedAdapter;
    #[cfg(feature = "target-export-dir")]
    static EXPORT_DIR: ExportDirAdapter = ExportDirAdapter;

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
        #[cfg(feature = "target-zed")]
        "zed" => Some(&ZED),
        #[cfg(feature = "target-export-dir")]
        "export_dir" => Some(&EXPORT_DIR),
        _ => None,
    }
}
