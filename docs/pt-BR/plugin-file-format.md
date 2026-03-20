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

## Payload de `read()` (Driver -> Runtime)

O `read()` deve retornar um objeto com dois mapas:

```json
{
  "sensors": {
    "sensor_1": 58.2,
    "sensor_2": 31.0
  },
  "actuators": {
    "actuator_1": 37.0
  }
}
```

Regras práticas:

- as chaves devem ser `id` de variáveis da planta
- os valores devem ser numéricos finitos
- chaves desconhecidas são ignoradas pela runtime
- se `sensors` ou `actuators` vier ausente, a runtime considera `{}` para aquele bloco

## Payload de `write(outputs)` (Runtime -> Driver)

Quando houver saída de controlador no ciclo, a runtime chama:

```python
write(outputs)
```

Formato de `outputs`:

```json
{
  "actuator_1": 42.0,
  "actuator_2": 15.5
}
```

Esse mapa já vem consolidado no ciclo, no espaço de unidades públicas da planta.

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

## Estrutura de `context.controller`

Dentro do controlador, `self.context.controller` expõe:

- `id`
- `name`
- `controller_type`
- `input_variable_ids`
- `output_variable_ids`
- `params`

Exemplo de `params`:

```json
{
  "kp": { "type": "number", "value": 1.2, "label": "Kp" },
  "enabled": { "type": "boolean", "value": true, "label": "Habilitado" },
  "mode": { "type": "string", "value": "auto", "label": "Modo" }
}
```

Uso típico no código:

- `self.context.controller.params["kp"]["value"]`
- `self.context.controller.input_variable_ids`
- `self.context.controller.output_variable_ids`

## Snapshot Básico

O snapshot de `compute()` inclui:

- `dt_s`
- `setpoints`
- `sensors`
- `actuators`
- `controller`

## Snapshot Completo de `compute(snapshot)`

Exemplo de snapshot real (simplificado):

```json
{
  "cycle_id": 17,
  "timestamp": 1710000000.123,
  "dt_s": 0.1,
  "plant": {
    "id": "plant_1",
    "name": "Forno Piloto"
  },
  "setpoints": {
    "sensor_1": 60.0
  },
  "sensors": {
    "sensor_1": 58.2
  },
  "actuators": {
    "actuator_1": 37.0
  },
  "variables_by_id": {
    "sensor_1": {
      "id": "sensor_1",
      "name": "Temperatura",
      "type": "sensor",
      "unit": "C",
      "setpoint": 60.0,
      "pv_min": 0.0,
      "pv_max": 100.0,
      "linked_sensor_ids": []
    }
  },
  "controller": {
    "id": "ctrl_1",
    "name": "PID Temperatura",
    "controller_type": "PID",
    "input_variable_ids": ["sensor_1"],
    "output_variable_ids": ["actuator_1"],
    "params": {
      "kp": { "type": "number", "value": 1.2, "label": "Kp" }
    }
  }
}
```

Leitura mais comum no `compute()`:

- PV atual: `snapshot["sensors"].get(sensor_id, 0.0)`
- SP atual: `snapshot["setpoints"].get(sensor_id, 0.0)`
- parâmetros: `self.context.controller.params`

`snapshot["actuators"]` representa o readback de atuador lido no ciclo.

## Payload de Retorno de `compute()` (Controlador -> Runtime)

`compute()` deve retornar um mapa `{actuator_id: valor}`:

```json
{
  "actuator_1": 42.0
}
```

Regras práticas:

- use IDs de atuador presentes em `output_variable_ids`
- valores devem ser numéricos finitos
- IDs não permitidos são ignorados pela runtime
- erro de tipo (ex.: string em vez de número) invalida aquele ciclo do controlador

## Unidades Públicas vs Unidades do Dispositivo

As variáveis da planta definem as unidades e limites públicos. O driver é o lugar certo para converter para o protocolo do dispositivo.

Exemplo:

- faixa pública do atuador: `0..100`
- duty cycle do dispositivo: `0..255`
- `write()` converte saída pública para a unidade crua
- `read()` converte feedback cru de volta para a unidade pública

## Fluxo Resumido de Payloads

1. Driver recebe `context.config` e `context.plant`.
2. Driver executa `read()` e retorna `sensors/actuators`.
3. Runtime monta `snapshot` para cada controlador.
4. Controlador executa `compute(snapshot)` e retorna saídas por `actuator_id`.
5. Runtime consolida as saídas e chama `driver.write(outputs)`.
