//! `cargo run --example code_preview` — press `f` to toggle markdown source ↔ rendered view, `q` to quit.
//!
//! A tiny "source → preview" flip. Front: raw markdown with hashes and asterisks visible.
//! Back: the same text styled — headers larger and coloured, bullets as `•`, bold as bold.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::Terminal;

use ratatui_flip_panel::{FlipPanel, FlipState};

const SOURCE: &str = "\
# ratatui-flip-panel

A **two-faced** ratatui widget that flips between front and back content.

## Why

- One panel, *two* related views
- Smooth horizontal-squish animation
- `StatefulWidget` — state lives in your app

## Quick start

Press `f` to flip. That's it.";

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = FlipState::new(Duration::from_millis(350));

    loop {
        terminal.draw(|frame| {
            let area = centered(frame.area(), 60, 18);
            let widget = FlipPanel::new(draw_source, draw_rendered);
            frame.render_stateful_widget(widget, area, &mut state);
            frame.render_widget(hint(), bottom(frame.area()));
        })?;

        let timeout = state.next_tick_in().unwrap_or(Duration::from_secs(1));
        if event::poll(timeout)? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('f') => {
                        state.flip();
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

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

fn bottom(area: Rect) -> Rect {
    Rect::new(
        area.x,
        area.y + area.height.saturating_sub(2),
        area.width,
        1,
    )
}

fn draw_source(area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " source.md ",
            Style::default().fg(Color::DarkGray),
        ))
        .render(area, buf);
    if area.width <= 4 || area.height <= 4 {
        return;
    }
    let inner = Rect::new(area.x + 2, area.y + 1, area.width - 4, area.height - 2);
    let lines: Vec<Line> = SOURCE
        .lines()
        .map(|l| Line::from(Span::styled(l, Style::default().fg(Color::Gray))))
        .collect();
    Paragraph::new(lines).render(inner, buf);
}

fn draw_rendered(area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green))
        .title(Span::styled(
            " preview ",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ))
        .render(area, buf);
    if area.width <= 4 || area.height <= 4 {
        return;
    }
    let inner = Rect::new(area.x + 2, area.y + 1, area.width - 4, area.height - 2);
    let lines: Vec<Line> = SOURCE.lines().map(render_md_line).collect();
    Paragraph::new(lines).render(inner, buf);
}

fn render_md_line(line: &str) -> Line<'_> {
    if let Some(rest) = line.strip_prefix("# ") {
        return Line::from(Span::styled(
            rest.to_owned(),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        ));
    }
    if let Some(rest) = line.strip_prefix("## ") {
        return Line::from(Span::styled(
            rest.to_owned(),
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ));
    }
    if let Some(rest) = line.strip_prefix("- ") {
        let mut spans = vec![Span::styled("  • ", Style::default().fg(Color::Yellow))];
        spans.extend(inline_spans(rest));
        return Line::from(spans);
    }
    Line::from(inline_spans(line))
}

fn inline_spans(text: &str) -> Vec<Span<'_>> {
    let mut spans = Vec::new();
    let mut rest = text;
    while !rest.is_empty() {
        if let Some(after_bold) = rest.strip_prefix("**") {
            if let Some(end) = after_bold.find("**") {
                spans.push(Span::styled(
                    &after_bold[..end],
                    Style::default().add_modifier(Modifier::BOLD),
                ));
                rest = &after_bold[end + 2..];
                continue;
            }
        }
        if let Some(after_em) = rest.strip_prefix('*') {
            if let Some(end) = after_em.find('*') {
                spans.push(Span::styled(
                    &after_em[..end],
                    Style::default().add_modifier(Modifier::ITALIC),
                ));
                rest = &after_em[end + 1..];
                continue;
            }
        }
        if let Some(after_code) = rest.strip_prefix('`') {
            if let Some(end) = after_code.find('`') {
                spans.push(Span::styled(
                    &after_code[..end],
                    Style::default().fg(Color::LightYellow).bg(Color::Black),
                ));
                rest = &after_code[end + 1..];
                continue;
            }
        }
        let next_marker = rest.find(['*', '`']).unwrap_or(rest.len());
        if next_marker == 0 {
            spans.push(Span::raw(&rest[..1]));
            rest = &rest[1..];
        } else {
            spans.push(Span::raw(&rest[..next_marker]));
            rest = &rest[next_marker..];
        }
    }
    spans
}

fn hint() -> Paragraph<'static> {
    Paragraph::new(Line::from(vec![
        Span::styled("[f]", Style::default().fg(Color::Yellow)),
        Span::raw(" toggle source ↔ preview   "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
}
