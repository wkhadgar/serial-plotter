# Commit Suggestion

`feat(runtime): implement explicit rust-python driver runtime pipeline with ephemeral runtimes and reusable isolated envs`

RULE: nao faca nenhum commit

# Task

## Objetivo

Definir um plano tecnico explicito para iniciar e operar instancias de `drivers` Python associadas a plantas, incluindo:

- fluxo de disparo pela interface;
- resolucao da planta e do driver;
- bootstrap do processo Python;
- protocolo de comunicacao Rust <-> Python;
- maquina de estados explicita para a runtime da planta;
- ciclo de amostragem;
- garantia de tempo de amostragem sem acumulacao de erro;
- deteccao de atraso de leitura e de overrun de ciclo;
- calculo do `dt` efetivo para alimentar o espaco reservado da UI;
- telemetria;
- entrega performatica para a UI;
- plotagem em tempo real;
- arquitetura de `runtimes/` efemeros;
- arquitetura de `.venv` isolados e reutilizaveis.

Por enquanto, esta task deve cobrir apenas `drivers`.

Mesmo assim, a arquitetura deve nascer preparada para o ciclo completo da planta:

`ler -> calcular controladores -> escrever`

Nesta fase, apenas a etapa de `ler` deve ser implementada de ponta a ponta, mas o backend ja deve ficar preparado para escalar para as proximas etapas sem quebrar o contrato do runtime.

## Decisao arquitetural principal

A conexao com Python deve ser feita por **processo filho isolado**, e nao por thread Rust embutindo interpretador Python.

### Motivo da escolha

Essa abordagem e a mais segura e escalavel para o estado atual do projeto, porque:

- isola falhas do driver Python;
- facilita restart do driver sem derrubar o backend Tauri;
- evita acoplamento forte com embedding de Python no processo principal;
- simplifica supervisao de runtime;
- permite protocolo de comunicacao explicito e testavel;
- prepara o sistema para multiplas plantas/instancias no futuro.

## Decisao de arquitetura para runtimes e ambientes Python

O backend deve separar claramente:

- artefatos persistentes do plugin;
- ambientes Python reutilizaveis;
- runtimes efemeros de execucao da planta.

### Estrutura recomendada

```text
workspace/
  plants/
    <plant_name>
        registry.json
  plugins/
    drivers/<plugin_name>/
      registry.json
      main.py
  envs/
    <env_hash>/
      .venv/
      requirements.lock.txt
      metadata.json
  runtimes/
    <runtime_id>/
      runtime.json
      plant.json
      driver/
        bootstrap.json
      logs/
      ipc/
```

### Regra principal

Quando uma planta for iniciada:

- o backend deve criar `workspace/runtimes/<runtime_id>/`;
- esse diretório representa apenas a execucao atual daquela planta;
- o processo Python do driver deve ficar vinculado a esse `runtime_id`;
- ao desligar a planta, esse `runtime_dir` deve ser apagado.

### Regra para ambientes Python

Os ambientes Python nao devem ser recriados toda vez que a planta ligar.

Em vez disso:

- deve existir um cache de ambientes em `workspace/envs/`;
- cada ambiente deve ser identificado por um `env_hash`;
- o `env_hash` deve ser derivado de runtime + dependencias + metadados relevantes;
- o runtime efemero referencia esse `.venv`, mas nao deve copiar o ambiente para dentro de `runtimes/`.

### Motivo dessa separacao

Essa abordagem evita:

- recriacao cara de `.venv` a cada inicializacao;
- reinstalacao repetitiva de dependencias;
- startup lento;
- consumo excessivo de disco e IO;
- lixo demais no workspace.

Ela garante:

- isolamento por dependencias;
- reuso performatico de ambiente;
- runtime efemero por planta;
- limpeza simples ao desligar.

## Estrategia de conexao Rust <-> Python

O backend Rust deve subir um processo Python por planta ativa.

### Modelo recomendado

Rust deve executar algo equivalente a:

```text
<workspace/envs/<env_hash>/.venv/bin/python> -u <runner.py> --runtime-dir <workspace/runtimes/<runtime_id>> --driver-dir <workspace/plugins/drivers/<plugin_id>> --runtime-config <json-base64-ou-stdin>
```

