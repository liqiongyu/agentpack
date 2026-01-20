use std::fmt::Write as _;

use anyhow::Context as _;

use crate::deploy::{DesiredState, TargetPath};
use crate::engine::Engine;
use crate::handlers::read_only::read_only_context_in;
use crate::targets::TargetRoot;

#[derive(Debug)]
pub struct ReadOnlyTextViews {
    pub warnings: Vec<String>,
    pub plan: String,
    pub diff: String,
    pub status: String,
}

pub fn collect_read_only_text_views(
    engine: &Engine,
    profile: &str,
    target_filter: &str,
) -> anyhow::Result<ReadOnlyTextViews> {
    let crate::handlers::read_only::ReadOnlyContext {
        desired,
        plan,
        warnings,
        roots,
        ..
    } = read_only_context_in(engine, profile, target_filter)?;

    let plan_text = render_plan_text(&warnings, &plan)?;
    let diff_text = render_diff_text(&warnings, &plan, &desired)?;
    let status_text = render_status_text(&warnings, &desired, &roots)?;

    Ok(ReadOnlyTextViews {
        warnings,
        plan: plan_text,
        diff: diff_text,
        status: status_text,
    })
}

fn render_warnings(out: &mut String, warnings: &[String]) {
    if warnings.is_empty() {
        return;
    }

    for w in warnings {
        let _ = writeln!(out, "Warning: {w}");
    }
    let _ = writeln!(out);
}

fn render_plan_text(
    warnings: &[String],
    plan: &crate::deploy::PlanResult,
) -> anyhow::Result<String> {
    let mut out = String::new();
    render_warnings(&mut out, warnings);

    writeln!(
        out,
        "Plan: +{} ~{} -{}",
        plan.summary.create, plan.summary.update, plan.summary.delete
    )
    .context("write plan summary")?;
    for c in &plan.changes {
        writeln!(out, "{:?} {} {}", c.op, c.target, c.path).context("write plan change")?;
    }

    Ok(out)
}

fn render_diff_text(
    warnings: &[String],
    plan: &crate::deploy::PlanResult,
    desired: &DesiredState,
) -> anyhow::Result<String> {
    let mut out = String::new();
    render_warnings(&mut out, warnings);

    if plan.changes.is_empty() {
        writeln!(out, "(no changes)").context("write diff no-op")?;
        return Ok(out);
    }

    for c in &plan.changes {
        let path = std::path::PathBuf::from(&c.path);
        let desired_key = TargetPath {
            target: c.target.clone(),
            path: path.clone(),
        };

        let before_text = if matches!(c.op, crate::deploy::Op::Create) {
            Some(String::new())
        } else {
            crate::deploy::read_text(&path)?
        };
        let after_text = if matches!(c.op, crate::deploy::Op::Delete) {
            Some(String::new())
        } else {
            desired
                .get(&desired_key)
                .and_then(|f| String::from_utf8(f.bytes.clone()).ok())
        };

        writeln!(out).context("write diff spacer")?;
        writeln!(out, "=== {} {} ===", c.target, c.path).context("write diff header")?;
        match (before_text, after_text) {
            (Some(from), Some(to)) => {
                out.push_str(&crate::diff::unified_diff(
                    &from,
                    &to,
                    &format!("before: {}", c.path),
                    &format!("after: {}", c.path),
                ));
            }
            _ => {
                writeln!(out, "(binary or non-utf8 content; diff omitted)")
                    .context("write diff omitted")?;
            }
        }
    }

    Ok(out)
}

fn render_status_text(
    warnings: &[String],
    desired: &DesiredState,
    roots: &[TargetRoot],
) -> anyhow::Result<String> {
    let report = crate::handlers::status::status_drift_report(
        desired,
        roots,
        warnings.to_vec(),
        crate::handlers::status::ExtraScanHashMode::SkipHashes,
    )?;
    let mut drift = report.drift;
    let summary = report.summary;

    drift.sort_by(|a, b| {
        (
            a.target.as_str(),
            a.root.as_deref().unwrap_or(""),
            a.path.as_str(),
        )
            .cmp(&(
                b.target.as_str(),
                b.root.as_deref().unwrap_or(""),
                b.path.as_str(),
            ))
    });

    let mut out = String::new();
    render_warnings(&mut out, &report.warnings);
    if drift.is_empty() {
        writeln!(out, "No drift").context("write no drift")?;
        return Ok(out);
    }

    writeln!(out, "Drift ({}):", drift.len()).context("write drift header")?;
    writeln!(
        out,
        "Summary: modified={} missing={} extra={}",
        summary.modified, summary.missing, summary.extra
    )
    .context("write drift summary")?;

    let mut last_group: Option<(String, String)> = None;
    for d in drift {
        let root = d.root.as_deref().unwrap_or("<unknown>");
        let group = (d.target.clone(), root.to_string());
        if last_group.as_ref() != Some(&group) {
            writeln!(out, "Root: {} ({})", root, d.target).context("write drift root")?;
            last_group = Some(group);
        }
        writeln!(out, "- {} {}", d.kind, d.path).context("write drift item")?;
    }

    Ok(out)
}
