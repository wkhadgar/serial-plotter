# Drivers and Controllers

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](drivers-and-controllers.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](../pt-BR/drivers-and-controllers.md)

## Driver Plugins

Drivers connect Senamby to the actual device or simulator. A driver is expected to:

- read sensors
- optionally read actuator feedback
- write actuator outputs when controllers are active

Driver configuration comes from the plugin schema and is stored in the plant driver instance.

## Controller Plugins

Controllers compute actuator outputs from the current snapshot. A controller instance stores:

- identity and display name
- input bindings
- output bindings
- parameter values
- runtime status

## Live Controller Updates

While a plant is connected, controllers can be added or edited live.

- if the current Python environment can load the updated set, the runtime hot-swaps the active controller list
- if a new controller requires environment changes, it is saved as `pending_restart`

## Runtime Status

Current controller runtime statuses:

- `synced`: active configuration is already applied to the runtime
- `pending_restart`: configuration is saved, but the current runtime must be reconnected before it can use it

## Removal Rule

An active synced controller cannot be removed while it is running. It must be deactivated first.

## Public Unit Rule

Controllers and plants should work in the plant's public engineering units. Device-specific raw conversions belong in the driver.

Example:

- plant actuator range: `0..100`
- Arduino duty cycle: `0..255`
- controller output: `0..100`
- driver write conversion: `0..100 -> 0..255`
- driver readback conversion: `0..255 -> 0..100`
