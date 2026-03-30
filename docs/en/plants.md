# Plants

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](plants.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](../pt-BR/plants.md)

## Creating a Plant

When creating a plant, define:

- plant name
- sample time in milliseconds
- sensor and actuator variables
- a driver plugin instance
- optional controller instances

Each actuator can be linked to one or more sensors for UI and control binding purposes.

## Importing a Plant

Senamby supports opening a JSON file for preview before import. After import:

- the plant is registered in the workspace
- the imported data and stats are available for inspection
- plugins referenced by the plant are reconciled against the current workspace when possible

## Basic Plant Payload

A persisted plant registry uses this basic shape:

```json
{
  "id": "plant_123",
  "name": "Oven 1",
  "sample_time_ms": 1000,
  "variables": [
    {
      "id": "var_0",
      "name": "Temperature",
      "type": "sensor",
      "unit": "C",
      "setpoint": 50.0,
      "pv_min": 0.0,
      "pv_max": 100.0
    },
    {
      "id": "var_1",
      "name": "Heater 1",
      "type": "atuador",
      "unit": "%",
      "setpoint": 0.0,
      "pv_min": 0.0,
      "pv_max": 100.0
    }
  ],
  "driver": {
    "plugin_id": "plugin_driver",
    "config": {
      "port": "/dev/ttyACM0"
    }
  },
  "controllers": []
}
```

## Connecting a Plant

Connecting a plant starts the runtime and live telemetry. During connect, Senamby:

- validates the driver and active controllers
- resolves plugin files from the workspace
- prepares the Python environment
- sends the bootstrap to the Python runner

## Pause and Resume

Pause and resume are visual session actions. The runtime keeps collecting and controlling in the background while the UI accumulates backlog. On resume, the queued telemetry is plotted.

## Closing a Plant

Closing a plant:

- stops the runtime if it is connected
- unloads the plant from the current session
- keeps the persisted plant file

Important reopen rule:

- when a closed plant is reopened, controller instances start inactive

## Removing a Plant

Removing a plant:

- stops the runtime if needed
- unloads the plant from the session
- deletes the saved plant registry from the workspace

## Setpoints

Setpoints are saved to the plant registry and, when the plant is connected, pushed to the running runtime.
