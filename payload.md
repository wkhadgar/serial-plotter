# Contrato de Payloads da Runtime

## Objetivo

Este documento descreve o contrato real entre:

- frontend Svelte/Tauri;
- commands e services em Rust;
- runner Python em `apps/desktop/src-tauri/runtime/python/runner.py`;
- plugins Python de driver e controlador.

O foco aqui e documentar:

- payloads de commands Tauri ligados a runtime;
- protocolo Rust <-> runner;
- bootstrap entregue ao processo Python;
- contexto recebido por driver e controlador;
- dados de sensores, atuadores, setpoints e parametros;
- formato de leitura, controle, escrita e telemetria.

## 1. Mapa rapido das fronteiras

### Frontend -> Tauri

Commands relevantes para runtime:

- `connect_plant`
- `disconnect_plant`
- `pause_plant`
- `resume_plant`
- `save_plant_setpoint`
- `save_controller_instance_config`
- `remove_controller_instance`
- `load_plugins`

### Rust -> runner Python

Transporte:

- JSON Lines por `stdin`.

Envelope:

```json
{
  "type": "update_setpoints",
  "payload": {}
}
```

### runner Python -> Rust

Transporte:

- JSON Lines por `stdout`.

Envelope:

```json
{
  "type": "telemetry",
  "payload": {}
}
```

### Rust -> Frontend

Eventos Tauri emitidos hoje:

- `plant://telemetry`
- `plant://status`
- `plant://error`

## 2. Payloads no boundary Tauri

## 2.1 `connect_plant`

Request:

```json
{
  "id": "plant_123"
}
```

Response:

- `PlantResponse`

## 2.2 `disconnect_plant`

Request:

```json
{
  "id": "plant_123"
}
```

Response:

- `PlantResponse`

## 2.3 `pause_plant`

Request:

```json
{
  "id": "plant_123"
}
```

Response:

- `PlantResponse`

## 2.4 `resume_plant`

Request:

```json
{
  "id": "plant_123"
}
```

Response:

- `PlantResponse`

## 2.5 `save_plant_setpoint`

Request:

```json
{
  "plant_id": "plant_123",
  "variable_id": "var_0",
  "setpoint": 42.0
}
```

Efeito:

- persiste no `registry.json` da planta;
- atualiza o `PlantStore`;
- se a planta estiver conectada, envia `update_setpoints` para o runner.

## 2.6 `save_controller_instance_config`

Request:

```json
{
  "plant_id": "plant_123",
  "controller_id": "ctrl_pid_1",
  "plugin_id": "plugin_pid",
  "name": "PID Temperatura",
  "controller_type": "PID",
  "active": true,
  "input_variable_ids": ["var_sensor_temp"],
  "output_variable_ids": ["var_valvula"],
  "params": [
    {
      "key": "kp",
      "type": "number",
      "value": 1.2,
      "label": "Kp"
    },
    {
      "key": "modo_auto",
      "type": "boolean",
      "value": true,
      "label": "Modo automatico"
    },
    {
      "key": "strategy",
      "type": "string",
      "value": "pi",
      "label": "Strategia"
    }
  ]
}
```

Regras atuais:

- persiste a instancia do controlador na planta;
- faz upsert por `controller_id`;
- pode ser chamado com a planta conectada;
- se a planta estiver conectada, o backend recalcula a lista de controladores ativos e envia `update_controllers` para o runner;
- se houver pelo menos um controlador ativo, o backend valida antes se o driver implementa `write(outputs)`.

## 2.7 `remove_controller_instance`

Request:

```json
{
  "plant_id": "plant_123",
  "controller_id": "ctrl_pid_1"
}
```

Efeito:

- remove do `PlantStore`;
- persiste no `registry.json` da planta;
- se a planta estiver conectada, recalcula o conjunto ativo e envia `update_controllers` para o runner.

## 2.8 `PlantResponse`

Formato retornado pelos commands de planta:

