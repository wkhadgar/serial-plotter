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

## Regras da Runtime

- a runtime só existe enquanto a planta estiver conectada
- plantas não são carregadas automaticamente no startup
- controladores podem ser atualizados em tempo real
- algumas mudanças exigem reconexão e ficam como `pending_restart`

## Backlog do Pause

Pause não interrompe o loop da runtime. O frontend apenas para de plotar temporariamente e acumula backlog. Ao retomar, a telemetria acumulada é reaplicada nos gráficos.

## Telemetria e Plotagem

O backend emite eventos achatados `plant://telemetry` para o frontend.

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
