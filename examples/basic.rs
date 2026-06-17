//! `cargo run --example basic` — press `f` to flip, `q` to quit.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::Terminal;

use ratatui_flip_panel::{FlipPanel, FlipState};

/// How long to block in `event::poll` when nothing is animating.
/// Arbitrary; needs to be finite so the loop re-checks state. 1 s
/// is unobtrusive in interactive use.
const IDLE_POLL: Duration = Duration::from_secs(1);

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = FlipState::new(Duration::from_millis(300));
    let mut flip_count = 0u32;

    loop {
        terminal.draw(|frame| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(3), Constraint::Length(3)])
                .split(frame.area());

            // The widget itself. Front + back render callbacks close
            // over our local state.
            let widget = FlipPanel::new(
                |area, buf| {
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Front (summary) ")
                        .render(area, buf);
                    Paragraph::new(vec![
                        Line::from(""),
                        Line::from("  This is the FRONT face."),
                        Line::from(""),
                        Line::from(Span::styled(
                            "  Press `f` to flip to the back.",
                            Style::default().fg(Color::Cyan),
                        )),
                        Line::from(""),
                        Line::from(format!("  Flips so far: {flip_count}")),
                    ])
                    .render(inner_padded(area), buf);
                },
                |area, buf| {
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Back (detail) ")
                        .border_style(Style::default().fg(Color::Magenta))
                        .render(area, buf);
                    Paragraph::new(vec![
                        Line::from(""),
                        Line::from(Span::styled(
                            "  Welcome to the BACK face!",
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Line::from(""),
                        Line::from("  Same panel, different content."),
                        Line::from(""),
                        Line::from(Span::styled(
                            "  Press `f` to flip back to the front.",
                            Style::default().fg(Color::Cyan),
                        )),
                    ])
                    .render(inner_padded(area), buf);
                },
            );
            frame.render_stateful_widget(widget, chunks[0], &mut state);

            // Hint bar.
            let hint = Paragraph::new(Line::from(vec![
                Span::raw("  "),
                Span::styled("[f]", Style::default().fg(Color::Yellow)),
                Span::raw(" flip   "),
                Span::styled("[q]", Style::default().fg(Color::Yellow)),
                Span::raw(" quit"),
            ]))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" ratatui-flip-panel demo "),
            );
            frame.render_widget(hint, chunks[1]);
        })?;

        // Event-driven idle, animation-aware wake. `next_tick_in()`
        // returns Some(~33ms) while flipping and None otherwise; the
        // host's event poll therefore wakes either on real input or
        // when the widget needs its next animation frame. No 30 Hz
        // busy-loop required.
        let timeout = state.next_tick_in().unwrap_or(IDLE_POLL);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('f') if state.flip() => {
                        flip_count = flip_count.saturating_add(1);
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

/// 1-cell padding inside a bordered block, while protecting against
/// over-narrow areas (which the flip animation produces near the
/// midpoint).
fn inner_padded(area: ratatui::layout::Rect) -> ratatui::layout::Rect {
    let pad_x: u16 = if area.width >= 4 { 2 } else { 0 };
    let pad_y: u16 = u16::from(area.height >= 4);
    ratatui::layout::Rect {
        x: area.x + pad_x,
        y: area.y + pad_y,
        width: area.width.saturating_sub(pad_x * 2),
        height: area.height.saturating_sub(pad_y * 2),
    }
}
