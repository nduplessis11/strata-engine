# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build all crates
cargo build

# Run tests (all crates)
cargo test

# Run tests for a specific crate
cargo test -p strata
cargo test -p substrate

# Run a single test by name
cargo test -p strata test_window_manager_creation

# Run the example game (requires a display/GPU)
cargo run -p example-game

# Run the editor
cargo run -p editor
```

> Note: Integration tests involving actual window creation or Vulkan rendering cannot run in headless/CI environments. Use `cargo test` for unit tests; manually run `example-game` to verify rendering.

## Architecture

This is a Cargo workspace with four crates:

- **`crates/strata`** — The core engine library. Public API surface: `Engine`, `Game` trait, `Renderer`, `WindowManager`, `StrataError`.
- **`crates/substrate`** — Low-level utilities: arena allocator (`Arena`) and frame draw-list building. Intended as the engine's internal memory/data layer.
- **`crates/example-game`** — Reference implementation of a game using the `strata` crate.
- **`crates/editor`** — Future level editor, currently a stub that runs the engine.

### Engine Lifecycle

Games implement the `Game` trait (`name()`, `update(dt)`, `render(renderer)`), then call `Engine::new()` and `engine.run(game)`. Internally:

1. `Engine::run` creates a `winit` event loop and an `EngineApp` handler.
2. On `resumed`, `EngineApp` creates the OS window via `WindowManager`, then initializes `Renderer` (Vulkan).
3. On `RedrawRequested`, it calls `game.update(dt)` then `game.render(renderer)` each frame.

### Renderer / Vulkan

`Renderer` wraps `VulkanContext`, which owns all raw Vulkan state (entry, instance, surface, physical device, queue family, debug messenger). In debug builds (`cfg!(debug_assertions)`), validation layers and a debug messenger are automatically enabled. Physical device selection currently picks the first available device — this is the active area of development (`vk-physical-device-selection` branch).

### Error Handling

All engine errors flow through `StrataError` (defined in `crates/strata/src/error.rs`) with a `Result<T>` alias. Vulkan `vk::Result` converts via `#[from]`.

### Workspace Dependencies

All shared dependencies (`winit`, `ash`, `ash-window`, `anyhow`, `thiserror`) are declared once in the root `Cargo.toml` under `[workspace.dependencies]` and referenced with `{ workspace = true }` in each crate.
