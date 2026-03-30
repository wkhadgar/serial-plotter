# Drivers e Controladores

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](../en/drivers-and-controllers.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](drivers-and-controllers.md)

## Plugins de Driver

Drivers conectam o Senamby ao dispositivo real ou simulador. Um driver normalmente:

- lê sensores
- opcionalmente lê feedback dos atuadores
- escreve saídas de atuador quando há controladores ativos

A configuração do driver vem do schema do plugin e é salva na instância de driver da planta.

## Plugins de Controlador

Controladores calculam saídas de atuador a partir do snapshot do ciclo atual. Uma instância de controlador guarda:

- identidade e nome de exibição
- bindings de entrada
- bindings de saída
- valores de parâmetros
- status de runtime

## Quem Recebe O Quê (Resumo Rápido)

### Driver

Recebe no construtor:

- `context.config`
- `context.plant`

Métodos de payload:

- `read()` retorna `{ "sensors": {...}, "actuators": {...} }`
- `write(outputs)` recebe `{ "actuator_id": valor }`

### Controlador

Recebe no construtor:

- `context.controller`
- `context.plant`

Método de payload:

- `compute(snapshot)` recebe snapshot do ciclo
- deve retornar `{ "actuator_id": valor }`

Conteúdo principal de `snapshot`:

- `dt_s`
- `setpoints`
- `sensors`
- `actuators`
- `controller`

Para estrutura completa e exemplos JSON, veja:

- [Formato de Arquivo de Plugin](plugin-file-format.md)
- [Comportamento da Runtime](runtime-behavior.md)

## Atualizações em Runtime

Enquanto a planta está conectada, controladores podem ser adicionados ou editados em tempo real.

- se o ambiente Python atual conseguir carregar o conjunto atualizado, a runtime faz hot swap
- se a mudança exigir reconstrução do ambiente, o controlador fica como `pending_restart`

## Status de Runtime

Status atuais de controlador:

- `synced`: a configuração já está aplicada na runtime
- `pending_restart`: a configuração foi salva, mas a runtime precisa ser reconectada

## Regra de Remoção

Um controlador ativo e sincronizado não pode ser removido enquanto estiver rodando. Ele precisa ser desativado primeiro.

## Regra de Unidade Pública

Controladores e plantas devem trabalhar nas unidades públicas da planta. Conversões cruas de dispositivo pertencem ao driver.

Exemplo:

- faixa pública do atuador: `0..100`
- duty cycle do Arduino: `0..255`
- saída do controlador: `0..100`
- conversão de escrita no driver: `0..100 -> 0..255`
- conversão de leitura no driver: `0..255 -> 0..100`
