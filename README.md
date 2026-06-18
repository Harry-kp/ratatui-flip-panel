# ratatui-flip-panel

A Ratatui widget for a two-faced panel that flips between front and back content with a card-flip animation ✨

[![crates.io badge]][ratatui-flip-panel-crate]
[![docs.rs badge]][ratatui-flip-panel-docs]
[![license badge]](./LICENSE-MIT)

## Examples

<table>
  <tr>
    <td align="center" width="50%" valign="top">
      <b>basic</b><br>
      <img src="https://raw.githubusercontent.com/Harry-kp/ratatui-flip-panel/main/assets/basic.gif?v=2" alt="basic example" />
    </td>
    <td align="center" width="50%" valign="top">
      <b>playing_card</b><br>
      <img src="https://raw.githubusercontent.com/Harry-kp/ratatui-flip-panel/main/assets/playing_card.gif?v=2" alt="playing_card example" />
    </td>
  </tr>
  <tr>
    <td align="center" width="50%" valign="top">
      <b>flashcard</b><br>
      <img src="https://raw.githubusercontent.com/Harry-kp/ratatui-flip-panel/main/assets/flashcard.gif?v=2" alt="flashcard example" />
    </td>
    <td align="center" width="50%" valign="top">
      <b>album_view</b><br>
      <img src="https://raw.githubusercontent.com/Harry-kp/ratatui-flip-panel/main/assets/album_view.gif?v=2" alt="album_view example" />
    </td>
  </tr>
  <tr>
    <td align="center" colspan="2" valign="top">
      <b>code_preview</b><br>
      <img src="https://raw.githubusercontent.com/Harry-kp/ratatui-flip-panel/main/assets/code_preview.gif?v=2" alt="code_preview example" width="60%" />
    </td>
  </tr>
</table>

Each example is self-contained — copy one whole file into your own app to bootstrap. Recording tapes for the GIFs above are committed under [`tapes/`](./tapes); regenerate with `vhs tapes/<name>.tape`.

## Features

- Pack two related views into the same panel — a front face and a back face
- Smooth horizontal-squish animation approximates a 3-D card flip in plain ASCII
- Idiomatic `StatefulWidget` API — animation state lives in your app, not the widget
- Builder methods for `min_width` (edge-on threshold) and `edge_style`
- Drop-in callbacks for the front and back render — close over any app state

## Installation

```shell
cargo add ratatui ratatui-flip-panel
```

## Usage

Construct a `FlipState` on your app struct (persists across frames), call `flip()` when the user requests a side change, and render a `FlipPanel` widget with two render callbacks each frame.

```rust
use std::time::Duration;
use ratatui::{Frame, widgets::{Paragraph, Widget}};
use ratatui_flip_panel::{FlipPanel, FlipState};

struct App {
    flip: FlipState,
}

impl App {
    fn new() -> Self {
        Self { flip: FlipState::new(Duration::from_millis(300)) }
    }

    fn on_key_f(&mut self) {
        self.flip.flip();
    }

    fn draw(&mut self, frame: &mut Frame) {
        let widget = FlipPanel::new(
            |area, buf| Paragraph::new("Front face").render(area, buf),
            |area, buf| Paragraph::new("Back face").render(area, buf),
        );
        frame.render_stateful_widget(widget, frame.area(), &mut self.flip);
    }
}
```

## Tips

- **Two faces only.** This widget is for *exactly two* related views of the same data. If you have three or more, use tabs (`tui-tabs`, `ratatui-comfy-tabs`) — multi-page flips dilute the metaphor.
- **Input handling is the host's job.** The widget is render-only. Your app decides which key (or button, or gesture) calls `state.flip()`. The example binds `f` because it's mnemonic and short, but it's purely a convention — bind whatever fits your app.
- **Works in event-driven loops.** No 30 Hz busy-loop required. Use `state.next_tick_in()` as your `event::poll` timeout: the call returns `Some(~33ms)` while flipping and `None` otherwise, so the host wakes either on real input or when the widget needs its next animation frame. See `examples/basic.rs`:

  ```rust
  let timeout = state.next_tick_in().unwrap_or(Duration::from_secs(1));
  if event::poll(timeout)? { /* handle input */ }
  // redraw + state.tick() happens via render_stateful_widget
  ```

- **Production use.** Used in [Vortix](https://github.com/Harry-kp/vortix) for the Security Guard panel (front: live verdict, back: per-connection audit trail).

## Contributing

See the [contribution guidelines](https://github.com/Harry-kp/vortix/blob/main/CONTRIBUTING.md).

## License

Licensed under either [Apache License Version 2.0](./LICENSE-APACHE) or [The MIT License](./LICENSE-MIT) at your option.

---

Copyright © 2026, Harry KP

[crates.io badge]: https://img.shields.io/crates/v/ratatui-flip-panel
[docs.rs badge]: https://img.shields.io/docsrs/ratatui-flip-panel
[license badge]: https://img.shields.io/crates/l/ratatui-flip-panel
[ratatui-flip-panel-crate]: https://crates.io/crates/ratatui-flip-panel
[ratatui-flip-panel-docs]: https://docs.rs/ratatui-flip-panel