```json
{
  "id": "plant_123",
  "name": "Forno 1",
  "sample_time_ms": 100,
  "connected": false,
  "paused": false,
  "variables": [
    {
      "id": "var_sensor_temp",
      "name": "Temperatura",
      "type": "sensor",
      "unit": "C",
      "setpoint": 45.0,
      "pv_min": 0.0,
      "pv_max": 100.0,
      "linked_sensor_ids": []
    }
  ],
  "stats": {
    "dt": 0.0,
    "uptime": 0
  },
  "driver": {
    "plugin_id": "plugin_driver_modbus",
    "plugin_name": "Driver Modbus",
    "runtime": "python",
    "source_file": "main.py",
    "config": {
      "port": "COM3",
      "baudrate": 115200,
      "channels": [1, 2, 3]
    }
  },
  "controllers": [
    {
      "id": "ctrl_pid_1",
      "plugin_id": "plugin_pid",
      "plugin_name": "PID",
      "name": "PID Temperatura",
      "controller_type": "PID",
      "active": true,
      "input_variable_ids": ["var_sensor_temp"],
      "output_variable_ids": ["var_valvula"],
      "params": {
        "kp": {
          "type": "number",
          "value": 1.2,
          "label": "Kp"
        }
      }
    }
  ]
}
```

## 2.9 Como campos configuraveis viram payload de runtime

Existe uma cadeia importante entre plugin, planta e runtime:

1. `PluginRegistry.schema` define os campos configuraveis do plugin.
2. Quando o plugin e usado como driver, os valores escolhidos para a instancia vao para `plant.driver.config`.
3. Quando o plugin e usado como controlador, os valores da instancia vao para `plant.controllers[*].params`.
4. No `connect_plant`, o backend serializa isso no bootstrap:
   - driver: `bootstrap.driver.config`
   - controlador ativo: `bootstrap.controllers[*].params`
5. O runner entrega esses dados ao codigo Python via:
   - driver: `context.config`
   - controlador: `context.controller.params`

Em outras palavras:

- `schema` descreve o contrato configuravel;
- `config` e o payload efetivo do driver em runtime;
- `params` e o payload efetivo do controlador em runtime.

## 3. Protocolo Rust -> runner

Canal:

- `stdin` do processo Python;
- uma mensagem JSON por linha.

Regras:

- `type` e obrigatorio;
- `payload` e opcional;
- `stdout` do runner e reservado ao protocolo;
- logs tecnicos devem ir para `stderr`.

## 3.1 `init`

Carrega ou substitui o bootstrap inteiro da runtime.

```json
{
  "type": "init",
  "payload": {
    "driver": {},
    "controllers": [],
    "plant": {},
    "runtime": {}
  }
}
```

## 3.2 `start`

```json
{
  "type": "start"
}
```

Efeito:

- instancia o driver;
- valida `write(outputs)` se houver controladores ativos;
- chama `driver.connect()`;
- carrega controladores ativos;
- chama `connect()` opcional dos controladores;
- entra em modo `running`.

## 3.3 `pause`

```json
{
  "type": "pause"
}
```

Efeito:

- pausa o ciclo inteiro da runtime;
- nao ha `read`, `compute` nem `write` enquanto pausado.

## 3.4 `resume`

```json
{
  "type": "resume"
}
```

Efeito:

- retoma o ciclo inteiro;
- o relogio de `uptime_s` desconta o tempo total pausado.

## 3.5 `update_setpoints`

```json
{
  "type": "update_setpoints",
  "payload": {
    "setpoints": {
      "var_sensor_temp": 42.0,
      "var_pressao": 1.4
    }
  }
}
```

Efeito:

- atualiza `bootstrap.plant.setpoints` em memoria;
- atualiza tambem o campo `setpoint` de cada variavel correspondente em `variables` e `variables_by_id`.

## 3.6 `update_controllers`

Substitui o conjunto ativo de controladores na runtime atual.

