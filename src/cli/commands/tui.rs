use std::io::IsTerminal as _;
use std::time::Duration;

use anyhow::Context as _;

use crate::engine::Engine;
use crate::user_error::UserError;

use super::Ctx;

pub(crate) fn run(ctx: &Ctx<'_>) -> anyhow::Result<()> {
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

    run_ui(views.plan, views.diff, views.status)
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
struct App {
    active: View,
    plan: ViewText,
    diff: ViewText,
    status: ViewText,
    last_content_height: u16,
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

fn run_ui(plan: String, diff: String, status: String) -> anyhow::Result<()> {
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
            Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                KeyCode::Char('1') | KeyCode::Char('p') => app.active = View::Plan,
                KeyCode::Char('2') | KeyCode::Char('d') => app.active = View::Diff,
                KeyCode::Char('3') | KeyCode::Char('s') => app.active = View::Status,
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
            },
            _ => {}
        }
    }
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
    use ratatui::widgets::{Block, Borders, Paragraph, Tabs, Wrap};

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

    let help = Paragraph::new(
        "q quit | 1/2/3 or p/d/s switch | ↑/↓ or j/k scroll | PgUp/PgDn page | Home/End",
    )
    .style(Style::default().fg(Color::DarkGray));
    f.render_widget(help, chunks[2]);
}
