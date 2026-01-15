use anyhow::Context as _;

use rmcp::{ServiceExt, transport::stdio};

use crate::user_error::UserError;

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &crate::cli::args::McpCommands) -> anyhow::Result<()> {
    if ctx.cli.json {
        return Err(anyhow::Error::new(UserError::new(
            "E_CONFIG_INVALID",
            "mcp serve does not support --json (stdout is reserved for MCP protocol messages)",
        )));
    }

    match command {
        crate::cli::args::McpCommands::Serve => serve(),
    }
}

fn serve() -> anyhow::Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("create tokio runtime")?;

    rt.block_on(async {
        let server = crate::mcp::AgentpackMcp::new()
            .serve(stdio())
            .await
            .context("start mcp server")?;
        server.waiting().await.context("wait for shutdown")?;
        Ok::<(), anyhow::Error>(())
    })
}
