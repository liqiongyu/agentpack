use clap::CommandFactory as _;

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, shell: clap_complete::Shell) -> anyhow::Result<()> {
    if ctx.cli.json {
        anyhow::bail!("completions does not support --json");
    }
    let mut cmd = super::super::args::Cli::command();
    clap_complete::generate(shell, &mut cmd, "agentpack", &mut std::io::stdout());
    Ok(())
}