```json
{
  "type": "update_controllers",
  "payload": {
    "controllers": [
      {
        "id": "ctrl_pid_1",
        "plugin_id": "plugin_pid",
        "plugin_name": "PID",
        "plugin_dir": "/.../workspace/controllers/PID",
        "source_file": "main.py",
        "class_name": "Pid",
        "name": "PID Temperatura",
        "controller_type": "PID",
        "active": true,
        "input_variable_ids": ["var_sensor_temp"],
        "output_variable_ids": ["var_valvula"],
        "params": {
          "kp": {
            "type": "number",
            "value": 1.2,
            "label": "Kp"
          }
        }
      }
    ]
  }
}
```

Regras:

- o runner faz hot-swap completo da lista carregada;
- controladores antigos recebem `stop()` opcional antes da troca;
- controladores novos recebem `connect()` opcional depois da carga;
- se houver controladores e o driver nao implementar `write(outputs)`, o runner rejeita a troca;
- o backend tenta bloquear isso antes, para a falha nao chegar tarde no processo Python.

## 3.7 `stop` e `shutdown`

```json
{
  "type": "shutdown"
}
```

`stop` e aceito como alias.

Efeito:

- marca `should_exit = true`;
- encerra o loop;
- executa `stop()` opcional de controladores e `driver.stop()`;
- emite `stopped`.

## 3.8 `write_outputs`

Existe no protocolo, mas hoje esta reservado.

```json
{
  "type": "write_outputs",
  "payload": {}
}
```

Comportamento atual:

- o runner responde com `warning` dizendo que o comando ainda nao e suportado.

## 4. Protocolo runner -> Rust

## 4.1 `ready`

Emitido quando o processo sobe e esta pronto para receber comandos.

```json
{
  "type": "ready",
  "payload": {
    "runtime_id": "rt_123",
    "plant_id": "plant_123",
    "driver": "Driver Mock",
    "runtime_dir": "/.../workspace/runtimes/rt_123"
  }
}
```

## 4.2 `connected`

Emitido apos o `start` concluir o handshake logico de conexao.

```json
{
  "type": "connected",
  "payload": {
    "runtime_id": "rt_123",
    "plant_id": "plant_123"
  }
}
```

## 4.3 `telemetry`

Emitido uma vez por ciclo.

```json
{
  "type": "telemetry",
  "payload": {
    "timestamp": 1710000000.0,
    "cycle_id": 8,
    "configured_sample_time_ms": 100,
    "effective_dt_ms": 100.2,
    "cycle_duration_ms": 4.7,
    "read_duration_ms": 1.0,
    "control_duration_ms": 0.8,
    "write_duration_ms": 0.5,
    "publish_duration_ms": 0.1,
    "cycle_late": false,
    "late_by_ms": 0.0,
    "phase": "publish_telemetry",
    "uptime_s": 0.8,
    "sensors": {
      "var_sensor_temp": 23.4
    },
    "actuators": {
      "var_valvula": 17.0
    },
    "actuators_read": {
      "var_valvula": 16.5
    },
    "setpoints": {
      "var_sensor_temp": 25.0
    },
    "controller_outputs": {
      "var_valvula": 17.0
    },
    "written_outputs": {
      "var_valvula": 17.0
    },
    "controller_durations_ms": {
      "ctrl_pid_1": 0.3
    }
  }
}
```

Semantica dos campos:

