# Payload do Driver Base

## Visão geral

O runner Python expõe a classe base `MCUDriver` e normaliza o contrato dos métodos do driver.

O construtor continua recebendo `**kwargs` com a configuração do plugin, para manter compatibilidade com drivers legados.

Antes de `connect()` ser chamado, o runner converte `self.config` em um objeto `DriverConfig` e anexa o contexto completo da planta.

## Como o driver recebe os dados

### 1. Configuração do plugin no `__init__`

Os campos cadastrados na configuração do driver chegam no construtor como `**kwargs`.

Exemplo:

```python
def __init__(self, **kwargs):
    # kwargs = {"port": "COM3", "baudrate": 115200}
    self.config = kwargs
```

Depois da instanciação, o runner reanexa `self.config` como um `DriverConfig`, preservando o acesso legado por chave:

```python
self.config["port"]
self.config["baudrate"]
```

### 2. Contexto tipado disponível a partir de `connect()`

Do `connect()` em diante, o driver também passa a ter:

- `self.params`: apenas os campos configurados no plugin.
- `self.runtime`: metadados completos do runtime/planta.
- `self.config.params`: mesmo conteúdo de `self.params`.
- `self.config.runtime`: mesmo conteúdo de `self.runtime`.
- `self.config.variables`: lista de variáveis da planta.
- `self.config.sensors`: ids das variáveis do tipo sensor.
- `self.config.actuators`: ids das variáveis do tipo atuador.
- `self.config.setpoints`: setpoints atuais por id de variável.

## Estrutura do `self.runtime`

```json
{
  "runtime_id": "rt_1",
  "plant_id": "plant_1",
  "plant_name": "Tanque 1",
  "sample_time_ms": 100,
  "variables": [
    {
      "id": "var_0",
      "name": "Temperatura",
      "type": "sensor",
      "unit": "C",
      "setpoint": 35.0,
      "pv_min": 0.0,
      "pv_max": 100.0,
      "linked_sensor_ids": []
    },
    {
      "id": "var_1",
      "name": "Resistencia",
      "type": "atuador",
      "unit": "%",
      "setpoint": 0.0,
      "pv_min": 0.0,
      "pv_max": 100.0,
      "linked_sensor_ids": ["var_0"]
    }
  ],
  "sensors": ["var_0"],
  "actuators": ["var_1"],
  "setpoints": {
    "var_0": 35.0,
    "var_1": 0.0
  },
  "driver_dir": "/.../plugins/drivers/modbus_driver",
  "runtime_dir": "/.../runtimes/rt_1",
  "venv_python_path": "/.../.venv/bin/python",
  "runner_path": "/.../runtime/python/runner.py"
}
```

## Estrutura do `self.config`

`self.config` é um objeto dict-like. Ele preserva as chaves do plugin e adiciona metadados como propriedades.

Exemplo prático:

```python
porta = self.config["port"]
baudrate = self.config["baudrate"]

sensores = self.config.sensors
atuadores = self.config.actuators
variaveis = self.config.variables
setpoints = self.config.setpoints
```

## Contrato dos métodos da base

### `connect(self) -> bool`

Entrada:

- usa `self.config`, `self.params` e `self.runtime`;
- não recebe argumentos.

Retorno esperado:

- `True`: conexão concluída;
- `False`: o runner trata como falha de conexão e encerra a inicialização.

### `reconnect(self) -> bool`

Entrada:

- usa `self.config`, `self.params` e `self.runtime`;
- não recebe argumentos.

Retorno esperado:

- `True`: reconexão concluída;
- `False`: reconexão falhou.

Observação:

- o método existe no contrato da base, mas ainda não é acionado pelo loop principal do runner.

### `stop(self) -> bool`

Entrada:

- usa `self.config`, `self.params` e `self.runtime`;
- não recebe argumentos.

Retorno esperado:

- `True`: finalização concluída;
- `False`: o runner emite `warning`, mas continua o encerramento.

### `read(self) -> dict[str, float]`

Entrada:

- usa `self.config`, `self.params` e `self.runtime`;
- não recebe argumentos.

Retorno esperado:

```json
{
  "var_0": 23.4,
  "var_2": 51.0
}
```

Regras:

- as chaves devem ser ids de variáveis do tipo sensor;
- os valores devem ser numéricos;
- valores inválidos são descartados pelo runner;
- o retorno é publicado em `telemetry.payload.sensors`.

### `send(self, outputs: dict[str, float] | None = None) -> bool`

Entrada esperada:

```json
{
  "var_1": 42.0,
  "var_3": 10.5
}
```

Regras:

- as chaves devem ser ids de variáveis do tipo atuador;
- os valores devem ser numéricos.

Retorno esperado:

- `True`: escrita aceita;
- `False`: escrita rejeitada.

Observação:

- o contrato já está tipado na base, mas o fluxo atual do runner ainda não usa `send()` no ciclo principal.

## Resumo rápido

- configuração do plugin: `self.params` ou `self.config["campo"]`;
- sensores da planta: `self.config.sensors`;
- atuadores da planta: `self.config.actuators`;
- variáveis completas e acoplamentos: `self.config.variables`;
- setpoints atuais: `self.config.setpoints`;
- leitura do driver: `read() -> dict[str, float]`;
- controle de ciclo/estado: `connect()`, `reconnect()`, `stop()`, `send()` retornam `bool`.