### Regras da inicializacao

- usar `-u` para evitar buffering agressivo;
- `stdin` do processo Python deve permanecer aberto para comandos;
- `stdout` deve ser reservado para mensagens estruturadas do protocolo;
- `stderr` deve ser tratado como canal de logs e diagnostico;
- cada processo deve ficar associado a uma unica planta;
- o processo deve usar sempre o executavel Python do `.venv` resolvido;
- o `runtime_dir` deve existir antes do spawn do processo;
- o processo nao deve alterar diretamente os arquivos persistentes do plugin.

## Runner Python dedicado

Deve existir um arquivo Python runner dedicado, separado do `main.py` do driver.

### Responsabilidade do runner

O runner deve:

- carregar o `main.py` do driver a partir do workspace;
- localizar a classe concreta do driver;
- validar que ela herda da classe pai de drivers;
- instanciar o driver com a configuracao da planta;
- executar `connect()`;
- iniciar o loop principal;
- receber comandos do Rust;
- devolver telemetria, status e erros por protocolo estruturado.

### Importante

O `main.py` do driver nao deve ser o processo principal diretamente.

O processo principal deve ser o runner, que encapsula:

- carregamento dinamico;
- controle de ciclo de vida;
- tratamento de erro;
- serializacao de mensagens.

## Protocolo de comunicacao entre Rust e Python

A comunicacao entre Rust e Python deve ser feita por **JSON Lines** via `stdin/stdout`.

### Motivo

JSON Lines e adequado aqui porque:

- e simples de debugar;
- funciona bem com stream;
- evita framing manual complexo;
- permite mensagens estruturadas;
- e suficiente para o volume inicial de telemetria.

## Formato das mensagens Rust -> Python

O Rust deve enviar comandos estruturados, por exemplo:

```json
{"type":"init","payload":{"runtime_id":"rt_1","plant_id":"plant_1","sample_time_ms":100,"driver_config":{...},"variables":[...]}}
{"type":"start"}
{"type":"write_outputs","payload":{"actuators":{"var_3":42.0}}}
{"type":"pause"}
{"type":"resume"}
{"type":"stop"}
{"type":"shutdown"}
```

### Regras

- toda mensagem deve ter `type`;
- comandos com dados devem ter `payload`;
- mensagens devem ser pequenas e incrementais;
- nao reenviar a configuracao completa da planta em toda iteracao.

## Formato das mensagens Python -> Rust

O processo Python deve responder com mensagens estruturadas, por exemplo:

```json
{"type":"ready","payload":{"runtime_id":"rt_1","driver":"modbus_driver","plant_id":"plant_1"}}
{"type":"connected"}
{"type":"telemetry","payload":{"timestamp":1710000000.123,"cycle_id":42,"configured_sample_time_ms":100,"effective_dt_ms":100.1,"cycle_duration_ms":98.4,"read_duration_ms":27.3,"cycle_late":false,"phase":"publish_telemetry","sensors":{"var_0":12.3},"actuators":{"var_2":40.0},"setpoints":{"var_0":15.0}}}
{"type":"cycle_overrun","payload":{"cycle_id":43,"configured_sample_time_ms":100,"cycle_duration_ms":132.6,"late_by_ms":32.6,"phase":"read_inputs"}}
{"type":"warning","payload":{"message":"read timeout, retrying"}}
{"type":"error","payload":{"message":"serial port not found"}}
{"type":"stopped"}
```

### Regras

- `stdout` deve conter apenas mensagens do protocolo;
- logs humanos nao devem ser escritos em `stdout`;
- logs humanos devem ir para `stderr`;
- `telemetry` deve ser padronizada e incremental;
- `telemetry` deve incluir metrica suficiente para o Rust detectar atraso e alimentar o `dt` real na UI;
- mensagens de erro devem ser parseaveis pelo Rust.

## Fluxo ponta a ponta

### 1. Clique em `LIGAR` na interface

Fluxo atual que deve ser preservado:

- `PlotterToolbar.svelte`
- `PlotterModule.svelte`
- `plantService.ts`
- command `connect_plant`

### 2. Entrada oficial no backend

Ao receber `connect_plant(plant_id)`:

- command deve ser fino;
- command delega a um caso de uso de runtime;
- o backend valida se a planta existe;
- valida se ela ja nao esta em execucao;
- valida se ha driver associado.

### 3. Resolver a planta

O backend deve:

- obter a planta em memoria;
- confirmar `plant.driver.plugin_id`;
- obter nome da planta, `sample_time_ms`, stats e configuracao atual;
- garantir consistencia com o registry persistido, se necessario.

### 4. Resolver o driver associado

O backend deve:

- localizar o plugin driver correspondente;
- localizar o diretório persistente do driver no workspace;
- ler `registry.json` do driver;
- ler o `main.py` salvo no disco;
- validar que ambos existem;
- validar que o runtime do driver e Python.

### 5. Resolver o ambiente Python isolado

Antes de subir o processo, o backend deve:

- ler o registry do driver;
- extrair dependencias declaradas;
- gerar um `env_hash` deterministico;
- localizar `workspace/envs/<env_hash>/`;
- se o ambiente nao existir, criar `.venv` e instalar dependencias;
- se o ambiente ja existir, reutilizar.

### Regras para o `env_hash`

O hash deve mudar quando mudar:

- runtime;
- dependencias;
- formato do ambiente;
- opcionalmente versao do plugin, se fizer parte da politica.

O hash nao deve mudar apenas por:

- nome da planta;
- `runtime_id`;
- ligar/desligar planta.

### 6. Criar o runtime efemero da planta

Antes do spawn do processo Python, o backend deve:

- gerar um `runtime_id`;
- criar `workspace/runtimes/<runtime_id>/`;
- materializar nesse diretório:
  - `runtime.json`
  - `plant.json`
  - `driver/bootstrap.json`
  - pasta `logs/`
  - pasta `ipc/`, se necessario

Esse diretório deve existir apenas enquanto a planta estiver em execucao.

### 7. Montar o runtime payload

Antes de subir o processo Python, o backend deve montar um payload unico contendo:

- `runtime_id`
- `plant_id`
- nome da planta
- `sample_time_ms`
- configuracoes do driver cadastradas na planta
- lista de variaveis da planta
- mapeamento de sensores
- mapeamento de atuadores
- setpoints atuais
- caminho do driver no workspace
- caminho do `runtime_dir`
- caminho do `.venv`
- caminho do runner Python

Esse payload deve ser a verdade unica de bootstrap da instancia.

## Runtime manager no backend

Deve existir uma camada dedicada, por exemplo:

- `PlantRuntimeManager`
- `DriverRuntimeService`
- `PythonRuntimeSupervisor`

### Essa camada deve manter

- mapa `plant_id -> runtime_handle`;
- opcionalmente `runtime_id -> runtime_handle`;
- `Child` process handle;
- canal interno para comandos;
- thread de leitura de `stdout`;
- thread opcional de leitura de `stderr`;
- estado da runtime;
- timestamp da ultima telemetria;
- mecanismo de stop seguro.

## Estrutura minima de um runtime handle

Cada planta ligada deve ter algo como:

- `plant_id`
- `runtime_id`
- `child_process`
- `env_hash`
- `venv_python_path`
- `runtime_dir`
- `stdin_writer`
- `stdout_reader_task`
- `stderr_reader_task`
- `lifecycle_state`
- `cycle_phase`
- `configured_sample_time_ms`
- `last_effective_dt_ms`
- `last_cycle_duration_ms`
- `last_read_duration_ms`
- `last_cycle_late`
- `late_cycle_count`
- `last_telemetry_at`

## Maquina de estados do runtime da planta

O runtime da planta deve seguir uma maquina de estados explicita em dois niveis:

- estado de ciclo de vida do runtime;
- estado interno do ciclo de amostragem.

### Estados de ciclo de vida

Estados minimos esperados:

- `created`
- `bootstrapping`
- `ready`
- `connecting`
- `running`
- `stopping`
- `stopped`
- `faulted`

### Estados internos do ciclo