- `timestamp`: tempo de parede em segundos via `time.time()`;
- `cycle_id`: contador incremental do runner;
- `configured_sample_time_ms`: `sample_time_ms` configurado na planta;
- `effective_dt_ms`: intervalo efetivo desde o ciclo anterior;
- `cycle_duration_ms`: custo total do ciclo;
- `read_duration_ms`: tempo gasto em `driver.read()`;
- `control_duration_ms`: tempo total gasto em todos os `compute()`;
- `write_duration_ms`: tempo gasto em `driver.write(outputs)`;
- `publish_duration_ms`: overhead local para emissao da telemetria;
- `cycle_late`: se o ciclo terminou depois da deadline;
- `late_by_ms`: atraso em relacao a deadline planejada;
- `phase`: fase onde o overrun foi detectado;
- `uptime_s`: uptime monotonic descontando tempo pausado; no primeiro ciclo sai `0.0`;
- `sensors`: snapshot normalizado dos sensores lidos no ciclo;
- `actuators_read`: leitura direta do driver para atuadores;
- `controller_outputs`: saida agregada de todos os controladores ativos;
- `written_outputs`: payload realmente enviado para `driver.write(outputs)`;
- `actuators`: view final do estado dos atuadores publicada para a UI. Hoje ela usa `written_outputs` quando existir, senao usa `actuators_read`.

## 4.4 `cycle_overrun`

Emitido quando um ciclo excede a deadline planejada.

```json
{
  "type": "cycle_overrun",
  "payload": {
    "cycle_id": 8,
    "configured_sample_time_ms": 100,
    "cycle_duration_ms": 123.5,
    "late_by_ms": 23.5,
    "phase": "publish_telemetry"
  }
}
```

## 4.5 `warning`

```json
{
  "type": "warning",
  "payload": {
    "message": "Falha em leitura de driver: timeout"
  }
}
```

Usado para falhas operacionais recuperaveis, por exemplo:

- `read()` falhou naquele ciclo;
- um controlador falhou em `compute()`;
- `write(outputs)` falhou naquele ciclo;
- `connect()` ou `stop()` opcional retornou `False`;
- `write_outputs` ainda nao e suportado.

## 4.6 `error`

```json
{
  "type": "error",
  "payload": {
    "message": "Falha ao atualizar controladores: Driver precisa implementar write(outputs) quando houver controladores ativos"
  }
}
```

Usado para falhas de protocolo, bootstrap ou operacao que nao cabem como warning.

## 4.7 `stopped`

```json
{
  "type": "stopped",
  "payload": {
    "runtime_id": "rt_123",
    "plant_id": "plant_123"
  }
}
```

## 5. Eventos Rust -> Frontend

## 5.1 `plant://telemetry`

Hoje o Rust emite um envelope proprio com metadados da runtime e o payload bruto da telemetria do runner aninhado em `payload`.

Campos principais consumidos no frontend:

```json
{
  "plant_id": "plant_123",
  "runtime_id": "rt_123",
  "lifecycle_state": "running",
  "cycle_phase": "publish_telemetry",
  "configured_sample_time_ms": 100,
  "effective_dt_ms": 100.2,
  "cycle_late": false,
  "payload": {
    "timestamp": 1710000000.0,
    "cycle_id": 8,
    "configured_sample_time_ms": 100,
    "effective_dt_ms": 100.2,
    "cycle_duration_ms": 4.7,
    "read_duration_ms": 1.0,
    "control_duration_ms": 0.8,
    "write_duration_ms": 0.5,
    "publish_duration_ms": 0.1,
    "cycle_late": false,
    "late_by_ms": 0.0,
    "phase": "publish_telemetry",
    "uptime_s": 0.8,
    "sensors": {},
    "actuators": {},
    "setpoints": {},
    "controller_outputs": {},
    "written_outputs": {}
  }
}
```

Observacao:

- os nomes de campo emitidos pelo backend para a UI estao em `snake_case`;
- o contrato do runner continua preservado dentro de `payload`.

## 5.2 `plant://status`

Usado para refletir mudancas de lifecycle da planta.

Campos tipicos:

```json
{
  "plant_id": "plant_123",
  "runtime_id": "rt_123",
  "lifecycle_state": "ready",
  "cycle_phase": "cycle_started",
  "configured_sample_time_ms": 100,
  "effective_dt_ms": 100.0,
  "cycle_late": false
}
```

## 5.3 `plant://error`

Usado quando o runtime ou o backend detecta falha relevante para UX.

