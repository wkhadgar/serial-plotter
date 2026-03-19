# Senamby Desktop - Arquitetura Atual

## 1. Objetivo

Este documento descreve a arquitetura real implementada hoje no desktop:

- frontend Svelte;
- shell Tauri;
- backend Rust;
- runtime Python para driver e controladores da planta.

O foco e deixar claro:

- onde cada responsabilidade vive;
- como dados entram e saem da runtime;
- como a aplicacao se organiza para manter performance, seguranca e escalabilidade.

## 2. Principio arquitetural central

Regra do sistema:

- frontend = UX, composicao visual, cache local e interacao;
- backend Rust = dominio, validacao, persistencia, reconciliacao e supervisao;
- runtime Python = ciclo fino de execucao dos plugins.

O frontend nao participa do hot loop de amostragem.

## 3. Mapa de camadas

```text
UI Svelte
  -> services TS (invoke/listen)
    -> commands Tauri
      -> services de dominio em Rust
        -> stores + workspace + runtime manager
          -> runner Python + plugins Python
```

## 4. Frontend

Estrutura principal:

```text
src/routes/+page.svelte
|- PlotterWorkspaceModule.svelte
|- AnalyzerModule.svelte
`- PluginsModule.svelte
```

Camadas internas:

- `components/*`: renderizacao e interacao;
- `services/*`: fronteira com Tauri;
- `stores/*`: estado reativo e buffers de plotagem;
- `types/*`: contratos espelhados do dominio.

Pontos importantes:

- `services/plant/plantService.ts` integra commands de planta e eventos `plant://*`;
- `services/plugin/pluginService.ts` integra CRUD e carga de plugins;
- `components/modules/PlotterWorkspaceModule.svelte` concentra a jornada da planta no frontend;
- o frontend nao calcula `dt`, nao executa controle e nao faz polling do driver.

## 5. Backend Rust/Tauri

Entrada oficial:

- `apps/desktop/src-tauri/src/lib.rs`

## 5.1 Commands expostos

### Commands de planta

- `create_plant`
- `update_plant`
- `list_plants`
- `get_plant`
- `remove_plant`
- `connect_plant`
- `disconnect_plant`
- `pause_plant`
- `resume_plant`
- `save_controller_instance_config`
- `remove_controller_instance`
- `save_plant_setpoint`
- `open_plant_file`
- `import_plant_file`

### Commands de plugin

- `create_plugin`
- `get_plugin`
- `update_plugin`
- `delete_plugin`
- `list_plugins`
- `list_plugins_by_type`
- `load_plugins`
- `import_plugin_file`

## 5.2 Services de dominio

Principais services:

- `PlantService`: CRUD de planta, validacoes e persistencia;
- `PluginService`: CRUD de plugin, carga e persistencia;
- `PlantImportService`: parse/import de plantas;
- `PluginImportService`: parse/import de plugins;
- `DriverRuntimeService`: API de dominio para connect, disconnect, pause, resume e hot updates;
- `PlantRuntimeManager`: gerenciamento de processo, handshake, streams e eventos;
- `WorkspaceService`: caminhos, IO de workspace e persistencia fisica.

## 5.3 Estado em memoria

`AppState` agrega:

- `PlantStore`
- `PluginStore`
- `PlantRuntimeManager`

Ownership do estado:

- plugins persistidos: workspace + `PluginStore`;
- plantas abertas/importadas: `PlantStore` + workspace;
- runtime conectada: `PlantRuntimeManager`;
- historico e buffers de grafico: frontend.

Regra importante:

- plugins sao carregados do workspace no startup do backend;
- plantas nao sao auto-carregadas no startup.

## 6. Modelos de dominio principais

## 6.1 Plugin

`PluginRegistry` representa um plugin persistido.

Campos relevantes:

- `id`
- `name`
- `type`: `driver` ou `controller`
- `runtime`: `python` ou `rust_native`
- `entry_class`
- `schema`
- `source_file`
- `source_code`
- `dependencies`

Papel arquitetural de `entry_class`:

- o backend persiste explicitamente a classe principal do plugin;
- o runner recebe isso como `class_name`;
- a runtime nao depende de heuristica baseada no nome visual do plugin.

Papel arquitetural de `schema`:

- define os campos configuraveis apresentados para a instancia do plugin;
- para drivers, os valores escolhidos acabam em `PlantDriver.config`;
- para controladores, os valores da instancia acabam em `PlantController.params`;
- no `connect_plant`, esses dados viram `bootstrap.driver.config` e `bootstrap.controllers[*].params`.

## 6.2 Planta

`Plant` representa a definicao persistida da planta.

Campos relevantes:

- `id`
- `name`
- `sample_time_ms`
- `variables`
- `driver`
- `controllers`
- `connected`
- `paused`
- `stats`

Subestruturas importantes:

- `PlantVariable`: sensor ou atuador, com unidade, limites e `setpoint`;
- `PlantDriver`: plugin vinculado e `config` da instancia;
- `PlantController`: instancia de controlador vinculada a variaveis e parametros.

## 7. Workspace e persistencia

Raiz de workspace:

- `Documents/Senamby/workspace`

Estrutura atual:

```text
workspace/
|- drivers/<plugin_name>/
|  |- registry.json
|  `- main.py
|- controllers/<plugin_name>/
|  |- registry.json
|  `- main.py
|- plants/<plant_name>/
|  `- registry.json
|- envs/<env_hash>/
|  |- .venv/
|  |- metadata.json
|  `- requirements.lock.txt
`- runtimes/<runtime_id>/
   |- runtime.json
   |- plant.json
   |- bootstrap.json
   |- runner.py
   |- logs/
   `- ipc/
```

Persistente:

- registries de plugin;
- registries de planta;
- ambientes Python reutilizaveis.

Efemero:

- uma pasta por runtime conectada.

## 8. Runtime da planta

A runtime da planta hoje executa no mesmo processo:

- 1 driver Python;
- 0..N controladores Python ativos.

Motivacao:

- reduzir latencia entre leitura, controle e escrita;
- evitar round-trip por amostra com frontend;
- manter sequencia deterministica;
- reduzir acoplamento de UX com timing operacional.

## 9. Ownership por camada

### Frontend

Responsavel por:

- fluxo de usuario;
- formularios de planta, driver e controlador;
- visualizacao de runtime;
- reacao a eventos do backend.

### Backend Rust

Responsavel por:

- validar requests;
- garantir integridade de dominio;
- persistir no workspace;
- reconciliar plugins referenciados por snapshots de planta;
- preparar `.venv` e runtime scaffold;
- supervisionar processo Python;
- emitir eventos para a UI.

### Runtime Python

Responsavel por:

- carregar classes Python do driver e dos controladores;
- instanciar componentes com contexto pronto;
- executar `read -> control -> write -> publish`;
- aplicar hot updates de setpoints e controladores;
- emitir telemetria e diagnostico operacional.

## 10. Protocolo entre Rust e runner

Canal:

- JSON Lines por `stdin/stdout`.

Comandos suportados hoje:

- `init`
- `start`
- `pause`
- `resume`
- `update_setpoints`
- `update_controllers`
- `shutdown`
- `stop`
- `write_outputs` reservado

Eventos emitidos pelo runner:

- `ready`
- `connected`
- `telemetry`
- `cycle_overrun`
- `warning`
- `error`
- `stopped`

Eventos emitidos pelo backend para a UI:

- `plant://telemetry`
- `plant://status`
- `plant://error`

## 11. Bootstrap da runtime

O bootstrap entregue ao runner contem quatro blocos:

- `driver`
- `controllers`
- `plant`
- `runtime`

### `driver`

Contem:

- identidade do plugin;
- `plugin_dir`;
- `source_file`;
- `class_name`;
- `config` da instancia.

### `controllers`

Contem apenas os controladores ativos.

Cada item leva:

- identidade da instancia;
- identidade do plugin;
- `plugin_dir`;
- `source_file`;
- `class_name`;
- `name`;
- `controller_type`;
- `input_variable_ids`;
- `output_variable_ids`;
- `params`.

### `plant`

Contem:

- `variables`;
- `variables_by_id`;
- `sensors`;
- `actuators`;
- `setpoints`.

### `runtime`

Contem:

- `runtime.id`;
- `timing.sample_time_ms`;
- `timing.owner = runtime`;
- `supervision.owner = rust`;
- paths da runtime efemera e da `.venv` reutilizada.

## 12. Fluxo de `connect_plant`

Resumo arquitetural:

1. command recebe `plant_id`;
2. `DriverRuntimeService` resolve planta, driver e controladores;
3. backend reconcilia referencias de plugins por id/nome;
4. backend escolhe ou cria a `.venv` pelo `env_hash`;
5. backend materializa a runtime efemera;
6. backend sobe o runner;
7. aguarda `ready`;
8. envia `init` e `start`;
9. recebe `connected`;
10. marca a planta como conectada.

## 13. Fluxo do ciclo por amostra

`PlantRuntimeEngine.run_cycle()` executa:

1. esperar deadline;
2. `driver.read()`;
3. normalizar sensores e atuadores lidos;
4. `controller.compute(snapshot)` para cada controlador ativo;
5. agregar saidas dos controladores;
6. `driver.write(outputs)` uma unica vez por ciclo, se houver saidas;
7. montar telemetria;
8. emitir `telemetry`;
9. emitir `cycle_overrun` quando necessario.

## 14. Contrato dos plugins Python

### Driver

Recebe `DriverPluginContext` com:

- `config`
- `plant`
- `runtime`

Metodos obrigatorios:

- `connect()`
- `stop()`
- `read()`

Metodo obrigatorio quando houver controladores ativos:

- `write(outputs)`

### Controlador

Recebe `ControllerPluginContext` com:

- `controller`
- `plant`
- `runtime`

Metodo obrigatorio:

- `compute(snapshot)`

Metodos opcionais:

- `connect()`
- `stop()`

## 15. Hot updates em runtime

### Setpoints

- persistem no backend;
- sao refletidos para a runtime via `update_setpoints`;
- entram no proximo ciclo sem restart.

### Controladores

- persistem no backend;
- sao refletidos para a runtime via `update_controllers`;
- o runner faz hot-swap das instancias ativas.

Limitacao arquitetural importante:

- hot update nao reconstrui a `.venv`;
- se um plugin novo nao participou da composicao do ambiente no momento do `connect`, a operacao mais segura continua sendo reconectar a planta.

## 16. Performance e escalabilidade

Decisoes que ajudam a escalar:

- pacing monotonic no runner, nao no frontend;
- ambiente Python deduplicado por `env_hash`;
- driver e controladores no mesmo processo para reduzir latencia;
- comando incremental para setpoints e controladores, evitando restart em alteracoes simples;
- eventos incrementais do backend para a UI, em vez de polling bruto.

## 17. Seguranca e previsibilidade

Regras arquiteturais importantes:

- o backend valida sintaxe Python antes do spawn;
- o backend valida `write(outputs)` antes de permitir controlador ativo;
- o runner revalida esse contrato defensivamente;
- o backend reconcilia snapshots antigos com plugins reais do workspace;
- warnings operacionais nao derrubam o processo automaticamente;
- erros estruturais viram eventos de erro e podem encerrar a runtime.

## 18. Pontos de extensao futuros

A arquitetura atual ja deixa espaco para:

- novos tipos de runtime alem de Python;
- commands adicionais de IO em runtime;
- telemetria mais rica por controlador;
- multiplas estrategias de scheduler no runner;
- modularizacao adicional do `runtime.rs` e do `runner.py` sem mudar o contrato externo.

## 19. Documentos complementares

- [payload.md](/home/higor/Documents/programacao/python/serial-plotter-app/payload.md)
- [connection.md](/home/higor/Documents/programacao/python/serial-plotter-app/connection.md)
