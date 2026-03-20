# Troubleshooting

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](troubleshooting.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](../pt-BR/troubleshooting.md)

## Plugin Not Found

Symptoms:

- connect fails
- a plant opens but cannot run

What to check:

- the plugin exists in the workspace
- the plugin `id` or name still matches the plant
- the plugin source file and registry were not deleted manually

## Controller Pending Restart

Meaning:

- the controller was saved
- the current runtime environment cannot apply it immediately

Fix:

- reconnect the plant to rebuild the runtime with the updated plugin set

## Cannot Remove Active Controller

If a controller is active and synced in a running plant, removal is blocked.

Fix:

1. deactivate the controller
2. save the plant/controller configuration
3. remove the controller

## Python Dependency Problems

If a runtime cannot start because of Python dependencies:

- verify the driver/controller dependency list
- reconnect the plant after fixing the plugin definition
- inspect the generated environment under `Documents/Senamby/workspace/envs/`

## Closed vs Deleted Plant

If a plant disappears from the session:

- it may only have been closed, not deleted
- closed plants remain saved and can be imported/opened again
- deleted plants remove the persisted registry from the workspace
