# Senamby Desktop — Arquitetura da UI e Guia de Integração

> Documento focado no frontend do desktop. A UI opera hoje em modo híbrido: já consome comandos Tauri que existem no backend e mantém fallback local nos fluxos que ainda não foram fechados no Rust.

## 1. Objetivo

O frontend foi organizado para cumprir três metas:

- entregar uma experiência completa para o usuário final mesmo antes do backend estar fechado;
- manter contratos estáveis de domínio para que a troca de persistência local por IPC seja previsível;
- escalar para novos módulos, novas abas e novos fluxos sem reescrever a base atual.

Hoje a aplicação já oferece:

- navegação modular;
- catálogo de plugins com criação, edição, exclusão e importação;
- criação e edição de plantas com driver configurável;
- criação de plugins com tipos personalizados além de driver e controlador;
- sincronização automática de `num_sensors` e `num_actuators` no driver a partir das variáveis da planta;
- plotagem e análise histórica;
- importação de JSON exportado para inspeção no Plotter;
- drag and drop funcional em Plotter, Analyzer e Plugins;
- padrão visual compartilhado de abas de workspace entre módulos;
- superfícies principais mais compactas, com uma coluna dominante por módulo e ações concentradas no topo;
- hierarquia visual mais enxuta, com menos texto secundário e menor ruído em áreas de operação.

## 2. Estrutura Geral

```text
routes/+page.svelte
|- Sidebar
|- PlotterModule
|- AnalyzerModule
`- PluginsModule
```

### Princípio de composição

Os módulos principais ficam montados e a página alterna visibilidade. Isso evita perda de contexto quando o usuário navega entre Plotter, Analyzer e Plugins.

### Pastas centrais

```text
src/lib/
|- components/
|- services/
|- stores/
|- types/
`- utils/
```

## 3. Camadas do Frontend

## 3.1 Componentes

Responsáveis por apresentação, interação e composição visual.

- `components/modules/*`: telas principais.
- `components/modals/*`: fluxos de criação, edição e configuração.
- `components/charts/*`: gráficos reutilizáveis.
- `components/layout/*`: shell global.
- `components/ui/*`: primitives reutilizáveis para tabs, toggles e entradas.

Os componentes não devem conhecer `invoke`, `listen` ou detalhes do backend. Eles consomem stores, services e tipos.

### Padrão visual adotado

Para manter homogeneidade e escalar sem retrabalho, os módulos seguem estas regras:

- cabeçalho curto com ação primária e filtros;
- conteúdo principal em uma única superfície dominante;
- abas reutilizáveis para workspaces;
- estados vazios com a mesma hierarquia visual;
- menus auxiliares só aparecem quando reduzem esforço real do usuário;
- contadores e métricas rápidas devem ser legíveis sem depender de subtítulos longos;
- interações recorrentes devem ficar no primeiro nível visual, sem sidebars informativas redundantes.

### Diretrizes de usabilidade e performance

- evitar recarregamentos redundantes quando o dado já está na memória do módulo;
- privilegiar grids simples e cartões leves, sem camadas visuais desnecessárias;
- manter overlays e menus contextuais apenas sob demanda;
- reaproveitar primitives visuais para evitar divergência entre módulos.

## 3.2 Stores

### `appStore`

Arquivo: `src/lib/stores/data.svelte.ts`

Responsável por:

- tema;
- módulo ativo;
- planta ativa;
- lista de plantas abertas;
- visibilidade da sidebar;
- estado do painel de controladores;
- modal de ajustes globais.

### `plantData`

Arquivo: `src/lib/stores/plantData.ts`

Responsável por:

- armazenar as séries temporais por planta;
- armazenar o catálogo de séries por planta com `key`, `label` e `role`;
- armazenar a política de retenção do buffer usado pelo Plotter;
- armazenar estatísticas globais por planta;
- armazenar estatísticas por variável;
- hidratar dados importados pelo Plotter;
- receber ingestão incremental pronta para streaming em tempo real.

Funções importantes:

- `getPlantData`
- `setPlantData`
- `appendPlantData`
- `setPlantBufferConfig`
- `getPlantBufferConfig`
- `setPlantSeriesCatalog`
- `seedPlantSeriesCatalog`
- `getPlantSeriesCatalog`
- `getPlantSeriesLabel`
- `ingestPlantTelemetry`
- `getPlantStats`
- `setPlantStats`
- `getVariableStats`
- `setVariableStats`
- `clearVariableStats`
- `clearPlant`

### `analyzerStore`

