use std::io::IsTerminal as _;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context as _;

use crate::engine::Engine;
use crate::tui_apply::ApplyOutcome;
use crate::user_error::UserError;

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>, adopt: bool) -> anyhow::Result<()> {
    if ctx.cli.json {
        return Err(anyhow::Error::new(UserError::new(
            "E_CONFIG_INVALID",
            "`tui` does not support --json output".to_string(),
        )));
    }

    if !std::io::stdout().is_terminal() {
        anyhow::bail!("tui requires a TTY");
    }

    let engine = Engine::load(ctx.cli.repo.as_deref(), ctx.cli.machine.as_deref())?;
    let views =
        crate::tui_core::collect_read_only_text_views(&engine, &ctx.cli.profile, &ctx.cli.target)?;

    let apply = ApplyConfig {
        repo: ctx.cli.repo.clone(),
        machine: ctx.cli.machine.clone(),
        profile: ctx.cli.profile.clone(),
        target: ctx.cli.target.clone(),
        adopt,
        dry_run: ctx.cli.dry_run,
    };

    run_ui(apply, views.plan, views.diff, views.status)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum View {
    Plan,
    Diff,
    Status,
}

#[derive(Debug)]
struct ViewText {
    title: &'static str,
    text: String,
    scroll_y: u16,
}

#[derive(Debug)]
struct ApplyConfig {
    repo: Option<PathBuf>,
    machine: Option<String>,
    profile: String,
    target: String,
    adopt: bool,
    dry_run: bool,
}

#[derive(Debug)]
enum Modal {
    ConfirmApply,
    Message {
        title: &'static str,
        body: String,
        is_error: bool,
    },
}

#[derive(Debug)]
struct App {
    active: View,
    plan: ViewText,
    diff: ViewText,
    status: ViewText,
    last_content_height: u16,
    apply: ApplyConfig,
    modal: Option<Modal>,
}

impl App {
    fn active_tab_index(&self) -> usize {
        match self.active {
            View::Plan => 0,
            View::Diff => 1,
            View::Status => 2,
        }
    }

    fn active_view_mut(&mut self) -> &mut ViewText {
        match self.active {
            View::Plan => &mut self.plan,
            View::Diff => &mut self.diff,
            View::Status => &mut self.status,
        }
    }
}

fn run_ui(apply: ApplyConfig, plan: String, diff: String, status: String) -> anyhow::Result<()> {
    use crossterm::execute;
    use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
    use ratatui::Terminal;
    use ratatui::backend::CrosstermBackend;

    crossterm::terminal::enable_raw_mode().context("enable raw mode")?;

    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).context("enter alt screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("create terminal")?;

    let mut app = App {
        active: View::Plan,
        plan: ViewText {
            title: "Plan",
            text: plan,
            scroll_y: 0,
        },
        diff: ViewText {
            title: "Diff",
            text: diff,
            scroll_y: 0,
        },
        status: ViewText {
            title: "Status",
            text: status,
            scroll_y: 0,
        },
        last_content_height: 0,
        apply,
        modal: None,
    };

    let result = run_event_loop(&mut terminal, &mut app);

    crossterm::terminal::disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_event_loop<B: ratatui::backend::Backend>(
    terminal: &mut ratatui::Terminal<B>,
    app: &mut App,
) -> anyhow::Result<()> {
    use crossterm::event::{self, Event, KeyCode, KeyEventKind};

    loop {
        terminal
            .draw(|f| draw_ui(f, app))
            .map_err(|e| anyhow::anyhow!("draw: {e}"))?;

        if !event::poll(Duration::from_millis(250)).context("poll event")? {
            continue;
        }

        match event::read().context("read event")? {
            Event::Key(k) if k.kind == KeyEventKind::Press => {
                if let Some(modal) = app.modal.take() {
                    match modal {
                        Modal::ConfirmApply => match k.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.modal = None;
                                apply_confirmed(app);
                            }
                            KeyCode::Char('n')
                            | KeyCode::Char('N')
                            | KeyCode::Esc
                            | KeyCode::Char('q') => {
                                app.modal = None;
                            }
                            _ => {
                                app.modal = Some(Modal::ConfirmApply);
                            }
                        },
                        Modal::Message { .. } => {
                            // Any key closes the message.
                            app.modal = None;
                        }
                    }
                    continue;
                }

                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('1') | KeyCode::Char('p') => app.active = View::Plan,
                    KeyCode::Char('2') | KeyCode::Char('d') => app.active = View::Diff,
                    KeyCode::Char('3') | KeyCode::Char('s') => app.active = View::Status,
                    KeyCode::Char('a') => {
                        if app.apply.dry_run {
                            app.modal = Some(Modal::Message {
                                title: "Info",
                                body: "Dry-run is enabled; apply is disabled.".to_string(),
                                is_error: false,
                            });
                        } else {
                            app.modal = Some(Modal::ConfirmApply);
                        }
                    }
                    KeyCode::Left => {
                        app.active = match app.active {
                            View::Plan => View::Status,
                            View::Diff => View::Plan,
                            View::Status => View::Diff,
                        };
                    }
                    KeyCode::Right => {
                        app.active = match app.active {
                            View::Plan => View::Diff,
                            View::Diff => View::Status,
                            View::Status => View::Plan,
                        };
                    }
                    KeyCode::Up | KeyCode::Char('k') => scroll_by(app, -1),
                    KeyCode::Down | KeyCode::Char('j') => scroll_by(app, 1),
                    KeyCode::PageUp => {
                        scroll_by(app, -(app.last_content_height as i32).saturating_sub(1))
                    }
                    KeyCode::PageDown => {
                        scroll_by(app, (app.last_content_height as i32).saturating_sub(1))
                    }
                    KeyCode::Home => app.active_view_mut().scroll_y = 0,
                    KeyCode::End => scroll_to_end(app),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

fn apply_confirmed(app: &mut App) {
    let result = crate::tui_apply::apply_from_tui(
        app.apply.repo.as_deref(),
        app.apply.machine.as_deref(),
        &app.apply.profile,
        &app.apply.target,
        app.apply.adopt,
        true,
    );

    match result {
        Ok(ApplyOutcome::Applied { snapshot_id }) => {
            let _ = refresh_views(app);
            app.modal = Some(Modal::Message {
                title: "Applied",
                body: format!("Snapshot: {snapshot_id}"),
                is_error: false,
            });
        }
        Ok(ApplyOutcome::NoChanges) => {
            app.modal = Some(Modal::Message {
                title: "No changes",
                body: "Nothing to apply.".to_string(),
                is_error: false,
            });
        }
        Err(err) => {
            app.modal = Some(Modal::Message {
                title: "Apply failed",
                body: format_anyhow_error(&err),
                is_error: true,
            });
        }
    }
}

fn refresh_views(app: &mut App) -> anyhow::Result<()> {
    let engine = Engine::load(app.apply.repo.as_deref(), app.apply.machine.as_deref())?;
    let views = crate::tui_core::collect_read_only_text_views(
        &engine,
        &app.apply.profile,
        &app.apply.target,
    )?;

    app.plan.text = views.plan;
    app.diff.text = views.diff;
    app.status.text = views.status;
    app.plan.scroll_y = 0;
    app.diff.scroll_y = 0;
    app.status.scroll_y = 0;

    Ok(())
}

fn format_anyhow_error(err: &anyhow::Error) -> String {
    let Some(user_err) = crate::user_error::find_user_error(err) else {
        return format!("{err:#}");
    };

    let mut out = format!("error[{}]: {}", user_err.code, user_err.message);
    if let Some(details) = user_err.details.as_ref() {
        if let Ok(pretty) = serde_json::to_string_pretty(details) {
            out.push_str("\n\n");
            out.push_str(&pretty);
        }
    }

    out
}

fn scroll_by(app: &mut App, delta: i32) {
    let view = app.active_view_mut();
    if delta == 0 {
        return;
    }
    if delta < 0 {
        let delta = (-delta) as u16;
        view.scroll_y = view.scroll_y.saturating_sub(delta);
    } else {
        view.scroll_y = view.scroll_y.saturating_add(delta as u16);
    }
}

fn scroll_to_end(app: &mut App) {
    let height = app.last_content_height.max(1) as usize;
    let view = app.active_view_mut();
    let lines = view.text.lines().count().max(1);
    let max_scroll = lines.saturating_sub(height) as u16;
    view.scroll_y = max_scroll;
}

fn draw_ui(f: &mut ratatui::Frame<'_>, app: &mut App) {
    use ratatui::layout::{Constraint, Direction, Layout};
    use ratatui::style::{Color, Modifier, Style};
    use ratatui::text::{Line, Span};
    use ratatui::widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap};

    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    let titles = [
        Line::from(Span::raw(" Plan ")),
        Line::from(Span::raw(" Diff ")),
        Line::from(Span::raw(" Status ")),
    ];
    let tabs = Tabs::new(titles)
        .select(app.active_tab_index())
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    let content_area = chunks[1];
    app.last_content_height = content_area.height;

    let view = match app.active {
        View::Plan => &mut app.plan,
        View::Diff => &mut app.diff,
        View::Status => &mut app.status,
    };

    let lines = view.text.lines().count().max(1) as u16;
    let max_scroll = lines.saturating_sub(content_area.height.max(1));
    view.scroll_y = view.scroll_y.min(max_scroll);

    let content = Paragraph::new(view.text.as_str())
        .block(Block::default().borders(Borders::ALL).title(view.title))
        .wrap(Wrap { trim: false })
        .scroll((view.scroll_y, 0));
    f.render_widget(content, content_area);

    let help_text = if app.apply.dry_run {
        "q quit | 1/2/3 or p/d/s switch | ↑/↓ or j/k scroll | PgUp/PgDn page | Home/End".to_string()
    } else {
        "q quit | a apply | 1/2/3 or p/d/s switch | ↑/↓ or j/k scroll | PgUp/PgDn page | Home/End"
            .to_string()
    };
    let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);

    if let Some(modal) = app.modal.as_ref() {
        let overlay_area = centered_rect(80, 40, area);
        f.render_widget(Clear, overlay_area);

        match modal {
            Modal::ConfirmApply => {
                let body =
                    "Apply changes?\n\nPress 'y' to apply, or 'n'/'Esc' to cancel.".to_string();
                let modal = Paragraph::new(body)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Confirm apply"),
                    )
                    .wrap(Wrap { trim: false });
                f.render_widget(modal, overlay_area);
            }
            Modal::Message {
                title,
                body,
                is_error,
            } => {
                let style = if *is_error {
                    Style::default().fg(Color::Red)
                } else {
                    Style::default()
                };
                let modal = Paragraph::new(body.as_str())
                    .style(style)
                    .block(Block::default().borders(Borders::ALL).title(*title))
                    .wrap(Wrap { trim: false });
                f.render_widget(modal, overlay_area);
            }
        }
    }
}

fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    use ratatui::layout::{Constraint, Direction, Layout};

    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
