# Fluxo de Conexao, Runtime e Sincronizacao da Planta

## Objetivo

Este documento descreve o fluxo real da aplicacao desde a acao da UI ate a plotagem:

- carga de plugins no backend;
- abertura ou criacao de planta;
- resolucao de driver e controladores;
- bootstrap da runtime Python;
- ciclo `read -> control -> write -> publish`;
- propagacao de eventos para a UI.

## 1. Visao geral

A runtime da planta so existe quando a planta esta ligada.

Fluxo de alto nivel:

1. frontend chama um command Tauri;
2. backend Rust resolve estado de dominio;
3. backend prepara ambiente Python e runtime efemera;
4. backend sobe `runner.py`;
5. backend envia `init` e `start`;
6. runner conecta driver e controladores ativos;
7. runner executa o loop de amostragem;
8. runner emite telemetria e eventos;
9. Rust traduz isso em eventos `plant://*` para a UI;
10. frontend atualiza cards, estados e graficos.

## 2. Responsabilidades por camada

### Frontend

Responsavel por:

- iniciar e encerrar a planta;
- editar setpoints e controladores;
- reagir a eventos `plant://telemetry`, `plant://status` e `plant://error`;
- manter buffers e graficos.

Nao e responsavel por:

- pacing do sample time;
- sequencia `read -> control -> write`;
- validacao de contrato de plugin.

### Backend Rust

Responsavel por:

- validacao de dominio;
- persistencia em workspace;
- stores em memoria;
- resolucao de plugins por id e por nome;
- reconciliacao de snapshots persistidos;
- montagem de bootstrap;
- preparo e reuse da `.venv`;
- supervisao do processo Python;
- traducao de eventos para a UI.

### Runner Python

Responsavel por:

- carregar classes Python explicitas;
- instanciar driver e controladores ativos;
- manter o relogio monotonic;
- executar `read -> control -> write -> publish`;
- aplicar hot updates de setpoints e controladores;
- emitir warnings, erros e telemetria por ciclo.

## 3. O que acontece no startup da aplicacao

Quando o desktop sobe:

1. `AppState::new()` cria `PlantStore`, `PluginStore` e `PlantRuntimeManager`;
2. o backend chama `PluginService::load_all(&plugin_store)`;
3. plugins persistidos em `workspace/drivers/*` e `workspace/controllers/*` sao reidratados no store.

Importante:

- plantas nao sao carregadas automaticamente no startup;
- a existencia de um `registry.json` de planta no workspace nao cria uma runtime nem abre a planta sozinho.

## 4. Workspace envolvido

## 4.1 Persistente

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
`- envs/<env_hash>/
   |- .venv/
   |- metadata.json
   `- requirements.lock.txt
```

## 4.2 Efemero por runtime conectada

```text
workspace/runtimes/<runtime_id>/
|- runtime.json
|- plant.json
|- bootstrap.json
|- runner.py
|- logs/
`- ipc/
```

A pasta em `runtimes/<runtime_id>` existe apenas enquanto a planta esta conectada.

## 5. Fluxo de abrir ou importar uma planta

## 5.1 `open_plant_file`

Usado para parsear um arquivo antes da importacao definitiva.

Efeito:

- o backend interpreta o JSON;
- devolve uma estrutura pronta para preview no frontend;
- nao persiste nada ainda.

## 5.2 `import_plant_file`

Usado para importar a planta de fato.

Efeito:

- cria ou reidrata a planta no backend;
- persiste em `workspace/plants/<plant_name>/registry.json`;
- devolve `PlantResponse`.

Durante essa fase:

- o backend tenta restaurar o driver e os controladores vinculados pela informacao persistida no arquivo;
- se um plugin de controlador ou driver nao estiver carregado, a planta ainda pode abrir, mas runtime e acoes dependentes desse plugin podem ser bloqueadas depois;
- a reconciliacao real do que vai para a runtime ocorre no `connect_plant`.

## 6. Fluxo de `connect_plant`

### Etapa A - chamada da UI

A UI chama:

```ts
invoke('connect_plant', { id: plantId })
```

### Etapa B - resolucao da planta

O backend busca a planta no `PlantStore` e bloqueia se:

- a planta ja estiver conectada;
- o driver persistido nao puder ser resolvido;
- houver controlador ativo sem plugin correspondente carregado;
- um controlador ativo exigir runtime Python e o plugin nao for Python.

### Etapa C - reconciliacao de driver e controladores

O backend resolve plugins assim:

1. tenta por `plugin_id`;
2. se nao achar, tenta por `plugin_name`;
3. se ainda nao achar, faz `PluginService::load_all()` uma vez e tenta de novo.