Arquivo: `src/lib/stores/analyzerStore.svelte.ts`

Responsável por:

- abas do Analyzer;
- variáveis selecionadas;
- zoom/range;
- modo de visualização dos gráficos;
- visibilidade do painel lateral.

## 3.3 Services

Os serviços são a fronteira entre UI, persistência local e backend.

### `services/plant/plantService.ts`

Hoje já opera em modo híbrido:

- usa Tauri para `list_plants`, `create_plant`, `remove_plant`, `connect_plant`, `disconnect_plant`, `pause_plant` e `resume_plant`;
- mantém fallback local para workspace e overrides de edição quando o backend ainda não cobre o fluxo completo;
- continua fazendo parsing de exportação JSON no frontend para abrir plantas no Plotter;
- ao abrir dados históricos, já devolve também o catálogo de séries do Plotter para manter legendas e tooltips coerentes.

Pontos que ainda podem migrar depois:

- streaming de dados;
- CRUD completo de edição no backend;
- importação real quando o backend assumir essa responsabilidade.

### `services/plugin/pluginService.ts`

Hoje já opera em modo híbrido:

- usa Tauri para listar e criar plugins dos tipos nativos do backend;
- usa `list_plugins_by_type` quando a UI precisa apenas de drivers;
- mantém catálogo local para tipos personalizados, edição local e remoção lógica de itens vindos do backend;
- continua fazendo importação/exportação e validação estrutural no frontend.

Pontos que ainda podem migrar depois:

- atualização e remoção reais no backend;
- publicação/instanciação completa;
- validações executadas pelo runtime Rust.

### `services/fileDialog.ts`

Utilitário de interface para seleção de arquivos.

### `services/export.ts`

Exportação local de CSV/JSON.

### `services/analyzerBackend.ts`

Camada de processamento histórico usada pelo Analyzer.

## 4. Modelos de Domínio que Devem Permanecer Estáveis

Esses tipos já sustentam a UI e devem ser preservados mesmo quando a implementação interna mudar:

- `Plant`
- `PlantVariable`
- `PlantDataPoint`
- `PlantStats`
- `VariableStats`
- `PlantSeriesDescriptor`
- `PlantSeriesCatalog`
- `PluginDefinition`
- `PluginInstance`
- `CreatePlantRequest`
- `CreatePluginRequest`
- `Controller`
- `ControllerParam`

Em outras palavras: o backend pode mudar a fonte dos dados, mas não deveria obrigar a UI a mudar a forma como renderiza cada fluxo.

## 5. Guia de Integração por Módulo

## 5.1 Shell, navegação e estado global

### Arquivos principais

- `src/routes/+page.svelte`
- `src/lib/components/layout/Sidebar.svelte`
- `src/lib/stores/data.svelte.ts`

### Papel

Controla:

- qual módulo está visível;
- tema;
- modais globais;
- sincronização da planta ativa.

### Como integrar com o backend

O shell não deve fazer chamadas de negócio diretamente. Ele só precisa reagir a:

- estado inicial da aplicação;
- preferências persistidas;
- eventos globais como falhas de conexão ou notificações.

### Sugestão de integração

1. Criar um comando `get_app_bootstrap`.
2. Esse comando retorna:
   - tema salvo;
   - lista de plantas restauráveis;
   - plugin catalog resumido;
   - preferências globais.
3. Popular `appStore` no carregamento da página.

### Eventos recomendados

- `app:error`
- `app:notification`
- `app:theme-updated`

## 5.2 Plotter

### Arquivos centrais

- `src/lib/components/modules/PlotterModule.svelte`
- `src/lib/components/plotter/PlotterToolbar.svelte`
- `src/lib/components/plotter/PlantTabs.svelte`
- `src/lib/components/plotter/ControllerPanel.svelte`
- `src/lib/components/charts/VariableGrid.svelte`
- `src/lib/stores/plantData.ts`

### Papel

É o módulo operacional em tempo quase real. Ele mostra:

- plantas abertas;
- variáveis de processo;
- curvas PV/SP/MV;
- estado de conexão;
- controladores e setpoints.

### Estado atual da UI

O Plotter já está pronto para:

- hidratar plantas iniciais do backend no carregamento da página;
- não ter nenhuma planta ativa;
- manter uma aba de workspace `Unnamed` quando não houver planta aberta;
- exibir uma área vazia com opção de criar ou abrir sem quebrar a navegação por abas;
- abrir exportações JSON por seletor e por drag and drop;
- conectar, desconectar, pausar e remover plantas via service;
- editar a planta atualmente selecionada;
- abrir exportações JSON e reconstruir a planta na interface;
- trocar visualização das curvas;
- exportar dados locais;
- usar legendas dinâmicas por série em vez de labels fixos `PV/SP/MV`;
- receber catálogo de séries separado da definição da planta;
- operar em sessões longas com buffer de retenção configurável e renderização adaptativa.