```json
{
  "plant_id": "plant_123",
  "runtime_id": "rt_123",
  "message": "Driver da planta 'Forno 1' nao foi encontrado"
}
```

Valores atuais de `lifecycle_state`:

- `created`
- `bootstrapping`
- `ready`
- `connecting`
- `running`
- `stopping`
- `stopped`
- `faulted`

Valores atuais de `cycle_phase`:

- `cycle_started`
- `read_inputs`
- `compute_controllers`
- `write_outputs`
- `publish_telemetry`
- `sleep_until_deadline`

## 6. Bootstrap oficial entregue ao runner

O payload completo entregue no `init` tem quatro blocos:

```json
{
  "driver": {},
  "controllers": [],
  "plant": {},
  "runtime": {}
}
```

## 6.1 `bootstrap.driver`

```json
{
  "plugin_id": "plugin_driver_modbus",
  "plugin_name": "Driver Modbus",
  "plugin_dir": "/.../workspace/drivers/Driver Modbus",
  "source_file": "main.py",
  "class_name": "DriverModbus",
  "config": {
    "port": "COM3",
    "baudrate": 115200,
    "channels": [1, 2, 3]
  }
}
```

Origem dos campos:

- `plugin_*`, `source_file`, `class_name`: `PluginRegistry` resolvido pelo backend;
- `config`: configuracao da instancia de driver salva dentro da planta.

Semantica:

- `class_name` vem do `entry_class` persistido do plugin;
- o runner nao infere classe Python pelo nome visual do plugin;
- `config` e o bloco que o driver deve usar para parametros configuraveis, como porta, baudrate, numero de sensores, canais, endereco, etc.

## 6.2 `bootstrap.controllers`

Contem somente os controladores ativos no momento do `connect` ou do ultimo `update_controllers`.

```json
[
  {
    "id": "ctrl_pid_1",
    "plugin_id": "plugin_pid",
    "plugin_name": "PID",
    "plugin_dir": "/.../workspace/controllers/PID",
    "source_file": "main.py",
    "class_name": "Pid",
    "name": "PID Temperatura",
    "controller_type": "PID",
    "active": true,
    "input_variable_ids": ["var_sensor_temp"],
    "output_variable_ids": ["var_valvula"],
    "params": {
      "kp": {
        "type": "number",
        "value": 1.2,
        "label": "Kp"
      },
      "modo_auto": {
        "type": "boolean",
        "value": true,
        "label": "Modo automatico"
      },
      "strategy": {
        "type": "string",
        "value": "pi",
        "label": "Strategia"
      }
    }
  }
]
```

Semantica:

- `id`: id da instancia do controlador dentro da planta;
- `plugin_id` e `plugin_name`: qual plugin implementa o controlador;
- `plugin_dir`, `source_file`, `class_name`: localizacao e classe Python real a carregar;
- `name`: nome da instancia para UI e logs;
- `controller_type`: categoria logica definida pela planta;
- `input_variable_ids`: ids das variaveis que o controlador considera como entrada;
- `output_variable_ids`: ids de atuadores que o controlador pode escrever;
- `params`: parametros configuraveis da instancia.

Contrato atual de `params`:

```json
{
  "<param_key>": {
    "type": "number | boolean | string",
    "value": "valor serializado em JSON",
    "label": "Nome amigavel"
  }
}
```

Regra importante:

- `value` deve ser coerente com o `type` declarado, mesmo que o valor viaje serializado como JSON generico.

## 6.3 `bootstrap.plant`