O ciclo da planta deve obedecer ao contrato:

`read_inputs -> compute_controllers -> write_outputs`

Com etapas auxiliares:

- `cycle_started`
- `read_inputs`
- `compute_controllers`
- `write_outputs`
- `publish_telemetry`
- `sleep_until_deadline`

### Regra de escopo atual

Mesmo que apenas `read_inputs` esteja sendo implementado agora, a maquina de estados ja deve prever explicitamente:

- a etapa futura de calculo de controladores;
- a etapa futura de escrita na planta;
- as transicoes entre essas etapas;
- a possibilidade de pular etapas futuras quando o ciclo estourar o prazo.

### Regra de observabilidade

O Rust deve saber, a qualquer momento:

- em que `lifecycle_state` a planta esta;
- em que `cycle_phase` o ciclo atual esta;
- qual foi o `dt` efetivo do ultimo ciclo;
- se a ultima leitura atrasou;
- se o ciclo ultrapassou o tempo configurado;
- quantos overruns ja aconteceram.

Essas informacoes devem ficar disponiveis para:

- logs;
- diagnostico;
- eventos para a UI;
- evolucao futura para controle e escrita.

## Startup handshake obrigatorio

Ao subir o processo Python:

1. Rust resolve ou cria o `.venv` isolado.
2. Rust cria `workspace/runtimes/<runtime_id>/`.
3. Rust grava o bootstrap do runtime.
4. Rust cria o processo Python usando o executavel do `.venv`.
5. Rust envia `init`.
6. Python responde `ready`.
7. Python tenta `connect()`.
8. Python responde `connected` ou `error`.
9. Apenas depois disso a planta passa a estado `connected = true`.

### Regra importante

Nao marcar a planta como conectada antes do handshake completar com sucesso.

## Loop de amostragem e garantia temporal

O loop principal deve ser controlado pelo processo Python.

### Motivo

Isso evita:

- jitter extra por ida e volta a cada sample;
- overhead de comando Rust para cada leitura;
- dependencias desnecessarias do Rust no tempo de amostragem fino.

### Contrato do ciclo

O contrato do ciclo deve ser sempre pensado como:

1. ler da planta;
2. calcular controladores;
3. escrever na planta;
4. publicar telemetria;
5. dormir apenas o restante do tempo ate o proximo ciclo.

### Regra de escopo atual

Nesta fase, apenas a etapa `ler da planta` deve rodar de fato.

Mesmo assim:

- a estrutura do loop ja deve prever `calcular controladores`;
- a estrutura do loop ja deve prever `escrever na planta`;
- o runtime deve ser modelado para crescer sem quebrar a temporizacao.

### Regra de temporizacao

O tempo de amostragem configurado deve ser tratado como deadline fixa por ciclo.

Nao pode haver acumulacao de erro por soma de sleeps ou por usar `sleep(sample_time_ms)` de forma cega no fim de cada iteracao.

O ciclo deve funcionar com relogio monotonic e deadline explicita.

### Algoritmo esperado

O agendamento deve seguir a ideia:

1. registrar `cycle_started_at`;
2. executar a etapa de leitura;
3. medir quanto tempo a leitura levou;
4. medir quanto tempo o ciclo inteiro levou;
5. calcular `remaining_time = next_deadline - now`;
6. se `remaining_time > 0`, dormir apenas esse restante;
7. se `remaining_time <= 0`, registrar overrun e iniciar o proximo ciclo sem dormir.

### Regra contra deriva acumulada

O proximo ciclo nao deve ser calculado a partir de `agora + sample_time`.

Ele deve ser calculado a partir da deadline planejada do ciclo anterior, para evitar deriva acumulada.

Em outras palavras:

- manter `next_deadline`;
- ao finalizar um ciclo, avancar `next_deadline += sample_time`;
- se o tempo ja tiver passado dessa deadline, marcar atraso;
- nao tentar compensar com sleeps extras;
- nao executar varios ciclos em burst para "recuperar" o tempo perdido.

### Regra quando houver estouro de tempo

Se o ciclo ultrapassar o `sample_time_ms`:

- o runtime deve marcar o ciclo como atrasado;
- o Rust deve ficar sabendo desse atraso;
- o `dt` efetivo deve refletir isso;
- o proximo ciclo deve comecar imediatamente, sem esperar um novo sleep artificial.

### Regra futura para controle e escrita

Quando `compute_controllers` e `write_outputs` entrarem em escopo, o contrato deve ser:

- se ainda houver tempo dentro da deadline, executar as proximas etapas;
- se a deadline estourar antes de concluir o ciclo, interromper o restante do ciclo;
- iniciar o proximo ciclo sem acumular erro;
- no futuro, um valor de controle previamente calculado podera ser reutilizado, mas isso ainda nao faz parte desta implementacao.

### `dt` real

O Python deve calcular e enviar:

- `effective_dt_ms` do ciclo;
- `cycle_duration_ms`;
- `read_duration_ms`;
- indicacao de `cycle_late`;
- timestamp monotonic ou timestamp absoluto;
- uptime acumulado da runtime.

O Rust deve usar essas informacoes para:

- preencher o espaco reservado de `dt` real na UI;
- detectar leitura atrasada;
- registrar overruns;
- preparar a evolucao para controle e escrita.

## Caminho performatico de telemetria

O caminho recomendado e:

- Python envia `telemetry` em JSON Lines;
- thread Rust dedicada consome o `stdout`;
- Rust converte para estrutura interna de telemetria;
- Rust atualiza runtime state minimo;
- Rust publica evento incremental para o frontend;
- frontend injeta apenas o novo sample no store.

### Regras de performance

- nao recalcular a planta inteira a cada sample;
- nao reenviar schema, driver config ou metadata em toda telemetria;
- nao usar polling da UI para buscar novos dados;
- nao clonar estruturas grandes sem necessidade;
- usar payload pequeno e orientado a append.

## Entrega para a UI

O backend deve emitir eventos especificos para a planta, por exemplo:

- `plant://telemetry`
- `plant://status`
- `plant://error`

Cada evento deve conter:

- `plant_id`
- `runtime_id`
- tipo do evento
- payload minimo necessario

Eventos de status e telemetria devem carregar tambem, quando aplicavel:

- `lifecycle_state`
- `cycle_phase`
- `configured_sample_time_ms`
- `effective_dt_ms`
- `cycle_late`

## Integracao com plotagem

No frontend, ao receber `telemetry`:

- localizar a planta alvo;
- anexar novo ponto ao buffer;
- atualizar `stats.dt` com o `effective_dt_ms` realmente praticado;
- atualizar `stats.uptime`;
- atualizar valores visiveis;
- acionar plotagem incremental.

Quando houver `cycle_overrun` ou ciclo atrasado:

- a UI deve conseguir refletir isso no estado da planta;
- sem interromper a plotagem;
- sem precisar de polling adicional.

### Politica de buffer

Para manter performance:

- definir limite maximo de samples em memoria por planta;
- usar janela deslizante quando necessario;
- evitar crescimento infinito de arrays;
- manter exportacao e historico alinhados com a estrategia escolhida.

## Escrita de atuadores

Mesmo que o foco inicial seja leitura, o caminho deve nascer preparado para escrita.

Fluxo recomendado:

- frontend ou controlador altera saida;
- backend enfileira comando da planta;
- runtime Rust envia `write_outputs` ao Python;
- Python chama `send(...)` no driver;
- o proximo pacote de telemetria pode refletir a saida aplicada.

## Tratamento de erro e recuperacao

O sistema deve prever explicitamente:

- `main.py` inexistente;
- `registry.json` do driver ausente ou invalido;
- classe do driver nao encontrada;
- classe nao herdando da base esperada;
- falha em `connect()`;
- timeout de leitura;
- driver travado;
- processo Python encerrado inesperadamente;
- telemetria invalida;
- JSON malformado no protocolo;
- falha ao criar `.venv`;
- falha ao instalar dependencias;
- corrida de instalacao simultanea do mesmo `env_hash`.

### Reacao esperada

Ao erro:

- runtime e marcada como falha;
- planta nao fica com estado falso de conectada;
- evento de erro vai para UI;
- handles e canais sao limpos;
- `runtime_dir` efemero deve ser limpo;
- `.venv` nao deve ser removido automaticamente, exceto em estrategia explicita de invalidacao;
- opcionalmente manter ponto de restart futuro preparado.
- atraso de leitura e overrun de ciclo nao devem ficar invisiveis para o backend.

## Stop e encerramento

Ao clicar em `DESLIGAR`:

- frontend chama `disconnect_plant`;
- backend localiza runtime ativa;
- Rust envia `stop` ou `shutdown` ao Python;
- Python chama `driver.stop()`;
- processo encerra com timeout controlado;
- se nao encerrar, Rust mata o processo;
- runtime handle e removido;
- `workspace/runtimes/<runtime_id>/` e apagado;
- o `.venv` em `workspace/envs/<env_hash>/` deve ser preservado;
- UI recebe `status = offline`.

## Ordem recomendada de implementacao

1. criar `PlantRuntimeManager` no backend;
2. criar uma camada de paths/workspace que suporte `plugins/`, `envs/` e `runtimes/`;
3. definir estrategia de `env_hash`;
4. criar runner Python dedicado;
5. definir structs Rust para protocolo JSON Lines;
6. criar bootstrap `connect_plant -> runtime start`;
7. implementar resolucao/criacao de `.venv` por dependencias;
8. materializar `runtime_dir` efemero;
9. implementar handshake `init -> ready -> connected`;
10. implementar leitura de `stdout` em thread dedicada;
11. emitir eventos incrementais para a UI;
12. integrar store da planta e plotagem;
13. implementar `disconnect_plant` com stop seguro;
14. limpar `runtime_dir` ao finalizar e preservar `envs/`;
15. adicionar tratamento de erro, locks de instalacao e timeouts.

## Regras obrigatorias

- implementar apenas `drivers` por enquanto;
- implementar de ponta a ponta apenas a etapa de leitura por enquanto;
- nao acoplar logica de runtime diretamente aos commands;
- nao embutir Python dentro do processo principal do Tauri nesta fase;
- usar processo Python isolado;
- usar protocolo estruturado;
- manter o workspace como fonte oficial do driver;
- usar `runtimes/` como area efemera por execucao da planta;
- usar `envs/` como cache reutilizavel de ambientes Python isolados;
- nao recriar `.venv` inteira a cada start da planta;
- modelar o runtime como maquina de estados explicita;
- garantir o `sample_time_ms` sem deriva acumulada;
- fazer o Rust saber quando a leitura ou o ciclo atrasarem;
- manter o backend preparado para expandir de leitura para `read -> control -> write`;
- manter compatibilidade com a planta e driver ja cadastrados;
- manter build funcional durante e ao final.

## Fora de escopo agora

- runtime completa de controladores;
- criar `.venv` efemera por start de planta;
- cluster de processos;
- RPC complexo;
- WebSocket interno;
- reescrever a UI inteira;
- sistema distribuido;
- orquestracao multi-maquina.

## Criterios de aceitacao

Esta task sera considerada completa quando houver um caminho tecnico claro, especifico e implementavel para:

- clicar em `LIGAR` na UI;
- criar um `runtime_id` e um `runtime_dir` efemero por planta ativa;
- resolver e reutilizar um `.venv` isolado por dependencias;
- subir um processo Python por planta usando o `.venv` correto;
- carregar `registry.json` e `main.py` do driver;
- instanciar a classe concreta do driver com configuracao da planta;
- completar handshake Rust <-> Python;
- ter uma maquina de estados explicita para runtime e ciclo;
- rodar loop de amostragem no `sample_time_ms` sem acumulacao de erro;
- calcular e expor `effective_dt_ms` para a UI;
- detectar e registrar atraso de leitura e overrun de ciclo;
- receber telemetria incremental em Rust;
- encaminhar eventos performaticos para a UI;
- atualizar buffers e iniciar plotagem em tempo real;
- deixar o backend preparado para evoluir de `read` para `read -> control -> write`;
- desligar a runtime com encerramento seguro e limpar apenas o `runtime_dir`.