### O que o backend precisa alimentar

#### 1. Lista de plantas

Ao iniciar o app ou abrir uma planta, o backend deve fornecer:

- `id`
- `name`
- `connected`
- `paused`
- `variables`
- `controllers`
- `stats`

Hoje o frontend já hidrata `appStore.state.plants` chamando `list_plants` no carregamento da rota.

#### 2. Séries temporais

O Plotter espera dados no formato `PlantDataPoint[]`, por exemplo:

```ts
{
  time: 12.4,
  var_0_pv: 35.2,
  var_0_sp: 40,
  var_1_pv: 18.7
}
```

Esses dados devem ir para `plantData`.

Para operação contínua, o caminho preferencial agora é ingestão em lote:

```ts
{
  plantId: string;
  points: PlantDataPoint[];
  stats?: PlantStats;
  variableStats?: VariableStats[];
  series?: [
    { key: 'var_0_pv', label: 'Temperatura PV', role: 'pv' },
    { key: 'var_0_sp', label: 'Temperatura SP', role: 'sp' },
    { key: 'var_2_pv', label: 'Válvula Abertura', role: 'mv' }
  ];
}
```

Esse payload pode ser aplicado diretamente com `ingestPlantTelemetry`.

#### 2.1. Catálogo de séries

O frontend agora trabalha com um catálogo por planta:

```ts
{
  plantId: string;
  series: [
    { key: 'var_0_pv', label: 'Temperatura PV', role: 'pv' },
    { key: 'var_0_sp', label: 'Temperatura SP', role: 'sp' },
    { key: 'var_2_pv', label: 'Válvula Abertura', role: 'mv' }
  ]
}
```

Uso desse catálogo:

- nomear legendas, tooltip e menu contextual;
- desacoplar a UI de labels fixos como `PV`, `SP` e `MV`;
- permitir que sensores, setpoints, atuadores e futuras curvas derivadas cheguem com nomes estáveis;
- manter fallback automático gerado a partir de `Plant.variables` quando o backend ainda não enviar metadados.

#### 3. Estatísticas

O toolbar e os cards consomem:

- `PlantStats`
- `VariableStats`

O backend pode:

- calcular isso nativamente e enviar pronto;
- ou enviar apenas dados crus e deixar a UI calcular.

Se a frequência for alta, o ideal é enviar estatísticas já agregadas.

### Como alimentar os gráficos em tempo real

Fluxo recomendado:

1. Usuário conecta a planta.
2. Frontend chama `connect_plant`.
3. Backend inicia a aquisição.
4. Backend publica eventos por planta.
5. Frontend escuta e atualiza `plantData`.

### Eventos recomendados

#### `plant:data`

Payload sugerido:

```ts
{
  plantId: string;
  points: PlantDataPoint[];
  stats?: PlantStats;
  variableStats?: VariableStats[];
  series?: [
    { key: 'var_0_pv', label: 'Temperatura PV', role: 'pv' },
    { key: 'var_0_sp', label: 'Temperatura SP', role: 'sp' },
    { key: 'var_2_pv', label: 'Válvula Abertura', role: 'mv' }
  ];
}
```

Uso:

- `append` ou ingestão em lote no buffer da planta correspondente;
- atualização do gráfico sem remontar o módulo;
- refresh de labels quando o catálogo de séries vier no payload.

#### `plant:stats`

Payload sugerido:

```ts
{
  plantId: string;
  stats: PlantStats;
  variableStats: VariableStats[];
}
```

Uso:

- atualizar toolbar;
- atualizar cards laterais e indicadores.

#### `plant:connection`

Payload sugerido:

```ts
{
  plantId: string;
  connected: boolean;
  paused: boolean;
  reason?: string;
}
```

Uso:

- badges de conexão;
- travar/destravar ações locais.

### Como alimentar novas curvas

A UI já suporta múltiplas curvas por variável, desde que o backend mantenha a convenção de chave.

#### Convenção atual

- sensor PV: `var_{index}_pv`
- sensor SP: `var_{index}_sp`
- atuador/MV: `var_{index}_pv` para a variável do tipo `atuador`

### Estratégias para escalar curvas

#### Estratégia A: manter convenção atual