```json
{
  "id": "plant_123",
  "name": "Forno 1",
  "variables": [
    {
      "id": "var_sensor_temp",
      "name": "Temperatura",
      "type": "sensor",
      "unit": "C",
      "setpoint": 45.0,
      "pv_min": 0.0,
      "pv_max": 100.0,
      "linked_sensor_ids": []
    },
    {
      "id": "var_valvula",
      "name": "Valvula",
      "type": "atuador",
      "unit": "%",
      "setpoint": 0.0,
      "pv_min": 0.0,
      "pv_max": 100.0,
      "linked_sensor_ids": ["var_sensor_temp"]
    }
  ],
  "variables_by_id": {
    "var_sensor_temp": {
      "id": "var_sensor_temp",
      "name": "Temperatura",
      "type": "sensor",
      "unit": "C",
      "setpoint": 45.0,
      "pv_min": 0.0,
      "pv_max": 100.0,
      "linked_sensor_ids": []
    }
  },
  "sensors": {
    "ids": ["var_sensor_temp"],
    "count": 1,
    "variables": [
      {
        "id": "var_sensor_temp",
        "name": "Temperatura",
        "type": "sensor",
        "unit": "C",
        "setpoint": 45.0,
        "pv_min": 0.0,
        "pv_max": 100.0,
        "linked_sensor_ids": []
      }
    ],
    "variables_by_id": {
      "var_sensor_temp": {
        "id": "var_sensor_temp",
        "name": "Temperatura",
        "type": "sensor",
        "unit": "C",
        "setpoint": 45.0,
        "pv_min": 0.0,
        "pv_max": 100.0,
        "linked_sensor_ids": []
      }
    }
  },
  "actuators": {
    "ids": ["var_valvula"],
    "count": 1,
    "variables": [
      {
        "id": "var_valvula",
        "name": "Valvula",
        "type": "atuador",
        "unit": "%",
        "setpoint": 0.0,
        "pv_min": 0.0,
        "pv_max": 100.0,
        "linked_sensor_ids": ["var_sensor_temp"]
      }
    ],
    "variables_by_id": {
      "var_valvula": {
        "id": "var_valvula",
        "name": "Valvula",
        "type": "atuador",
        "unit": "%",
        "setpoint": 0.0,
        "pv_min": 0.0,
        "pv_max": 100.0,
        "linked_sensor_ids": ["var_sensor_temp"]
      }
    }
  },
  "setpoints": {
    "var_sensor_temp": 45.0,
    "var_valvula": 0.0
  }
}
```

Semantica:

- `variables`: lista completa de variaveis da planta;
- `variables_by_id`: acesso rapido por id;
- `sensors`: recorte de variaveis do tipo sensor;
- `actuators`: recorte de variaveis do tipo atuador;
- `setpoints`: mapa simples por `variable_id`.

`linked_sensor_ids`:

- opcional no modelo de planta;
- chega ao runner como lista vazia quando ausente;
- util para atuadores vinculados a sensores ou para estrategias de controle e UI.

## 6.4 `bootstrap.runtime`

```json
{
  "id": "rt_123",
  "timing": {
    "owner": "runtime",
    "clock": "monotonic",
    "strategy": "deadline",
    "sample_time_ms": 100
  },
  "supervision": {
    "owner": "rust",
    "startup_timeout_ms": 12000,
    "shutdown_timeout_ms": 4000
  },
  "paths": {
    "runtime_dir": "/.../workspace/runtimes/rt_123",
    "venv_python_path": "/.../workspace/envs/<env_hash>/.venv/bin/python",
    "runner_path": "/.../workspace/runtimes/rt_123/runner.py",
    "bootstrap_path": "/.../workspace/runtimes/rt_123/bootstrap.json"
  }
}
```

Semantica:

- `timing.owner = runtime`: quem faz o pacing fino e o proprio runner;
- `supervision.owner = rust`: quem supervisiona processo, timeout e handshake e o backend;
- `paths`: caminhos de trabalho da runtime efemera e do ambiente Python reutilizado.

## 7. Contexto recebido pelo driver

O driver e instanciado assim:

```python
driver = DriverClass(context)
```

Onde `context` contem:

```python
context.config
context.plant
context.runtime
```

### 7.1 `context.config`

Origem:

- `plant.driver.config`

Uso esperado:

- porta serial;
- baudrate;
- endereco de dispositivo;
- numero de sensores;
- numero de canais;
- flags de simulacao;
- qualquer parametro configuravel do plugin.