Se houver divergencia entre snapshot persistido e plugin real do workspace, o backend atualiza a planta em memoria e persiste novamente o `registry.json`.

Isso corrige casos em que:

- o plugin foi reimportado e ganhou novo `id`;
- o nome persistido mudou de caixa;
- o `source_file` ou `runtime` do driver mudou.

### Etapa D - definicao do conjunto de plugins da runtime

O backend monta dois conjuntos diferentes:

- `active_controllers`: apenas controladores ativos, que vao para `bootstrap.controllers`;
- `runtime_plugins`: driver + todos os plugins de controlador resolvidos que ja fazem parte da planta naquele momento.

Motivo:

- o loop da runtime so precisa instanciar controladores ativos;
- a composicao do ambiente Python precisa conhecer o conjunto de plugins que podem ser usados pela planta sem depender de heuristica do frontend.

## 7. Como a `.venv` da runtime e escolhida

O backend:

1. coleta dependencias de todos os `runtime_plugins`;
2. deduplica por pacote;
3. monta o `requirements.lock.txt` logico;
4. gera um `env_hash`;
5. reusa ou cria `workspace/envs/<env_hash>`.

Regra de dependencia:

- se `version` vier vazia, o pacote vai como apenas `name`;
- se `version` vier preenchida, o pacote vai como `name==version`.

Consequencia pratica:

- hot update de parametros e de controladores ativos nao recria `.venv`;
- para introduzir em runtime um plugin novo que nao participou da composicao do ambiente no `connect`, a forma mais segura continua sendo reconectar a planta.

## 8. Validacoes antes do spawn

Antes de subir o processo Python, o backend valida:

- existencia de `registry.json` do driver e dos controladores ativos;
- existencia de `main.py` do driver e dos controladores ativos;
- sintaxe Python dos arquivos;
- contrato do driver quando houver controlador ativo, especialmente `write(outputs)`.

Isso evita falhas tarde demais no ciclo.

## 9. Materializacao da runtime efemera

Para cada `connect_plant`, o backend cria `workspace/runtimes/<runtime_id>` e grava:

- `runtime.json`: resumo da configuracao de runtime;
- `plant.json`: snapshot da planta entregue ao runner;
- `bootstrap.json`: bootstrap completo usado no `init`;
- `runner.py`: copia do runner embutido no backend.

## 10. Spawn do runner

O backend sobe o processo com:

```text
<venv_python_path> -u <runner.py> --runtime-dir <runtime_dir> --bootstrap <bootstrap.json>
```

Streams:

- `stdin`: protocolo Rust -> runner;
- `stdout`: protocolo runner -> Rust;
- `stderr`: logs tecnicos e stack traces.

## 11. Handshake de startup

Sequencia real:

1. o processo Python abre e valida `bootstrap.json` do disco;
2. runner emite `ready`;
3. Rust aguarda `ready`;
4. Rust envia `init` com o bootstrap oficial;
5. Rust envia `start`;
6. runner carrega a classe do driver;
7. runner instancia o driver com `DriverPluginContext`;
8. runner valida `write(outputs)` se houver controladores ativos;
9. runner chama `driver.connect()`;
10. runner carrega controladores ativos;
11. runner instancia cada controlador com `ControllerPluginContext`;
12. runner chama `connect()` opcional em cada controlador;
13. runner emite `connected`;
14. Rust marca a planta como `connected = true`, `paused = false`.

Timeouts atuais:

- startup: `12000 ms`;
- shutdown: `4000 ms`.

## 12. Fluxo do ciclo de amostragem

Por ciclo, o runner faz:

1. espera a deadline monotonic;
2. incrementa `cycle_id`;
3. chama `driver.read()`;
4. normaliza sensores e atuadores lidos;
5. para cada controlador ativo:
   - constroi o snapshot;
   - chama `compute(snapshot)`;
   - normaliza as saidas;
   - agrega em `controller_outputs`;
6. se houver saidas agregadas, chama `driver.write(outputs)` uma unica vez;
7. monta a telemetria final;
8. emite `telemetry`;
9. se houver overrun, emite `cycle_overrun`;
10. calcula a proxima deadline.

## 13. Como dados da planta entram no driver e no controlador

## 13.1 Driver

O driver recebe:

- `context.config`: parametros configuraveis da instancia;
- `context.plant`: planta completa com variaveis, grupos e setpoints;
- `context.runtime`: sample time, timeout e paths da runtime.

Isso permite ao driver saber:

- quais sensores e atuadores existem;
- qual `variable_id` usar para publicar cada leitura;
- quais canais ou portas deve abrir;
- qual sample time a planta esta usando.