Mais simples para curto prazo.

#### Estratégia B: introduzir metadados de série

Criar um evento complementar com a definição das séries:

```ts
{
  plantId: string;
  series: [
    { key: 'var_0_pv', label: 'Temperatura PV', role: 'pv' },
    { key: 'var_0_sp', label: 'Temperatura SP', role: 'sp' },
    { key: 'var_2_pv', label: 'Válvula Abertura', role: 'mv' }
  ]
}
```

Essa estratégia passa a ser a preferencial no frontend atual.

Ela é melhor se futuramente houver:

- curvas derivadas;
- predição;
- alarmes;
- envelopes;
- múltiplos controladores por variável.

Na implementação atual:

- o Plotter usa `label` em legendas, tooltips e menu contextual;
- o store faz seed automático de labels a partir da planta para manter compatibilidade;
- quando o backend mandar esse catálogo, ele prevalece sobre o fallback local.

### Retenção e performance para sessões longas

O frontend foi preparado para operar continuamente sem crescimento ilimitado do custo de renderização:

- o buffer de telemetria do Plotter suporta política de retenção por planta via `setPlantBufferConfig`;
- a ingestão incremental usa `appendPlantData` e `ingestPlantTelemetry`, evitando recriar arrays completos a cada amostra;
- o gráfico renderiza uma amostragem adaptativa da janela visível, reduzindo o custo quando a planta acumula muitos pontos;
- a detecção de atualização do gráfico considera assinatura temporal da série, então continua funcionando mesmo com buffer fixo e trim.

Diretriz prática:

- o frontend deve manter um working set para operação;
- o backend deve continuar sendo o responsável por histórico completo, persistência e replay quando isso for requisito.

### Como integrar o painel de controladores

`ControllerPanel.svelte` já possui UI para:

- ativar/desativar controlador;
- renomear controlador;
- editar parâmetros;
- ajustar setpoints;
- remover controlador.

### Fluxo recomendado

1. UI altera um parâmetro.
2. Frontend envia comando específico.
3. Backend confirma com snapshot atualizado do controlador.
4. UI reconcilia o estado local.

### Comandos sugeridos

- `add_controller`
- `update_controller_meta`
- `update_controller_param`
- `remove_controller`
- `update_setpoint`

### Importação de planta no Plotter

Hoje `openPlant` já permite abrir exportação JSON sem Rust.
Esse fluxo já é usado tanto pelo botão de abrir quanto pelo drag and drop.

Quando integrar com backend, há duas opções:

#### Opção A: manter parsing no frontend

Vantagens:

- menor latência para abrir arquivo;
- menos acoplamento com Rust.

#### Opção B: mover parsing para Rust

Vantagens:

- validação única;
- possibilidade de formatos adicionais;
- melhor controle de erros e versionamento.

Se escolher a opção B, manter o shape de `OpenPlantResponse`.

## 5.3 Analyzer

### Arquivos centrais

- `src/lib/components/modules/AnalyzerModule.svelte`
- `src/lib/components/analyzer/AnalyzerTabs.svelte`
- `src/lib/components/analyzer/VariableChart.svelte`
- `src/lib/components/analyzer/VariableSelectorPanel.svelte`
- `src/lib/stores/analyzerStore.svelte.ts`

### Papel

Analisar arquivos históricos fora da operação em tempo real.

### O que o backend precisa alimentar

O Analyzer trabalha melhor com payload já processado por variável.

Estrutura esperada:

- nome da variável/sensor;
- unidade;
- amostras de sensor;
- amostras de setpoint;
- amostras de atuadores vinculados.

### Estratégias de integração

#### Estratégia A: processamento no frontend

Manter o formato atual e usar `analyzerBackend.ts`.
Hoje é isso que acontece, inclusive no drag and drop do módulo.

#### Estratégia B: processamento no backend

Backend recebe arquivo cru, processa e devolve:

- `plantName`
- `processedVariables`
- metadados do experimento

Essa estratégia é melhor se houver:

- arquivos grandes;
- parsing pesado;
- filtros avançados;
- análise estatística nativa.

### Eventos/comandos sugeridos

- `analyzer_process_file`
- `analyzer_load_session`
- `analyzer_export_selection`

### O que não precisa mudar na UI

- multiabas;
- seleção de variáveis;
- range/zoom;
- context menu de séries.

Tudo isso já opera sobre estruturas tipadas e independe da origem do dado.

## 5.4 Plugins

### Arquivos centrais

