# Formato de Arquivo de Plugin

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](../en/plugin-file-format.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](plugin-file-format.md)

## JSON de Plugin

O Senamby aceita arquivos JSON de plugin com esta forma básica:

```json
{
  "name": "Meu Driver",
  "kind": "driver",
  "runtime": "python",
  "entryClass": "MeuDriver",
  "sourceFile": "main.py",
  "schema": [
    {
      "name": "port",
      "type": "string",
      "description": "Porta serial"
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

Kinds suportados:

- `driver`
- `controller`

Tipos de campo de schema:

- `bool`
- `int`
- `float`
- `string`
- `list`

## Contrato Python de Driver

```python
class MeuDriver:
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

O contexto público do driver expõe apenas:

- `context.config`
- `context.plant`

## Contrato Python de Controlador

```python
class MeuControlador:
    def __init__(self, context):
        self.context = context

    def compute(self, snapshot: dict[str, object]) -> dict[str, float]:
        return {
            actuator_id: 0.0
            for actuator_id in self.context.controller.output_variable_ids
        }
```

O contexto público do controlador expõe apenas:

- `context.controller`
- `context.plant`

## Snapshot Básico

O snapshot de `compute()` inclui:

- `dt_s`
- `setpoints`
- `sensors`
- `actuators`
- `controller`

## Unidades Públicas vs Unidades do Dispositivo

As variáveis da planta definem as unidades e limites públicos. O driver é o lugar certo para converter para o protocolo do dispositivo.

Exemplo:

- faixa pública do atuador: `0..100`
- duty cycle do dispositivo: `0..255`
- `write()` converte saída pública para a unidade crua
- `read()` converte feedback cru de volta para a unidade pública
