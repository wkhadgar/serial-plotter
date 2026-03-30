# Plugin File Format

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](plugin-file-format.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](../pt-BR/plugin-file-format.md)

## Plugin JSON

Senamby accepts plugin JSON files with this basic shape:

```json
{
  "name": "My Driver",
  "kind": "driver",
  "runtime": "python",
  "entryClass": "MyDriver",
  "sourceFile": "main.py",
  "schema": [
    {
      "name": "port",
      "type": "string",
      "description": "Serial port"
    }
  ],
  "dependencies": [
    {
      "name": "pyserial",
      "version": ""
    }
  ]
}
```

Supported plugin kinds:

- `driver`
- `controller`

Supported schema field types:

- `bool`
- `int`
- `float`
- `string`
- `list`

## Driver Python Contract

```python
class MyDriver:
    def __init__(self, context):
        self.context = context

    def connect(self) -> bool:
        return True

    def stop(self) -> bool:
        return True

    def read(self) -> dict[str, dict[str, float]]:
        return {
            "sensors": {"var_0": 0.0},
            "actuators": {"var_2": 0.0}
        }

    def write(self, outputs: dict[str, float]) -> bool:
        return True
```

Driver runtime context exposes only:

- `context.config`
- `context.plant`

## Controller Python Contract

```python
class MyController:
    def __init__(self, context):
        self.context = context

    def compute(self, snapshot: dict[str, object]) -> dict[str, float]:
        return {
            actuator_id: 0.0
            for actuator_id in self.context.controller.output_variable_ids
        }
```

Controller runtime context exposes only:

- `context.controller`
- `context.plant`

## Snapshot Basics

The controller `compute()` snapshot includes:

- `dt_s`
- `setpoints`
- `sensors`
- `actuators`
- `controller`

## Public Units vs Device Units

Plant variables define public units and limits. Drivers are the right place for raw-device conversion.

Example:

- public actuator range: `0..100`
- device duty cycle: `0..255`
- `write()` converts public output to raw device output
- `read()` converts raw device feedback back to public units
