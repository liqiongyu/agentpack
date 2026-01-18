use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, rebase: bool, remote: &str) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "sync")?;

    let repo_dir = ctx.repo.repo_dir.as_path();
    if !repo_dir.join(".git").exists() {
        anyhow::bail!(
            "config repo is not a git repository: {}",
            repo_dir.display()
        );
    }

    let status = crate::git::git_in(repo_dir, &["status", "--porcelain"])?;
    if !status.trim().is_empty() {
        anyhow::bail!("refusing to sync with a dirty working tree (commit or stash first)");
    }

    let branch = crate::git::git_in(repo_dir, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = branch.trim();
    if branch == "HEAD" {
        anyhow::bail!("refusing to sync on detached HEAD");
    }

    // Ensure remote exists.
    let _ = crate::git::git_in(repo_dir, &["remote", "get-url", remote])?;

    let mut ran = Vec::new();
    if rebase {
        ran.push(format!("git pull --rebase {remote} {branch}"));
        let _ = crate::git::git_in(repo_dir, &["pull", "--rebase", remote, branch])?;
    } else {
        ran.push(format!("git pull {remote} {branch}"));
        let _ = crate::git::git_in(repo_dir, &["pull", remote, branch])?;
    }

    ran.push(format!("git push {remote} {branch}"));
    let _ = crate::git::git_in(repo_dir, &["push", remote, branch])?;

    if ctx.cli.json {
        let envelope = JsonEnvelope::ok(
            "sync",
            serde_json::json!({
                "repo": repo_dir.display().to_string(),
                "repo_posix": crate::paths::path_to_posix_string(repo_dir),
                "remote": remote,
                "branch": branch,
                "rebase": rebase,
                "commands": ran,
            }),
        );
        print_json(&envelope)?;
    } else {
        println!("Synced {} ({} {})", repo_dir.display(), remote, branch);
    }

    Ok(())
}
