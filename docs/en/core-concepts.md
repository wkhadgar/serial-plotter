# Core Concepts

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](core-concepts.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](../pt-BR/core-concepts.md)

## Plant

A plant is the main runtime unit in Senamby. It contains:

- variables
- one driver configuration
- zero or more controller instances
- sample time and runtime-facing metadata

## Variable

A variable describes a signal in the plant.

- `sensor`: a read value, usually plotted with PV and SP
- `atuador`: an actuator/output value, usually plotted as the manipulated variable

Each variable has:

- `id`
- `name`
- `unit`
- `setpoint`
- `pv_min`
- `pv_max`

## Driver

A driver plugin is responsible for plant I/O.

Its public runtime contract receives:

- `context.config`
- `context.plant`

It must implement:

- `connect()`
- `stop()`
- `read()`

It must also implement `write(outputs)` when active controllers are present.

## Controller

A controller plugin computes actuator outputs from the current cycle snapshot.

Its public runtime contract receives:

- `context.controller`
- `context.plant`

Its required method is:

- `compute(snapshot)`

## Runtime

The runtime is created only when a plant is connected. It runs the live cycle:

`read -> control -> write -> publish`

The frontend does not execute this loop. It reacts to status and telemetry events emitted by the backend.

## Workspace

The workspace is the persistent storage area for:

- plugins
- plants
- Python environments
