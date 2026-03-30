# Getting Started

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](getting-started.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](../pt-BR/getting-started.md)

## 1. Run the Desktop App

If you run Senamby from source, the desktop app lives in `apps/desktop`. The current repository scripts include:

- `pnpm --dir apps/desktop install`
- `pnpm --dir apps/desktop tauri dev`

If you run a packaged desktop build, start the application normally from your operating system.

## 2. Understand the Workspace

Senamby stores its working files under:

`Documents/Senamby/workspace`

The workspace contains:

- `drivers/` for driver plugins
- `controllers/` for controller plugins
- `plants/` for persisted plant registries
- `envs/` for reused Python environments
- `runtimes/` for connected runtime sessions

## 3. Load or Create Plugins

Before a plant can run, it needs at least one driver plugin. You can:

- create a plugin from the UI
- import a plugin JSON file
- load plugins already saved in the workspace

## 4. Create or Import a Plant

You can either:

- create a new plant in the UI
- import an existing JSON plant file for preview and registration

Each plant needs:

- a name
- a sample time
- variables
- a driver instance
- optional controllers

## 5. Connect the Plant

When you connect a plant, Senamby:

- resolves the driver and active controllers
- prepares or reuses a Python environment
- starts the runtime
- begins the `read -> control -> write -> publish` loop

## 6. Close vs Delete

- **Close plant**: unloads the plant from the current session and stops the runtime, but keeps the saved plant file
- **Remove plant**: unloads the plant and deletes its saved registry from the workspace

## 7. Reopen Behavior

Plants are not auto-loaded on application startup. A closed plant comes back only when you import or open it again.
