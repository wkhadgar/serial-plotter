# Conceitos Centrais

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](../en/core-concepts.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](core-concepts.md)

## Planta

Uma planta é a unidade principal de execução no Senamby. Ela contém:

- variáveis
- uma configuração de driver
- zero ou mais instâncias de controlador
- tempo de amostragem e metadados de runtime

## Variável

Uma variável descreve um sinal da planta.

- `sensor`: valor lido, normalmente plotado com PV e SP
- `atuador`: valor de saída, normalmente plotado como variável manipulada

Cada variável possui:

- `id`
- `name`
- `unit`
- `setpoint`
- `pv_min`
- `pv_max`

## Driver

Um driver é o plugin responsável pelo I/O da planta.

O contrato público dele recebe:

- `context.config`
- `context.plant`

Métodos obrigatórios:

- `connect()`
- `stop()`
- `read()`

`write(outputs)` passa a ser obrigatório quando houver controladores ativos.

## Controlador

Um controlador calcula saídas de atuador a partir do snapshot do ciclo atual.

O contrato público dele recebe:

- `context.controller`
- `context.plant`

Método obrigatório:

- `compute(snapshot)`

## Runtime

A runtime só existe quando a planta está conectada. Ela executa:

`read -> control -> write -> publish`

O frontend não roda esse loop; ele apenas reage a eventos de status e telemetria emitidos pelo backend.

## Workspace

O workspace é a área persistente de armazenamento para:

- plugins
- plantas
- ambientes Python

O `PlantStore` representa apenas as plantas carregadas na sessão atual. Plantas persistidas no disco não são reabertas automaticamente.
