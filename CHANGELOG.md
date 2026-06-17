# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-06-18

Initial release.

### Added

- `FlipState`: animation state container with `flip()`, `tick()`, `progress()`, `width_ratio()`, `showing_back()`, `set_showing_back()`, and `next_tick_in()` for event-driven hosts.
- `FlipPanel<F, B>`: stateful widget rendering two closures (front / back) with a horizontal-squish flip animation.
- Builder methods: `min_width(u16)`, `edge_style(Style)`, `edge_char(impl Into<Cow<'static, str>>)`.
- `StatefulWidget` impls for both `FlipPanel` (by value) and `&FlipPanel` (by reference).
- `examples/basic.rs`: interactive demo using `next_tick_in()` for animation-aware event polling.
