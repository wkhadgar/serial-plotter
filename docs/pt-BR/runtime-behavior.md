# Comportamento da Runtime

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](../en/runtime-behavior.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](runtime-behavior.md)

## Fluxo de Conexão

Quando uma planta conecta, o backend:

1. resolve o driver e os controladores ativos
2. atualiza metadados de plugin a partir do workspace quando necessário
3. prepara o ambiente Python
4. monta um bootstrap compacto
5. inicia o runner Python

## Fronteiras de Payload

Na prática, os payloads passam por estas fronteiras:

1. frontend -> backend (commands Tauri)
2. backend Rust -> runner Python (bootstrap)
3. runner Python -> plugins Python (`context`, `snapshot`, `outputs`)
4. runner Python -> frontend (`plant://telemetry`)

## Bootstrap (Backend -> Runner)

O runner é iniciado com um bootstrap contendo:

- `driver`
- `controllers`
- `plant`
- `runtime`

Exemplo simplificado:

```json
{
  "driver": {
    "plugin_id": "driver_1",
    "plugin_name": "Driver Serial",
    "plugin_dir": "...",
    "source_file": "main.py",
    "class_name": "SerialDriver",
    "config": { "port": "COM3" }
  },
  "controllers": [
    {
      "id": "ctrl_1",
      "name": "PID Temperatura",
      "controller_type": "PID",
      "input_variable_ids": ["sensor_1"],
      "output_variable_ids": ["actuator_1"],
      "params": {
        "kp": { "type": "number", "value": 1.2, "label": "Kp" }
      }
    }
  ],
  "plant": {
    "id": "plant_1",
    "name": "Forno Piloto",
    "variables": [],
    "sensor_ids": ["sensor_1"],
    "actuator_ids": ["actuator_1"],
    "setpoints": { "sensor_1": 60.0 }
  },
  "runtime": {
    "id": "rt_1",
    "timing": { "sample_time_ms": 100 },
    "supervision": {},
    "paths": {}
  }
}
```

## Regras da Runtime

- a runtime só existe enquanto a planta estiver conectada
- plantas não são carregadas automaticamente no startup
- controladores podem ser atualizados em tempo real
- algumas mudanças exigem reconexão e ficam como `pending_restart`

## Ciclo `read -> control -> write -> publish`

### 1. `read`

O runner chama `driver.read()` e espera:

```json
{
  "sensors": { "sensor_1": 58.2 },
  "actuators": { "actuator_1": 37.0 }
}
```

### 2. `control`

Para cada controlador ativo, o runner monta um `snapshot` com:

- `dt_s`
- `setpoints`
- `sensors`
- `actuators`
- `variables_by_id`
- `controller`

Depois chama `compute(snapshot)` e recebe:

```json
{
  "actuator_1": 42.0
}
```

### 3. `write`

O runner consolida saídas do ciclo e chama:

```python
driver.write(outputs)
```

### 4. `publish`

O runner publica telemetria para o frontend.

## Backlog do Pause

Pause não interrompe o loop da runtime. O frontend apenas para de plotar temporariamente e acumula backlog. Ao retomar, a telemetria acumulada é reaplicada nos gráficos.

## Telemetria e Plotagem

O backend emite eventos achatados `plant://telemetry` para o frontend.

Payload principal (simplificado):

```json
{
  "plant_id": "plant_1",
  "runtime_id": "rt_1",
  "timestamp": 1710000000.123,
  "cycle_id": 17,
  "configured_sample_time_ms": 100,
  "effective_dt_ms": 100.0,
  "cycle_duration_ms": 8.4,
  "read_duration_ms": 2.1,
  "control_duration_ms": 1.4,
  "write_duration_ms": 0.8,
  "publish_duration_ms": 0.3,
  "cycle_late": false,
  "late_by_ms": 0.0,
  "phase": "publish_telemetry",
  "uptime_s": 25.6,
  "sensors": { "sensor_1": 58.2 },
  "actuators": { "actuator_1": 42.0 },
  "actuators_read": { "actuator_1": 37.0 },
  "setpoints": { "sensor_1": 60.0 },
  "controller_outputs": { "actuator_1": 42.0 },
  "written_outputs": { "actuator_1": 42.0 }
}
```

Para gráficos de atuador, a regra atual de plotagem usa o readback de atuador presente na telemetria, e não o payload bruto de `write()`.

## Pastas de Runtime

Dados persistentes do workspace ficam em:

- `drivers/`
- `controllers/`
- `plants/`
- `envs/`

Sessões conectadas também usam:

- `runtimes/<runtime_id>/bootstrap.json`

O script do runner Python é gravado uma vez na raiz de runtimes e reutilizado entre sessões.
