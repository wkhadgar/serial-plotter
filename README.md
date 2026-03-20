# Senamby

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](docs/en/index.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](docs/pt-BR/index.md)

Senamby is a desktop workspace for creating, running, and analyzing plants driven by reusable drivers and controllers. It combines a Svelte/Tauri desktop UI, a Rust backend, and a Python runtime for plant plugins.

## What You Can Do

- Create plants with sensors and actuators
- Register reusable driver and controller plugins
- Connect a plant to a live runtime
- Plot sensor and actuator behavior in real time
- Import plants from JSON files and preview them before loading
- Configure controllers, bindings, and setpoints from the UI

## Documentation

- English: [docs/en/index.md](docs/en/index.md)
- Português (Brasil): [docs/pt-BR/index.md](docs/pt-BR/index.md)

## Quick Start

If you are running the app from source:

1. Install the frontend dependencies inside `apps/desktop`
2. Start the desktop app with Tauri
3. Create or import plugins
4. Create or import a plant
5. Connect the plant and monitor the charts

The current frontend scripts live in `apps/desktop/package.json`, including `pnpm --dir apps/desktop tauri dev`.

## Documentation Guide

- Start with [Getting Started](docs/en/getting-started.md)
- Learn the vocabulary in [Core Concepts](docs/en/core-concepts.md)
- Use [Plants](docs/en/plants.md) for plant lifecycle and runtime actions
- Use [Drivers and Controllers](docs/en/drivers-and-controllers.md) to understand plugins and live control
- Use [Plugin File Format](docs/en/plugin-file-format.md) for JSON and Python basics
