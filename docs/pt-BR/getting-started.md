# Primeiros Passos

[![English](https://img.shields.io/badge/Language-English-2563eb?style=for-the-badge)](../en/getting-started.md)
[![Português](https://img.shields.io/badge/Idioma-Portugu%C3%AAs-16a34a?style=for-the-badge)](getting-started.md)

## 1. Execute a Aplicação Desktop

Se você executa o Senamby a partir do código-fonte, a aplicação desktop fica em `apps/desktop`. Os scripts atuais do repositório incluem:

- `pnpm --dir apps/desktop install`
- `pnpm --dir apps/desktop tauri dev`

Se você usa um build empacotado, basta abrir a aplicação normalmente no sistema operacional.

## 2. Entenda o Workspace

O Senamby guarda seus arquivos em:

`Documents/Senamby/workspace`

O workspace contém:

- `drivers/` para plugins de driver
- `controllers/` para plugins de controlador
- `plants/` para registries persistidos de planta
- `envs/` para ambientes Python reutilizados
- `runtimes/` para sessões conectadas

## 3. Carregue ou Crie Plugins

Antes de uma planta rodar, ela precisa de pelo menos um driver. Você pode:

- criar um plugin pela interface
- importar um arquivo JSON de plugin
- carregar plugins já salvos no workspace

## 4. Crie ou Importe uma Planta

Você pode:

- criar uma nova planta na interface
- importar um arquivo JSON de planta com preview antes do cadastro

Cada planta precisa de:

- nome
- tempo de amostragem
- variáveis
- uma instância de driver
- controladores opcionais

## 5. Conecte a Planta

Quando a planta é conectada, o Senamby:

- resolve o driver e os controladores ativos
- prepara ou reutiliza o ambiente Python
- sobe a runtime
- inicia o loop `read -> control -> write -> publish`

## 6. Fechar vs Remover

- **Fechar planta**: descarrega da sessão atual e encerra a runtime, mas preserva o arquivo salvo
- **Remover planta**: descarrega da sessão e apaga o registry salvo no workspace

## 7. Regra de Reabertura

Plantas não são recarregadas automaticamente no startup. Uma planta fechada só volta quando for importada ou aberta novamente.