- `src/lib/components/modules/PluginsModule.svelte`
- `src/lib/components/plugins/PluginCard.svelte`
- `src/lib/components/modals/CreatePluginModal.svelte`
- `src/lib/components/modals/PluginInstanceConfigModal.svelte`
- `src/lib/services/plugin/pluginService.ts`

### Papel

Gerenciar a biblioteca de:

- drivers;
- controladores reutilizáveis;
- tipos personalizados adicionais;
- schemas de configuração;
- código associado.

### Estado atual da UI

A UI já possui:

- listagem;
- busca;
- filtros por categoria;
- criação;
- edição;
- exclusão;
- importação por JSON;
- visualização de código;
- criação de kind personalizado normalizado em `snake_case`.

O módulo foi simplificado para priorizar fluxo: filtros e ações no topo, métricas rápidas em cards e grade principal ocupando toda a largura útil.

### O que o backend precisa fornecer

#### Catálogo

Lista de `PluginDefinition`.

Hoje a UI já mistura:

- plugins vindos do backend para tipos nativos;
- plugins locais para tipos personalizados e overrides.

#### Operações

- criar;
- atualizar;
- remover;
- listar;
- importar/exportar;
- validar;
- instanciar para uso em plantas.

### Comandos sugeridos

- `create_plugin`
- `update_plugin`
- `delete_plugin`
- `list_plugins`
- `list_plugins_by_type`
- `validate_plugin_file`
- `register_plugin`
- `instantiate_plugin`

### Instância de plugin em planta

`PluginInstanceConfigModal.svelte` já está pronto para renderizar campos com base no schema. O backend só precisa respeitar:

- nome do campo;
- tipo do campo;
- valor padrão;
- obrigatoriedade implícita.

Para drivers, a UI também reserva:

- `num_sensors`
- `num_actuators`

Esses campos são preenchidos automaticamente a partir das variáveis da planta, ficam bloqueados no modal de instância e são sincronizados em tempo real durante a criação/edição da planta.

### Fluxo recomendado

1. UI lista plugins.
2. Usuário cria ou edita.
3. Backend valida e persiste.
4. UI atualiza o catálogo com o objeto normalizado retornado.

Não é necessário refazer a tela ao integrar. Basta trocar a implementação do service.

## 6. Como Escalar para Novos Módulos

O padrão atual deve ser repetido:

1. criar um novo `type` para o domínio;
2. criar um novo service dedicado;
3. criar store apenas se houver estado compartilhado do módulo;
4. manter a tela principal em `components/modules`;
5. plugar o módulo em `+page.svelte` e na `Sidebar`.

### Regra prática

Se um fluxo for reutilizável, vira componente.

Se um fluxo falar com backend ou persistência, vira service.

Se o estado precisar sobreviver à navegação interna, vira store.

## 7. Contratos Recomendados para o Backend Tauri

## 7.1 Commands

Sugestão mínima:

- `get_app_bootstrap`
- `list_plants`
- `create_plant`
- `open_plant`
- `remove_plant`
- `connect_plant`
- `disconnect_plant`
- `pause_plant`
- `resume_plant`
- `add_controller`
- `update_controller_meta`
- `update_controller_param`
- `update_setpoint`
- `list_plugins`
- `create_plugin`
- `update_plugin`
- `delete_plugin`
- `validate_plugin_file`
- `register_plugin`
- `analyzer_process_file`

Comandos já integrados no frontend atual:

- `list_plants`
- `create_plant`
- `remove_plant`
- `connect_plant`
- `disconnect_plant`
- `pause_plant`
- `resume_plant`
- `list_plugins`
- `list_plugins_by_type`
- `create_plugin`

## 7.2 Events

Sugestão mínima:

- `plant:data`
- `plant:stats`
- `plant:connection`
- `plugin:updated`
- `plugin:removed`
- `app:error`
- `app:notification`

## 8. Resumo Técnico

O frontend já está pronto para integração porque:

- os módulos estão separados;
- a UI não depende diretamente de `invoke`;
- os dados fluem por tipos previsíveis;
- o Plotter já sabe lidar com planta ausente via workspace `Unnamed`, importação por arquivo e drag and drop;
- plugins e controladores já possuem telas de edição/configuração;
- plantas já podem ser criadas, editadas e reconciliadas em modo híbrido;
- a UI já aceita tipos personalizados de plugin sem ficar acoplada aos tipos nativos do backend;
- o sistema visual já usa primitives reutilizáveis para abas e composição de workspace;
- a troca para backend real pode acontecer na camada de services sem quebrar a experiência do usuário final.