Exemplo:

```json
{
  "port": "/dev/ttyUSB0",
  "baudrate": 115200,
  "sensor_count": 4,
  "channels": [1, 2, 3, 4],
  "mock": false
}
```

### 7.2 `context.plant`

Contem a planta inteira ja serializada para uso em runtime:

- `context.plant.id`
- `context.plant.name`
- `context.plant.variables`
- `context.plant.variables_by_id`
- `context.plant.sensors.ids`
- `context.plant.sensors.variables`
- `context.plant.actuators.ids`
- `context.plant.actuators.variables`
- `context.plant.setpoints`

Uso esperado:

- mapear quais sensores e atuadores o driver deve publicar;
- descobrir unidades, limites, setpoints e relacoes entre variaveis;
- montar tabelas de IO internamente.

### 7.3 `context.runtime`

Contem:

- `context.runtime.id`
- `context.runtime.timing.sample_time_ms`
- `context.runtime.supervision.*`
- `context.runtime.paths.*`

Uso esperado:

- conhecer o sample time configurado;
- acessar diretorio de runtime, se necessario;
- logar metadados com o `runtime_id`.

## 8. Contrato do driver Python

Metodos obrigatorios hoje:

```python
def connect(self) -> bool: ...
def stop(self) -> bool: ...
def read(self) -> dict[str, dict[str, float]]: ...
```

Metodo condicionalmente obrigatorio:

```python
def write(self, outputs: dict[str, float]) -> bool | None: ...
```

`write(outputs)` passa a ser obrigatorio quando houver controladores ativos.

### 8.1 Retorno de `read()`

Formato exigido:

```json
{
  "sensors": {
    "var_sensor_temp": 23.4
  },
  "actuators": {
    "var_valvula": 16.5
  }
}
```

Regras de normalizacao:

- apenas ids conhecidos em `plant.sensors.ids` e `plant.actuators.ids` sao mantidos;
- valores precisam ser numericos finitos;
- ids extras sao ignorados;
- se o formato nao for um objeto com `sensors` e `actuators`, o ciclo gera `warning`.

### 8.2 Entrada de `write(outputs)`

Formato recebido:

```json
{
  "var_valvula": 17.0,
  "var_bomba": 55.0
}
```

Regras:

- as chaves sao ids de atuadores;
- o runner so aceita ids presentes em `output_variable_ids` dos controladores ativos;
- o driver deve aplicar a escrita em uma unica chamada por ciclo.

## 9. Contexto recebido pelo controlador

O controlador e instanciado assim:

```python
controller = ControllerClass(context)
```

Onde `context` contem:

```python
context.controller
context.plant
context.runtime
```

### 9.1 `context.controller`

Contem os metadados da instancia do controlador:

```json
{
  "id": "ctrl_pid_1",
  "plugin_id": "plugin_pid",
  "plugin_name": "PID",
  "plugin_dir": "/.../workspace/controllers/PID",
  "source_file": "main.py",
  "class_name": "Pid",
  "name": "PID Temperatura",
  "controller_type": "PID",
  "active": true,
  "input_variable_ids": ["var_sensor_temp"],
  "output_variable_ids": ["var_valvula"],
  "params": {
    "kp": {
      "type": "number",
      "value": 1.2,
      "label": "Kp"
    }
  }
}
```

Uso esperado:

- ler parametros da instancia;
- descobrir quais variaveis entram e quais atuadores podem ser escritos;
- manter identidade do controlador para logs e diagnostico.

### 9.2 `context.plant` e `context.runtime`

O controlador recebe o mesmo `plant` e `runtime` descritos para o driver.

Isso permite que ele combine:

- setpoints globais;
- metadados das variaveis;
- sample time;
- ids de sensores e atuadores.

## 10. Contrato do controlador Python

Metodo obrigatorio:

```python
def compute(self, snapshot: dict[str, object]) -> dict[str, float]: ...
```

Metodos opcionais:

```python
def connect(self) -> bool | None: ...
def stop(self) -> bool | None: ...
```

### 10.1 Snapshot entregue ao `compute()`

```json
{
  "cycle_id": 12,
  "timestamp": 12345.678,
  "dt_s": 0.1,
  "plant": {
    "id": "plant_123",
    "name": "Forno 1"
  },
  "setpoints": {
    "var_sensor_temp": 45.0
  },
  "sensors": {
    "var_sensor_temp": 44.8
  },
  "actuators": {
    "var_valvula": 10.0
  },
  "variables_by_id": {
    "var_sensor_temp": {
      "id": "var_sensor_temp",
      "name": "Temperatura",
      "type": "sensor",
      "unit": "C",
      "setpoint": 45.0,
      "pv_min": 0.0,
      "pv_max": 100.0,
      "linked_sensor_ids": []
    }
  },
  "controller": {
    "id": "ctrl_pid_1",
    "plugin_id": "plugin_pid",
    "plugin_name": "PID",
    "name": "PID Temperatura",
    "controller_type": "PID",
    "active": true,
    "input_variable_ids": ["var_sensor_temp"],
    "output_variable_ids": ["var_valvula"],
    "params": {
      "kp": {
        "type": "number",
        "value": 1.2,
        "label": "Kp"
      }
    }
  }
}
```

Semantica:

- `cycle_id`: contador do ciclo atual;
- `timestamp`: `time.monotonic()` no inicio do ciclo;
- `dt_s`: `effective_dt_ms / 1000.0`;
- `setpoints`: mapa atual de setpoints da planta;
- `sensors`: snapshot lido do driver neste ciclo;
- `actuators`: leitura de atuadores retornada pelo `read()` do driver neste ciclo;
- `variables_by_id`: metadados completos para consulta por id;
- `controller`: a propria instancia serializada.

### 10.2 Retorno de `compute()`

Formato esperado:

```json
{
  "var_valvula": 17.0
}
```

Regras:

- as chaves devem ser ids presentes em `output_variable_ids`;
- valores precisam ser numericos finitos;
- dois controladores nao podem escrever o mesmo `variable_id` no mesmo ciclo;
- se isso acontecer, o runner trata como erro daquele caminho e emite `warning` ou `error` conforme o caso.

## 11. Sequencia real do ciclo `read -> control -> write -> publish`

Por ciclo, o runner faz:

1. verifica se esta pausado ou parado;
2. espera a deadline monotonic do proximo ciclo;
3. incrementa `cycle_id`;
4. chama `driver.read()`;
5. normaliza `sensors` e `actuators_read`;
6. para cada controlador ativo:
   - monta o snapshot;
   - chama `compute(snapshot)`;
   - normaliza saidas;
   - agrega em `controller_outputs`;
7. se houver `controller_outputs`, chama `driver.write(outputs)` uma unica vez;
8. monta o payload final de telemetria;
9. emite `telemetry`;
10. se estourar a deadline, emite `cycle_overrun`.

## 12. Regras importantes de ownership e consistencia

- o backend e a fonte de verdade para planta, plugin, setpoint e configuracao de controlador;
- o runner e a fonte de verdade para tempo fino, sequencing por ciclo e hot loop;
- o frontend nao participa do ciclo por amostra;
- `entry_class` do plugin e resolvido no backend e entregue ao runner como `class_name`;
- a runtime nao tenta adivinhar classe Python com heuristica baseada no nome visual do plugin;
- plugins sao carregados do workspace no startup do backend;
- plantas nao sao auto-carregadas no startup.

## 13. Limitacoes atuais documentadas

- `write_outputs` ainda nao esta implementado como command funcional;
- hot update de controladores nao recompila nem recria a `.venv`; ele so troca as instancias em memoria;
- para adicionar em runtime um controlador cujo plugin nao participou da composicao do ambiente no `connect`, pode ser necessario reconectar a planta para reconstruir o ambiente Python.
