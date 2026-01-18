use std::path::Path;
use std::process::Command;

use anyhow::Context as _;

pub fn resolve_git_ref(url: &str, ref_name: &str) -> anyhow::Result<String> {
    if is_hex_sha(ref_name) {
        return Ok(ref_name.to_string());
    }

    let patterns = [
        format!("refs/heads/{ref_name}"),
        format!("refs/tags/{ref_name}"),
        format!("refs/tags/{ref_name}^{{}}"),
    ];

    let mut cmd = Command::new("git");
    cmd.arg("ls-remote").arg(url);
    for p in &patterns {
        cmd.arg(p);
    }

    let out = cmd.output().context("git ls-remote")?;
    if !out.status.success() {
        anyhow::bail!(
            "git ls-remote failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    let stdout = String::from_utf8(out.stdout).context("decode git ls-remote output")?;
    let mut peeled: Option<String> = None;
    let mut direct: Option<String> = None;

    for line in stdout.lines() {
        let mut parts = line.split_whitespace();
        let sha = parts.next().unwrap_or_default();
        let r = parts.next().unwrap_or_default();
        if sha.is_empty() || r.is_empty() {
            continue;
        }
        if r.ends_with("^{}") {
            peeled = Some(sha.to_string());
        } else {
            direct.get_or_insert_with(|| sha.to_string());
        }
    }

    peeled
        .or(direct)
        .with_context(|| format!("ref not found: {ref_name}"))
}

pub fn clone_checkout_git(
    url: &str,
    ref_name: &str,
    commit: &str,
    dest_dir: &Path,
    shallow: bool,
) -> anyhow::Result<()> {
    if dest_dir.exists() {
        return Ok(());
    }

    let tmp_dir = dest_dir.with_extension("tmp");

    let try_clone_checkout = |use_shallow: bool| -> anyhow::Result<()> {
        if tmp_dir.exists() {
            std::fs::remove_dir_all(&tmp_dir).ok();
        }

        let mut clone = Command::new("git");
        clone.arg("clone");
        if use_shallow && !is_hex_sha(ref_name) {
            clone.arg("--depth").arg("1").arg("--branch").arg(ref_name);
        }
        clone.arg(url).arg(&tmp_dir);
        let out = clone.output().context("git clone")?;
        if !out.status.success() {
            anyhow::bail!("git clone failed: {}", String::from_utf8_lossy(&out.stderr));
        }

        let mut checkout = Command::new("git");
        checkout.current_dir(&tmp_dir).arg("checkout").arg(commit);
        let out = checkout.output().context("git checkout")?;
        if !out.status.success() {
            anyhow::bail!(
                "git checkout failed: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        }

        Ok(())
    };

    let shallow_attempt = shallow && !is_hex_sha(ref_name);
    match try_clone_checkout(shallow_attempt) {
        Ok(()) => {}
        Err(err) if shallow_attempt => {
            let first_err = err.to_string();
            try_clone_checkout(false).with_context(|| {
                format!(
                    "shallow clone/checkout failed (retrying non-shallow); if this persists, set shallow=false in the module source: {first_err}"
                )
            })?;
        }
        Err(err) => return Err(err),
    }

    std::fs::rename(&tmp_dir, dest_dir).context("finalize git checkout")?;
    Ok(())
}

pub fn git_in(cwd: &Path, args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .with_context(|| format!("git {args:?}"))?;
    if !out.status.success() {
        anyhow::bail!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }
    String::from_utf8(out.stdout).context("decode git output")
}

fn is_hex_sha(s: &str) -> bool {
    if s.len() != 40 {
        return false;
    }
    s.chars().all(|c| c.is_ascii_hexdigit())
}
