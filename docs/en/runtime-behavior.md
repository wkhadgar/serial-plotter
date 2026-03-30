# Runtime Behavior

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](runtime-behavior.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](../pt-BR/runtime-behavior.md)

## Connect Flow

When a plant connects, the backend:

1. resolves the saved driver and active controllers
2. refreshes plugin metadata from the workspace when needed
3. prepares the Python environment
4. builds a compact bootstrap payload
5. starts the Python runner

## Live Runtime Rules

- the runtime exists only while the plant is connected
- plants are not auto-loaded on startup
- controllers can be hot-updated while connected
- some controller changes may require reconnect and become `pending_restart`

## Pause Backlog

Pause does not stop the runtime loop. The frontend stops plotting temporarily and accumulates telemetry backlog. On resume, the queued telemetry is replayed into the charts.

## Telemetry and Plotting

The backend emits flat `plant://telemetry` events to the frontend.

For actuator plots, the current frontend plotting rule is based on actuator readback from the runtime telemetry, not on the raw write command payload.

## Runtime Folders

Persistent workspace data lives under:

- `drivers/`
- `controllers/`
- `plants/`
- `envs/`

Connected runtime sessions also use:

- `runtimes/<runtime_id>/bootstrap.json`

The Python runner script is written once under the runtime root and reused across runtime sessions.