## 13.2 Controlador

O controlador recebe:

- `context.controller`: metadados da instancia e parametros da planta;
- `context.plant`: variaveis e setpoints globais;
- `context.runtime`: informacoes de timing e paths.

E a cada ciclo ele recebe ainda um `snapshot` com:

- leituras atuais de sensores;
- leituras atuais de atuadores;
- setpoints atuais;
- `dt_s` efetivo;
- `variables_by_id` para consulta de limites, unidades e links.

## 14. Fluxo de setpoint em runtime

Quando a UI altera um setpoint:

1. chama `save_plant_setpoint`;
2. o backend persiste no `registry.json` da planta;
3. atualiza o `PlantStore`;
4. se a planta estiver conectada, envia `update_setpoints` para o runner;
5. o runner atualiza `plant.setpoints` em memoria;
6. o novo valor entra automaticamente no snapshot do proximo ciclo.

## 15. Fluxo de controlador em runtime

## 15.1 Salvar ou editar controlador

Quando a UI salva um controlador:

1. chama `save_controller_instance_config`;
2. o backend faz upsert da instancia na planta;
3. persiste no `registry.json`;
4. se a planta estiver conectada, resolve novamente os controladores ativos;
5. envia `update_controllers` para o runner;
6. o runner faz hot-swap da lista carregada.

Hoje e permitido em runtime:

- alterar `active`;
- alterar `input_variable_ids`;
- alterar `output_variable_ids`;
- alterar `params`.

## 15.2 Remover controlador

Quando a UI remove um controlador:

1. chama `remove_controller_instance`;
2. o backend remove do store e do `registry.json`;
3. se conectado, recalcula a lista de ativos;
4. envia `update_controllers` com a nova lista.

## 15.3 Regras importantes

- controladores nascem desligados por padrao;
- para existir controlador ativo, o driver precisa suportar `write(outputs)`;
- o backend tenta bloquear isso antes de chegar ao runner;
- o runner continua com validacao defensiva do mesmo contrato.

## 16. Fluxo de pause e resume

## 16.1 `pause_plant`

1. UI chama `pause_plant`;
2. backend valida que a planta esta conectada;
3. envia `pause` ao runner;
4. marca `paused = true` no `PlantStore`.

Efeito no runner:

- interrompe o ciclo inteiro;
- nao faz `read`, `compute` nem `write`;
- mantem a runtime viva.

## 16.2 `resume_plant`

1. UI chama `resume_plant`;
2. backend valida que a planta esta conectada e pausada;
3. envia `resume` ao runner;
4. marca `paused = false` no `PlantStore`.

Efeito no runner:

- retoma o ciclo fino;
- ajusta o relogio para descontar o tempo pausado do `uptime_s`.

## 17. Fluxo de disconnect

Quando a UI chama `disconnect_plant`:

1. o backend envia `shutdown` ao runner;
2. aguarda `stopped` ou timeout de encerramento;
3. mata o processo se necessario;
4. remove a pasta `workspace/runtimes/<runtime_id>`;
5. marca `connected = false` e `paused = false` no `PlantStore`.

## 18. Como a telemetria vira grafico

1. runner emite `telemetry` por `stdout`;
2. `PlantRuntimeManager` le a mensagem;
3. backend traduz para evento Tauri `plant://telemetry`;
4. `plantService.ts` recebe o evento;
5. stores de frontend atualizam buffers por variavel;
6. `PlotlyChart.svelte` e componentes derivados reagem ao estado;
7. a UI plota sensores, atuadores, setpoints e derivados.

## 19. Falhas e degradacao controlada

### Falhas tratadas como `warning`

Exemplos:

- erro em `driver.read()` num ciclo;
- erro em `controller.compute()` num ciclo;
- erro em `driver.write(outputs)` num ciclo.

Comportamento:

- o processo continua vivo;
- o ciclo seguinte ainda pode rodar.

### Falhas tratadas como `error`

Exemplos:

- bootstrap invalido;
- comando JSON invalido;
- troca de controladores rejeitada;
- falha estrutural do runner.

Comportamento:

- o backend propaga `plant://error`;
- dependendo do momento da falha, a runtime pode continuar ou ser derrubada pelo supervisor.

## 20. Invariantes que a arquitetura assume

- o frontend nunca decide o pacing do sample time;
- Rust e a fonte de verdade de dominio e lifecycle;
- Python e a fonte de verdade do hot loop por amostra;
- o runner nunca infere classe Python pelo nome visual do plugin;
- o binding entre planta e plugin e reconciliado no backend por id e por nome;
- a UI so plota o que chega da telemetria oficial da runtime.
