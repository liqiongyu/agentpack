use anyhow::Context as _;

use crate::fs::write_atomic;
use crate::output::{JsonEnvelope, print_json};

use super::Ctx;

fn ensure_gitignore_contains(repo_root: &std::path::Path, line: &str) -> anyhow::Result<bool> {
    let gitignore_path = repo_root.join(".gitignore");
    let mut contents = std::fs::read_to_string(&gitignore_path).unwrap_or_default();
    let already = contents.lines().any(|l| l.trim() == line);
    if already {
        return Ok(false);
    }

    if !contents.is_empty() && !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(line);
    contents.push('\n');
    write_atomic(&gitignore_path, contents.as_bytes())
        .with_context(|| format!("write {}", gitignore_path.display()))?;
    Ok(true)
}

pub(crate) fn run(ctx: &Ctx<'_>, git: bool) -> anyhow::Result<()> {
    super::super::util::require_yes_for_json_mutation(ctx.cli, "init")?;

    ctx.repo.init_repo_skeleton().context("init repo")?;

    let mut gitignore_updated = false;
    if git {
        crate::git::git_in(&ctx.repo.repo_dir, &["init"]).context("git init")?;
        gitignore_updated |=
            ensure_gitignore_contains(&ctx.repo.repo_dir, ".agentpack.manifest.json")?;
        gitignore_updated |= ensure_gitignore_contains(&ctx.repo.repo_dir, ".DS_Store")?;
    }

    if ctx.cli.json {
        let mut data = serde_json::json!({
            "repo": ctx.repo.repo_dir,
            "repo_posix": crate::paths::path_to_posix_string(&ctx.repo.repo_dir),
        });
        if git {
            data.as_object_mut()
                .context("init json data must be an object")?
                .insert(
                "git".to_string(),
                serde_json::json!({
                    "initialized": true,
                    "gitignore_updated": gitignore_updated,
                    "gitignore_path": ctx.repo.repo_dir.join(".gitignore"),
                    "gitignore_path_posix": crate::paths::path_to_posix_string(&ctx.repo.repo_dir.join(".gitignore")),
                }),
            );
        }

        let envelope = JsonEnvelope::ok("init", data);
        print_json(&envelope)?;
    } else {
        println!(
            "Initialized agentpack repo at {}",
            ctx.repo.repo_dir.display()
        );
        if git {
            println!("Initialized git repo and ensured .gitignore (updated={gitignore_updated})");
        }
    }

    Ok(())
}
