# Plantas

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](../en/plants.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](plants.md)

## Criando uma Planta

Ao criar uma planta, defina:

- nome da planta
- tempo de amostragem em milissegundos
- variáveis de sensor e atuador
- uma instância de driver
- controladores opcionais

Cada atuador pode ser vinculado a um ou mais sensores para fins de UI e bindings de controle.

## Importando uma Planta

O Senamby suporta abrir um arquivo JSON para preview antes da importação. Depois da importação:

- a planta é registrada no workspace
- os dados e estatísticas importados ficam disponíveis para inspeção
- plugins referenciados são reconciliados com o workspace quando possível

## Payload Básico de Planta

Um registry persistido de planta usa esta forma básica:

```json
{
  "id": "plant_123",
  "name": "Forno 1",
  "sample_time_ms": 1000,
  "variables": [
    {
      "id": "var_0",
      "name": "Temperatura",
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

## Conectando uma Planta

Conectar uma planta inicia a runtime e a telemetria ao vivo. Durante a conexão, o Senamby:

- valida o driver e os controladores ativos
- resolve os arquivos de plugin no workspace
- prepara o ambiente Python
- envia o bootstrap para o runner Python

## Pausar e Retomar

Pausar e retomar são ações visuais de sessão. A runtime continua coletando e controlando em segundo plano enquanto a UI acumula backlog. Ao retomar, a telemetria acumulada é plotada.

## Fechando uma Planta

Fechar uma planta:

- encerra a runtime, se ela estiver conectada
- descarrega a planta da sessão atual
- preserva o arquivo persistido da planta

Regra importante de reabertura:

- quando a planta for reaberta, as instâncias de controlador começam inativas

## Removendo uma Planta

Remover uma planta:

- encerra a runtime, se necessário
- descarrega a planta da sessão
- apaga o registry persistido no workspace

## Setpoints

Setpoints são persistidos no registry da planta e, quando a planta está conectada, também enviados para a runtime em execução.
