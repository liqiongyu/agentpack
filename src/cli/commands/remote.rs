use crate::output::{JsonEnvelope, print_json};

use super::super::args::RemoteCommands;
use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, command: &RemoteCommands) -> anyhow::Result<()> {
    match command {
        RemoteCommands::Set { url, name } => {
            super::super::util::require_yes_for_json_mutation(ctx.cli, "remote set")?;
            let repo_dir = ctx.repo.repo_dir.as_path();
            if !repo_dir.join(".git").exists() {
                let _ = crate::git::git_in(repo_dir, &["init"])?;
            }

            let has_remote =
                crate::git::git_in(repo_dir, &["remote", "get-url", name.as_str()]).is_ok();
            if has_remote {
                let _ = crate::git::git_in(
                    repo_dir,
                    &["remote", "set-url", name.as_str(), url.as_str()],
                )?;
            } else {
                let _ =
                    crate::git::git_in(repo_dir, &["remote", "add", name.as_str(), url.as_str()])?;
            }

            if ctx.cli.json {
                let envelope = JsonEnvelope::ok(
                    "remote.set",
                    serde_json::json!({
                        "repo": repo_dir.display().to_string(),
                        "repo_posix": crate::paths::path_to_posix_string(repo_dir),
                        "remote": name,
                        "url": url,
                    }),
                );
                print_json(&envelope)?;
            } else {
                println!("Set remote {name} -> {url}");
            }
        }
    }

    Ok(())
}
