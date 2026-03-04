# Senamby Desktop — Arquitetura & Guia de Integração

> **Stack:** Tauri 2.0 · SvelteKit 2.0 · Svelte 5 (runes) · TypeScript · uPlot · Tailwind CSS

---

## Sumário

1. [Visão Geral](#1-visão-geral)
2. [Estrutura de Pastas](#2-estrutura-de-pastas)
3. [Arquitetura de Estado](#3-arquitetura-de-estado)
4. [Árvore de Componentes](#4-árvore-de-componentes)
5. [Descrição dos Componentes](#5-descrição-dos-componentes)
6. [Sistema de Tipos](#6-sistema-de-tipos)
7. [Camada de Serviços (Backend)](#7-camada-de-serviços-backend)
8. [Sistema de Gráficos](#8-sistema-de-gráficos)
   - 8.1 [ChartBuilder](#81-chartbuilder)
   - 8.2 [Fluxo de Dados](#82-fluxo-de-dados-do-gráfico)
   - 8.3 [Fluxo Completo de Criação e Renderização](#83-fluxo-completo-de-criação-e-renderização-de-gráficos-no-plotter)
   - 8.4 [Como Criar um Novo Tipo de Gráfico](#84-como-criar-um-novo-tipo-de-gráfico)
   - 8.5 [Gráficos no Analyzer](#85-gráficos-no-analyzer-análise-de-dados-históricos)
   - 8.6 [Exemplos Práticos](#86-exemplos-práticos-de-uso)
   - 8.7 [Detalhes Técnicos de Sincronização](#87-detalhes-técnicos-da-sincronização-de-dados)
9. [Guia de Integração com Backend Real](#9-guia-de-integração-com-backend-real)

---

## 1. Visão Geral

O Senamby Desktop é uma aplicação SCADA (Supervisory Control and Data Acquisition) que roda como app desktop via Tauri. A interface permite monitorar plantas industriais em tempo real com gráficos de tendência, controladores PID configuráveis, sistema de plugins para drivers de comunicação e análise de dados históricos.

### Arquitetura de Módulos

A aplicação usa dois módulos independentes, ambos sempre montados no DOM mas alternados via CSS `display`. Isso preserva o estado interno de cada módulo durante a navegação.

| Módulo | Descrição |
|--------|-----------|
| **Plotter** | Monitoramento em tempo real — gráficos de tendência, multi-plantas, controladores PID, exportação CSV/JSON |
| **Analyzer** | Análise de dados históricos — upload de JSON exportado, visualização de variáveis, zoom/pan, multi-abas |

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri 2.0 (Rust)                     │
│  ┌───────────────────────────────────────────────────┐  │
│  │              SvelteKit (Frontend)                 │  │
│  │  ┌──────┐  ┌──────────────────────────────────┐   │  │
│  │  │      │  │      display: flex | none         │   │  │
│  │  │  S   │  │  ┌────────────────────────────┐   │   │  │
│  │  │  I   │  │  │ PlotterModule              │   │   │  │
│  │  │  D   │  │  │  Charts · Controllers      │   │   │  │
│  │  │  E   │  │  └────────────────────────────┘   │   │  │
│  │  │  B   │  │  ┌────────────────────────────┐   │   │  │
│  │  │  A   │  │  │ AnalyzerModule             │   │   │  │
│  │  │  R   │  │  │  Upload · Análise          │   │   │  │
│  │  │      │  │  └────────────────────────────┘   │   │  │
│  │  └──────┘  └──────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Estrutura de Pastas

```
src/
├── routes/
│   ├── +layout.ts              # SSR desabilitado (ssr = false, prerender = false)
│   ├── +layout.svelte          # Layout raiz (slot vazio)
│   └── +page.svelte            # Página única — orquestra módulos e sidebar
│
├── lib/
│   ├── components/
│   │   ├── charts/             # Componentes de gráficos
│   │   │   ├── PlotlyChart.svelte       # Wrapper uPlot universal
│   │   │   ├── VariableCard.svelte      # Card individual de variável com mini-gráfico
│   │   │   └── VariableGrid.svelte      # Grid de gráficos com modos de visualização
│   │   │
│   │   ├── layout/             # Layout global
│   │   │   ├── Sidebar.svelte           # Barra lateral com navegação de módulos
│   │   │   └── SidebarBtn.svelte        # Botão individual da sidebar
│   │   │
│   │   ├── modals/             # Modais da aplicação
│   │   │   ├── CodeEditorModal.svelte         # Editor de código com syntax highlighting (Python)
│   │   │   ├── CreatePlantModal.svelte        # Wizard de criação de planta (4 etapas)
│   │   │   ├── CreatePluginModal.svelte       # Criação de plugin (driver/controlador)
│   │   │   ├── GenericModal.svelte            # Modal genérico (info/error/warning/success)
│   │   │   ├── GlobalSettingsModal.svelte     # Preferências globais (tema)
│   │   │   ├── PlantRemovalModal.svelte       # Confirmação de remoção de planta
│   │   │   └── PluginInstanceConfigModal.svelte # Configuração de instância de plugin
│   │   │
│   │   ├── modules/            # Módulos principais (telas)
│   │   │   ├── PlotterModule.svelte     # Módulo de tendências em tempo real
│   │   │   └── AnalyzerModule.svelte    # Módulo de análise de dados históricos
│   │   │
│   │   ├── plotter/            # Sub-componentes do Plotter
│   │   │   ├── ChartContextMenu.svelte  # Menu de contexto por variável
│   │   │   ├── ControllerPanel.svelte   # Painel lateral de controladores PID
│   │   │   ├── PlantAddMenu.svelte      # Menu dropdown para adicionar plantas
│   │   │   ├── PlantTabs.svelte         # Abas de navegação entre plantas
│   │   │   └── PlotterToolbar.svelte    # Barra de ferramentas do plotter
│   │   │
│   │   ├── analyzer/           # Sub-componentes do Analyzer
│   │   │   ├── AnalyzerTabs.svelte          # Abas de arquivos abertos no analyzer
│   │   │   ├── VariableChart.svelte         # Gráfico individual de variável analisada
│   │   │   └── VariableSelectorPanel.svelte # Painel de seleção de variáveis
│   │   │
│   │   └── ui/                 # Componentes UI reutilizáveis
│   │       ├── DynamicParamInput.svelte # Input dinâmico por tipo de parâmetro
│   │       └── SimpleToggle.svelte      # Toggle switch reusável
│   │
│   ├── services/               # Camada de serviços (backend mock)
│   │   ├── api.ts                   # Ponto único de re-export (fachada)
│   │   ├── plantBackend.ts          # CRUD de plantas e drivers
│   │   ├── pluginBackend.ts         # Sistema de plugins/drivers
│   │   ├── analyzerBackend.ts       # Processamento de arquivos JSON para análise
│   │   ├── export.ts                # Exportação de dados (CSV/JSON)
│   │   ├── fileDialog.ts            # Diálogos de arquivo (mock, substituir por Tauri)
│   │   └── simulation.ts            # Simulação de processo em tempo real
│   │
│   ├── stores/                 # Gerenciamento de estado
│   │   ├── data.svelte.ts          # AppStore global (Svelte 5 runes)
│   │   ├── plantData.ts            # Dados de séries temporais por planta (Map)
│   │   └── analyzerStore.svelte.ts # Estado do módulo Analyzer
│   │
│   ├── types/                  # Definições TypeScript
│   │   ├── app.ts                  # AppState (estado global)
│   │   ├── plant.ts                # Plant, PlantVariable, PlantDataPoint
│   │   ├── controller.ts           # Controller, ControllerParam, PIDParams
│   │   ├── plugin.ts               # PluginDefinition, PluginInstance, schemas
│   │   ├── driver.ts               # DriverConfig, DriverType
│   │   ├── chart.ts                # ChartState, SeriesConfig, ViewMode
│   │   ├── analyzer.ts             # ProcessedVariableData, AnalyzerFile
│   │   ├── plantExport.ts          # PlantExportJSON (formato de exportação)
│   │   └── ui.ts                   # TabKey, MODULE_TABS
│   │
│   └── utils/                  # Utilitários
│       ├── chartBuilder.ts         # Builder de configuração de gráficos uPlot
│       └── format.ts               # Formatação de números, IDs, tempo
```

---

## 3. Arquitetura de Estado

### 3.1 AppStore (`stores/data.svelte.ts`)

Store global usando Svelte 5 runes (`$state`). Classe singleton exportada como `appStore`.

**Estado (`AppState`):**
```typescript
interface AppState {
  theme: 'dark' | 'light';
  activeModule: TabKey;          // 'plotter' | 'analyzer'
  activePlantId: string;
  sidebarCollapsed: boolean;
  showGlobalSettings: boolean;
  showControllerPanel: boolean;
  plants: Plant[];
}
```

**Métodos principais:**
| Método | O que faz |
|--------|-----------|
| `setTheme(theme)` | Altera tema dark/light |
| `setActiveModule(module)` | Muda o módulo ativo na sidebar |
| `setActivePlantId(id)` | Seleciona a planta ativa |
| `addPlant(plant)` | Adiciona nova planta ao estado |
| `removePlant(plantId)` | Remove planta (ajusta activePlantId se necessário) |
| `toggleConnect(plantId)` | Alterna estado connected da planta |
| `togglePause(plantId)` | Alterna estado paused da planta |
| `addController(plantId, ctrl)` | Adiciona controlador a uma planta |
| `deleteController(plantId, ctrlId)` | Remove controlador |
| `updateControllerParam(plantId, ctrlId, key, value)` | Atualiza parâmetro de controlador |
| `updateVariableSetpoint(plantId, varIdx, sp)` | Atualiza setpoint de variável |
| `addVariable(plantId, variable)` | Adiciona variável a uma planta |
| `removeVariable(plantId, varIdx)` | Remove variável de uma planta |

### 3.2 PlantData (`stores/plantData.ts`)

Armazena dados de séries temporais por planta usando `Map`. Não é reativo — lido por referência nos efeitos de renderização.

| Função | O que faz |
|--------|-----------|
| `getPlantData(plantId)` | Retorna array de `PlantDataPoint[]` |
| `getPlantStats(plantId)` | Retorna estatísticas globais da planta |
| `setPlantStats(plantId, stats)` | Atualiza estatísticas da planta |
| `getVariableStats(plantId, varIndex)` | Retorna stats de uma variável específica |
| `setVariableStats(plantId, varIndex, stats)` | Atualiza stats de variável |
| `clearPlant(plantId)` | Limpa todos os dados da planta |

**Formato dos dados (`PlantDataPoint`):**
```typescript
interface PlantDataPoint {
  time: number;
  var_0_pv: number;   // Process Value da variável 0
  var_0_sp: number;   // Setpoint da variável 0
  var_0_mv: number;   // Manipulated Variable da variável 0
  var_1_pv: number;   // ... variável 1
  // ... para cada variável
}
```

### 3.3 AnalyzerStore (`stores/analyzerStore.svelte.ts`)

Store do módulo Analyzer usando Svelte 5 runes. Gerencia arquivos abertos, variáveis processadas e estado de visualização.

---

## 4. Árvore de Componentes

```
+page.svelte
├── Sidebar
│   └── SidebarBtn (× N módulos)
├── PlotterModule                    [display: flex|none]
│   ├── PlantTabs
│   │   └── PlantAddMenu
│   ├── PlotterToolbar
│   ├── VariableGrid
│   │   └── VariableCard (× N variáveis)
│   │       └── PlotlyChart
│   ├── ChartContextMenu
│   ├── ControllerPanel
│   │   ├── DynamicParamInput (× N params)
│   │   └── SimpleToggle
│   ├── PlantRemovalModal
│   └── CreatePlantModal
│       ├── CreatePluginModal
│       │   └── CodeEditorModal
│       └── PluginInstanceConfigModal
├── AnalyzerModule                   [display: flex|none]
│   ├── AnalyzerTabs
│   ├── VariableSelectorPanel
│   └── VariableChart (× N variáveis)
│       └── PlotlyChart
└── GlobalSettingsModal
```

---

## 5. Descrição dos Componentes

### 5.1 Página Principal (`+page.svelte`)

Orquestra toda a aplicação. Gerencia:
- Efeito de tema (dark/light) no `<html>` e `<body>`
- Inicia simulação de processo via `startSimulation()`
- Renderiza `Sidebar`, `PlotterModule`, `AnalyzerModule` e `GlobalSettingsModal`
- Alterna módulos via CSS `display: flex | none` (ambos permanecem montados)

### 5.2 Módulo Plotter (`PlotterModule.svelte`)

Tela principal de monitoramento em tempo real. Recebe via props: `plants`, `activePlantId`, `theme`, `active`.

**Estado interno:**
- `chartState`: Configuração de visualização (viewMode, cores, zoom, séries visíveis)
- `displayTick`: Contador incrementado via `requestAnimationFrame` quando `active=true`
- `contextMenu`: Estado do menu de contexto por variável
- `showControllerPanel`: Visibilidade do painel lateral
- `createPlantModal`, `removeModal`: Estado dos modais

**Modos de visualização (`viewMode`):**
- `grid` — Todos os gráficos em grade
- `stacked` — Gráficos empilhados verticalmente
- `overlay` — Todas variáveis em um único gráfico
- `focused` — Uma variável ampliada (focusedVariableIndex)

**Funcionalidades:**
- Navegação por teclado (1-4 para modos, Escape para resetar zoom)
- Menu de contexto por variável (click direito nos cards) com toggle de visibilidade e seletor de cor
- Exportação CSV e JSON da planta ativa
- Timer de uptime controlado pelo prop `active`
- Cálculo de estatísticas por variável em cada `displayTick`

### 5.3 Módulo Analyzer (`AnalyzerModule.svelte`)

Tela de análise de dados históricos. Recebe: `theme`, `active`.

**Funcionalidades:**
- Upload e processamento de arquivos JSON (formato `PlantExportJSON`)
- Multi-abas (vários arquivos abertos simultaneamente)
- Painel lateral de seleção de variáveis com toggle individual
- Gráficos por variável com zoom/pan
- Atalho de teclado (Ctrl+O para abrir arquivo) quando `active=true`

### 5.4 Charts

**PlotlyChart.svelte** — Wrapper universal do uPlot. Cria, atualiza e destrói instâncias uPlot. Recebe `series` (configuração de linhas), `getData()` (função que retorna dados), `opts` (opções do uPlot). Usa `requestAnimationFrame` para renderizar e `$effect` para sincronizar dados. Suporta temas dark/light e callbacks de `onRangeChange`.

**VariableCard.svelte** — Card individual que exibe: nome da variável, tipo (sensor/atuador), valores atuais (PV, SP, MV), mini-gráfico via `PlotlyChart`, e indicadores de erro/estabilidade. Clicável para menu de contexto.

**VariableGrid.svelte** — Componente **crítico** que renderiza `VariableCard` em diferentes layouts conforme o `viewMode`. Implementa os 4 modos de visualização:

| Modo | Layout | CSS |
|------|--------|-----|
| **grid** | Cards lado a lado em grade | `display: grid; grid-template-columns: repeat(auto-fit, minmax(400px, 1fr))` |
| **stacked** | Cards empilhados verticalmente | `display: flex; flex-direction: column; width: 100%` |
| **overlay** | Um único gráfico com todas variáveis | Um único `PlotlyChart` com séries de todas as variáveis combinadas |
| **focused** | Uma variável ampliada (full-width) | Card selecionado em `display: flex; width: 100%`, outros em `display: none` |

```svelte
<!-- VariableGrid.svelte simplified -->
<script>
  let viewMode = 'grid'; // muda via PlotterToolbar
  let focusedVariableIndex = 0;
</script>

<div class={cn('variable-container', viewMode === 'overlay' ? 'single-overlay' : '')}>
  {#if viewMode === 'overlay'}
    <!-- Um único gráfico com TODAS variáveis -->
    <PlotlyChart 
      series={combineAllSeries(plant.variables)}
      getData={() => getPlantData(plant.id)}
    />
  {:else}
    <!-- Cards individuais (grid, stacked ou focused) -->
    <div class="grid" style="display: {viewMode === 'stacked' ? 'flex' : 'grid'}">
      {#each plant.variables as variable, varIndex (variable.id)}
        {#if viewMode !== 'focused' || focusedVariableIndex === varIndex}
          <VariableCard 
            {variable} 
            {varIndex}
            {plant}
            isFocused={viewMode === 'focused' && focusedVariableIndex === varIndex}
            {displayTick}
          />
        {/if}
      {/each}
    </div>
  {/if}
</div>

<style>
  .grid[style*="display: grid"] {
    grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  }
  .grid[style*="display: flex"] {
    flex-direction: column;
  }
</style>
```

### 5.5 Plotter Sub-componentes

**PlantTabs.svelte** — Abas de navegação entre plantas. Inclui botão "+" com `PlantAddMenu` para criar nova planta ou abrir arquivo.

**PlotterToolbar.svelte** — Barra com: nome da planta, status de conexão/pausa, estatísticas (dt, erro, estabilidade, uptime), botões de modo de visualização, exportação e painel de controladores.

**ControllerPanel.svelte** — Painel lateral deslizante com lista de controladores PID. Cada controlador tem: toggle ativo/inativo, campo de tipo, campos de parâmetros dinâmicos (`DynamicParamInput`), controle de setpoint por variável.

**ChartContextMenu.svelte** — Menu de contexto ativado por click direito em um card de variável. Permite: toggle de visibilidade das séries (PV, SP, MV), alteração de cor da série.

### 5.6 Modais

**CreatePlantModal.svelte** — Wizard de 4 etapas: Informações → Driver → Variáveis → Controladores. Permite selecionar/criar plugin de driver, definir variáveis (sensores/atuadores com vínculos), e adicionar controladores a partir de templates.

**CreatePluginModal.svelte** — Criação de plugin (driver ou controlador). Campos: nome, tipo, runtime (Python), código fonte (file picker ou editor inline), dependências Python (pip), schema de configuração (campos dinâmicos com tipos: string, int, float, bool, list).

**CodeEditorModal.svelte** — Editor de código com syntax highlighting via highlight.js. Features: numeração de linhas, highlight em tempo real com debounce rAF, sincronização de scroll entre textarea e overlay, suporte a Tab para indentação.

**PluginInstanceConfigModal.svelte** — Configura uma instância de plugin para uma planta. Renderiza cada campo do schema com input apropriado ao tipo: toggle para bool, number para int/float, text para string, array builder para list.

**PlantRemovalModal.svelte** — Confirmação de remoção de planta com nome e motivo.

**GlobalSettingsModal.svelte** — Preferências globais (seletor de tema dark/light).

**GenericModal.svelte** — Modal genérico reutilizável para mensagens (info, error, warning, success).

### 5.7 UI Reutilizáveis

**DynamicParamInput.svelte** — Input que renderiza campo de número, toggle booleano ou texto baseado no `ControllerParam.type`.

**SimpleToggle.svelte** — Toggle switch on/off reutilizável.

---

## 6. Sistema de Tipos

### 6.1 Plant (`types/plant.ts`)

```typescript
interface Plant {
  id: string;
  name: string;
  connected: boolean;
  paused: boolean;
  variables: PlantVariable[];
  stats: PlantStats;
  controllers: Controller[];
}

interface PlantVariable {
  id: string;               // 'var_0', 'var_1', ...
  name: string;
  type: 'sensor' | 'atuador';
  unit: string;
  setpoint: number;
  pvMin: number;
  pvMax: number;
  mvMin: number;
  mvMax: number;
  linkedSensorIds?: string[]; // Somente atuadores — IDs dos sensores vinculados
}
```

### 6.2 Controller (`types/controller.ts`)

```typescript
interface Controller {
  id: string;
  name: string;
  type: 'PID' | 'Flow' | 'Level';
  active: boolean;
  params: PIDParams | Record<string, ControllerParam>;
}

interface ControllerParam {
  type: 'number' | 'boolean' | 'string';
  value: number | boolean | string;
  label: string;
}
```

### 6.3 Plugin (`types/plugin.ts`)

```typescript
interface PluginDefinition {
  id: string;
  name: string;
  kind: 'driver' | 'controller';
  runtime: 'python' | 'rust-native';
  sourceFile: string;
  sourceCode?: string;
  schema: PluginSchemaField[];
  dependencies?: PluginDependency[];
  description?: string;
  version?: string;
}

interface PluginInstance {
  pluginId: string;
  pluginName: string;
  pluginKind: PluginKind;
  config: Record<string, SchemaFieldValue>;
}
```

### 6.4 Chart (`types/chart.ts`)

Define `ChartState` (modo, zoom, séries), `SeriesConfig` (cor, visibilidade), `ViewMode` (`grid | stacked | overlay | focused`).

### 6.5 Export (`types/plantExport.ts`)

Define `PlantExportJSON` — formato de exportação compatível com o Analyzer (sensores, atuadores, amostras de dados).

---

## 7. Camada de Serviços (Backend)

Todos os serviços estão em `lib/services/` e usam implementações mock com `mockDelay()`. O arquivo `api.ts` é o ponto único de importação.

### 7.1 api.ts — Fachada Central

Re-exporta todas as funções de todos os serviços. **Qualquer componente que precise chamar o backend importa de `$lib/services/api`**.

```typescript
// Exemplo de uso em um componente:
import { createPlant, listPlugins, exportPlantDataCSV } from '$lib/services/api';
```

### 7.2 Serviços Disponíveis

| Serviço | Funções | Backend Real |
|---------|---------|--------------|
| **plantBackend** | `createPlant`, `openPlant`, `saveDriver`, `listDrivers`, `listControllerTemplates` | Tauri IPC → Rust |
| **pluginBackend** | `listPlugins`, `getPlugin`, `validatePluginFile`, `registerPlugin`, `validatePluginInstanceConfig`, `validateDriverSourceCode` | Tauri IPC → Rust + Python runtime |
| **analyzerBackend** | `processJSONFile` | Pode permanecer no frontend |
| **export** | `exportPlantDataCSV`, `exportPlantDataJSON`, `buildPlantExportJSON` | Tauri IPC → save_file dialog |
| **fileDialog** | `openFileDialog`, `openFilesDialog`, `readFileAsText`, `readFileAsJSON` | Tauri dialog API |
| **simulation** | `startSimulation` | Substituir por dados reais do hardware |

### 7.3 Contratos de Request/Response

Cada serviço define interfaces tipadas para request e response:

```typescript
// plantBackend.ts
interface CreatePlantRequest {
  name: string;
  driverId: string;
  variables: PlantVariable[];
  controllers: Controller[];
}

interface CreatePlantResponse {
  success: boolean;
  plant?: Plant;
  error?: string;
}
```

Todas as responses seguem o padrão `{ success: boolean; data?: T; error?: string }`.

---

## 8. Sistema de Gráficos

### 8.1 ChartBuilder (`utils/chartBuilder.ts`)

Builder fluent para configurar gráficos uPlot. Gera `series` e `opts` completos.

**Uso:**
```typescript
const { series, opts } = createChartBuilder('dark')
  .addPvSeries(variable, config, stats)
  .addSpSeries(variable, config)
  .addMvSeries(variable, config)
  .buildOpts({ width, height, windowSize, xMin, xMax, showGrid: true, showHover: true });
```

**Séries disponíveis:**
- **PV (Process Value)** — Linha sólida, cor configurável
- **SP (Setpoint)** — Linha tracejada, cor configurável
- **MV (Manipulated Variable)** — Linha pontilhada, cor configurável

### 8.2 Fluxo de Dados do Gráfico

```
simulation.ts → plantData (Map) → PlotterModule ($effect com rAF)
                                        ↓
                                  displayTick incrementa
                                        ↓
                                  VariableGrid ← getData()
                                        ↓
                                  VariableCard → PlotlyChart → uPlot
```

O `PlotterModule` incrementa `displayTick` a cada frame via `requestAnimationFrame` quando `active=true`. Os componentes derivados leem `displayTick` para saber quando re-renderizar.

### 8.3 Fluxo Completo de Criação e Renderização de Gráficos no Plotter

#### **Passo 1: Adicionar uma Planta**

Quando você clica no botão **"+"** em `PlantTabs.svelte` ou seleciona "Criar Planta", o fluxo é:

1. **CreatePlantModal.svelte** abre (4 etapas: info → driver → variáveis → controladores)
2. Usuário preenche:
   - Nome da planta
   - Driver (conexão com hardware)
   - **Variáveis** (lista de sensores e atuadores com limites min/max)
   - Controladores PID (opcional)
3. Ao confirm, chama `createPlant(request)` via backend
4. **AppStore** recebe a nova planta via `appStore.addPlant(plant)` 
5. A planta é adicionada ao estado reativo

#### **Passo 2: Inicialização de Dados da Planta**

Quando a planta é adicionada, **automaticamente** são criados:

1. **PlantData (Map)** — Uma entry na Map global de dados da planta:
   ```typescript
   // Em stores/plantData.ts
   getPlantData(plant.id);  // Retorna [] vazio inicialmente
   ```

2. **Stats da Planta** — Estrutura para armazenar estatísticas:
   ```typescript
   setPlantStats(plant.id, {
     dt: 0,
     error: 0,
     stability: 0,
     uptime: 0
   });
   ```

3. **Stats por Variável** — Para cada variável da planta:
   ```typescript
   for (let varIdx = 0; varIdx < plant.variables.length; varIdx++) {
     setVariableStats(plant.id, varIdx, {
       avgPv: 0, maxPv: 0, minPv: 0,
       avgError: 0, maxError: 0
     });
   }
   ```

#### **Passo 3: Renderização Automática de Gráficos por Variável**

Quando `PlotterModule.svelte` detecta que a planta foi adicionada:

1. **VariableGrid.svelte** recebe:
   - `plant.variables` (array de variáveis)
   - `activePlantId`
   - `chartState` (modo de visualização, cores, zoom)

2. **VariableGrid** itera sobre cada variável e renderiza um **VariableCard** para cada uma:
   ```svelte
   {#each plant.variables as variable, varIndex (variable.id)}
     <VariableCard 
       {variable} 
       varIndex={varIndex}
       {plant}
       {chartState}
       {displayTick}
     />
   {/each}
   ```

3. **VariableCard.svelte** para cada variável:
   - Extrai dados dela do `plantData(Map)` usando `getPlantData(plant.id)`
   - Calcula configuração de gráfico via `ChartBuilder`:
     ```typescript
     // VariableCard.svelte
     const { series, opts } = createChartBuilder(theme)
       .addPvSeries(variable, seriesConfig, stats)
       .addSpSeries(variable, seriesConfig, stats)
       .addMvSeries(variable, seriesConfig, stats)
       .buildOpts({ width, height, windowSize, xMin, xMax, showGrid: true, showHover: true });
     ```
   - Renderiza o gráfico via **PlotlyChart.svelte**

4. **PlotlyChart.svelte** (wrapper uPlot):
   - Cria instância do uPlot com `series` e `opts`
   - A cada `displayTick`, chama `getData()` para buscar dados atualizados
   - Re-renderiza o gráfico com dados novos via `requestAnimationFrame`

#### **Fluxo Visual de Renderização**
```
PlotterModule (ativo)
  ↓
  $effect(() => {
    if (active) {
      requestAnimationFrame(() => displayTick++);
    }
  })
  ↓
  VariableGrid (lê displayTick)
  ↓ (para cada variável)
  VariableCard (lê displayTick)
  ↓
  ChartBuilder (recalcula config se mudou tema/zoom)
  ↓
  PlotlyChart (getData() → plotar dados novos)
  ↓
  uPlot (renderiza canvas)
```

**A cada frame (quando `displayTick` muda):**
- VariableCard recalcula dados estatísticos
- PlotlyChart chama `getData()` que retorna dados da Map
- uPlot renderiza o canvas com novos dados

#### **Diagrama de Fluxo Simplificado**

```
┌─ Adicionar Planta via Modal ───────────┐
│ • Nome, Driver, Variáveis, Controllers │
└──────────────────┬────────────────────┘
                   │
                   ↓
        ┌─ AppStore.addPlant() ─────────────┐
        │ • Planta entra no estado reativo   │
        │ • Planta.variables[] é populado    │
        └────────────┬──────────────────────┘
                     │
        ┌────────────┴─────────────────────────┐
        │                                      │
        ↓                                      ↓
PlantData.getPlantData(id)        PlantData.setPlantStats()
(Array[] inicializado)             (Stats da planta)
                                   
                                   ┌─────────────────────┐
                                   │  Para cada variável │
                                   │  setVariableStats() │
                                   └─────────────────────┘

    ┌────────────────────────────────────────────────┐
    │   PlotterModule (ativo=true)                   │
    │   • Inicia requestAnimationFrame                │
    │   • Incrementa displayTick a cada frame         │
    └────────────────┬─────────────────────────────┘
                     │
                     ↓
        ┌─ VariableGrid recebe ────────────────┐
        │ plant.variables[] do AppStore         │
        │ displayTick reativo                   │
        └────────────┬──────────────────────────┘
                     │
    ┌────────────────┴──────────────────────────┐
    │ Para cada PlantVariable (sensor/atuador)  │
    └────────────────∨──────────────────────────┘
                     │
         ┌───────────┴────────────┐
         │ {#each plant.variables}│
         └────────────┬───────────┘
                      │
                      ↓
        ┌─ VariableCard ────────────────────┐
        │ • Recebe variable, varIndex        │
        │ • Lê displayTick (gatilho rAF)     │
        │ • ChartBuilder constrói config     │
        │ • renderiza PlotlyChart            │
        └────────────┬─────────────────────┘
                     │
                     ↓
        ┌─ ChartBuilder.buildSeries() ──────┐
        │ • addPvSeries()                   │
        │ • addSpSeries()                   │
        │ • addMvSeries()                   │
        │ • buildOpts({width, height, ...}) │
        └────────────┬─────────────────────┘
                     │
                     ↓
        ┌─ PlotlyChart (uPlot Wrapper) ─────┐
        │ • $effect: monitora displayTick    │
        │ • getData() retorna PlantData      │
        │ • uPlot.setData() atualiza canvas  │
        │ • renderiza no <canvas>            │
        └───────────────────────────────────┘
                     ↑
         ┌───────────┴────────────────┐
         │ Dados fluem de:            │
         │ • Simulação (simulation.ts)│
         │ • Hardware real (Tauri IPC)│
         │ • Armazenados em Map       │
         └────────────────────────────┘
```

### 8.4 Como Criar um Novo Tipo de Gráfico

#### **Cenário: Adicionar um novo tipo de série "Desvio Padrão"**

##### Passo 1: Adicionar tipo de série ao TypeScript

```typescript
// types/chart.ts
export type SeriesType = 'pv' | 'sp' | 'mv' | 'stddev';  // ← Novo tipo

export interface SeriesConfig {
  visible: boolean;
  color: string;
  type: SeriesType;
}
```

##### Passo 2: Adicionar método ao ChartBuilder

```typescript
// utils/chartBuilder.ts
export class ChartBuilder {
  // ... métodos existentes ...

  addStddevSeries(variable: PlantVariable, config: SeriesConfig, stats: VariableStats) {
    const seriesName = `${variable.name} StdDev`;
    
    return this
      .addSeries(seriesName, {
        label: seriesName,
        stroke: config.color,
        width: 1,
        dash: [5, 5],  // ← Linha tracejada diferente
        show: config.visible,
      });
  }
}
```

##### Passo 3: Usar no VariableCard

```svelte
<!-- VariableCard.svelte -->
<script>
  const { series, opts } = createChartBuilder(theme)
    .addPvSeries(variable, pvConfig, stats)
    .addSpSeries(variable, spConfig, stats)
    .addMvSeries(variable, mvConfig, stats)
    .addStddevSeries(variable, stddevConfig, stats)  // ← Add aqui
    .buildOpts({ ... });
</script>
```

##### Passo 4: Permitir toggle no Menu de Contexto

```svelte
<!-- ChartContextMenu.svelte -->
<button on:click={() => toggleSeries(varIndex, 'stddev')}>
  Desvio Padrão {chartState.visibleSeries.includes('stddev') ? '✓' : ''}
</button>
```

### 8.5 Gráficos no Analyzer (Análise de Dados Históricos)

O Analyzer renderiza gráficos a partir de **dados já coletados** (em vez de tempo real).

#### **Fluxo de Dados no Analyzer**

```
Upload JSON (PlantExportJSON)
  ↓
analyzerBackend.processJSONFile()
  ↓
AnalyzerStore (carrega arquivo + variáveis disponíveis)
  ↓
AnalyzerTabs (exibe abas de arquivos abertos)
  ↓ (usuário seleciona variáveis)
VariableSelectorPanel (checkbox de variáveis)
  ↓
VariableChart[] (renderiza gráfico por variável)
  ↓
PlotlyChart → uPlot (exibe dados históricos)
```

#### **Diferenças vs Plotter**

| Aspecto | Plotter | Analyzer |
|---------|---------|----------|
| **Fonte de dados** | `getPlantData(plantId)` (Map) | Array de pontos do arquivo JSON |
| **Atualização** | A cada `displayTick` via rAF | Estática (dados históricos) |
| **Interatividade** | Zoom/pan com xMin/xMax | Zoom/pan completo com seleção de range |
| **Configuração** | ChartBuilder calcula a cada frame | Calculada uma vez ao abrir arquivo |

#### **Como o Analyzer Carrega um Arquivo**

1. **Upload**: AnalyzerTabs → file picker → `openFileDialog()`
2. **Processamento**: `analyzerBackend.processJSONFile(json)` retorna:
   ```typescript
   interface ProcessedData {
     fileName: string;
     variables: ProcessedVariableData[];  // Uma por variável do arquivo
   }
   
   interface ProcessedVariableData {
     name: string;
     unit: string;
     points: { time: number; pv: number; sp?: number; mv?: number }[];
     stats: { min: number; max: number; avg: number };
   }
   ```

3. **Armazenamento**: `analyzerStore.openFile(processed)` adiciona à lista de abas
4. **Renderização**: Para cada variável selecionada, `VariableChart` renderiza um gráfico:
   ```svelte
   <!-- VariableChart.svelte (Analyzer) -->
   <PlotlyChart 
     series={buildAnalyzerSeries(processedVariable)}
     getData={() => processedVariable.points}
     opts={buildAnalyzerOpts(theme)}
   />
   ```

### 8.6 Exemplos Práticos de Uso

#### **Exemplo 1: Adicionar uma Planta e Ver Gráficos Automaticamente**

```
1. Clique no botão "+" em PlantTabs
2. CreatePlantModal abre
3. Preencha:
   - Nome: "Tanque A"
   - Driver: Selecione driver de comunicação (ex: Modbus)
   - Variáveis: Adicione "Temperatura" (sensor, 0-100°C), "Válvula" (atuador, 0-100%)
   - Controladores: (opcional) Adicione PID para temperatura
4. Clique Finish
5. RESULTADO: Automaticamente aparecem 2 cards de gráfico
   - Um para Temperatura (com série PV tracejada)
   - Um para Válvula (com série MV pontilhada)
6. Dados começam a chegar via simulação/driver real
7. Gráficos atualizam em tempo real
```

#### **Exemplo 2: Personalizar a Cor de uma Série**

```
1. Clique direito em um VariableCard
2. ChartContextMenu aparece
3. Escolha uma série (ex: "Temperatura Process Value")
4. Clique no seletor de cor → Escolha cor nova
5. RESULTADO: Série PV da temperatura muda de cor
6. Cor é persistida em chartState.seriesConfig[variableId]
```

#### **Exemplo 3: Mudar Modo de Visualização**

```
1. Em PlotterToolbar, clique em um dos botões de modo:
   - [Grid] → Todos gráficos lado a lado
   - [Stacked] → Gráficos empilhados verticalmente
   - [Overlay] → Todas variáveis em um único gráfico
   - [Focused] → Uma variável ampliada

2. RESULTADO: VariableGrid recalcula layout dos cards
   - Grid: CSS Grid com 2-3 colunas
   - Stacked: Flexbox column 100% width
   - Overlay: Um único PlotlyChart com todas séries
   - Focused: VariableCard expandido, outros com display:none
```

#### **Exemplo 4: Analisar Dados Históricos no Analyzer**

```
1. Exporte dados do Plotter clicando no botão "📥 Exportar JSON"
2. Mude para o módulo Analyzer (sidebar)
3. Clique em "+" ou Ctrl+O para abrir arquivo
4. Selecione arquivo JSON exportado
5. analyzerBackend.processJSONFile() converte dados
6. Abrir arquivo: VariableSelectorPanel mostra lista de variáveis
7. Selecione variáveis para visualizar (ex: Temperatura, Válvula)
8. Para cada variável selecionada, um VariableChart renderiza
9. Você pode fazer zoom nos dados históricos (clique e arraste no gráfico)
```

### 8.7 Detalhes Técnicos da Sincronização de Dados

#### **Como PlantData é Mantida Sincronizada**

A `Map<plantId, PlantDataPoint[]>` é o ponto central:

1. **Dados entram via `simulate()` ou listeners Tauri**:
   ```typescript
   // simulation.ts
   const data = getPlantData(plantId);
   data.push({
     time: Date.now(),
     var_0_pv: sensorValue,
     var_0_sp: setpoint,
     var_0_mv: output,
     // ... mais variáveis
   });
   
   // Limpar dados antigos (janela deslizante)
   if (data.length > MAX_POINTS) data.shift();
   ```

2. **PlotterModule lê via `displayTick`**:
   ```typescript
   // PlotterModule.svelte
   $effect(() => {
     if (!active) return;
     const frame = requestAnimationFrame(() => displayTick++);
     return () => cancelAnimationFrame(frame);
   });
   ```

3. **VariableCard extrai dados para seu gráfico**:
   ```typescript
   // VariableCard.svelte
   const getData = () => {
     const points = getPlantData(plant.id);
     return points.map(p => ({
       time: p.time,
       pv: p[`var_${varIndex}_pv`],
       sp: p[`var_${varIndex}_sp`],
       mv: p[`var_${varIndex}_mv`],
     }));
   }
   ```

4. **PlotlyChart renderiza via uPlot**:
   ```typescript
   // PlotlyChart.svelte
   $effect(() => {
     if (!instance) return;
     const data = getData();
     instance.setData([
       // time axis
       data.map(p => p.time),
       // PV series
       data.map(p => p.pv),
       // SP series
       data.map(p => p.sp),
       // MV series
       data.map(p => p.mv),
     ]);
   });
   ```

#### **Performance: Por que usar Map em vez de Store Reativo**

- **PlantData é não-reativo** — Evita re-render desnecessário de toda a árvore
- **displayTick é reativo** — Dispara re-cálculos apenas quando há novos frames
- **VariableCard lê a Map por referência** → Dados atualizados sem trigger de reatividade
- **Resultado**: 60 FPS possível com centenas de pontos de dados por planta

#### **Limites de Performance**

```typescript
const MAX_POINTS = 10000;  // Máximo de pontos por planta em memória

// Se simulação roda a 100ms = 10 pontos/segundo:
// 10000 pontos / 10 pts/seg = 1000 segundos ≈ 17 minutos de histórico
```

Para históricos mais longos, considere:
- Agregar dados (ex: média a cada 1 segundo, em vez de cada 100ms)
- Usar IndexedDB no frontend para persistir histórico
- Arquivar dados antigos no backend Rust

---

## 8.8 FAQ — Perguntas Frequentes sobre Gráficos

### **P: Quando eu adiciono uma planta, ela cria automaticamente todos os gráficos de cada variável?**

**R: SIM.** Aqui está como funciona:

1. Você preenche o `CreatePlantModal` com as variáveis (ex: Temperatura, Pressão, Válvula)
2. Clica "Finish"
3. `appStore.addPlant(plant)` é chamado
4. `PlotterModule` já está renderizando porque `plants` é reativo
5. `VariableGrid` recebe `plant.variables` que agora tem 3 elementos
6. Automaticamente via `{#each plant.variables}`, 3 `VariableCard`s são criados
7. Cada um renderiza um `PlotlyChart` com suas séries (PV, SP, MV)
8. **Resultado**: 3 gráficos aparecem na tela instantaneamente

Não precisa de código extra — Svelte renderiza os cards automaticamente quando a variável é adicionada.

### **P: Como exatamente um gráfico é criado no Plotter?**

**R: 7 passos:**

```
1. PlotterModule.svelte coloca plant.variables no estado
   ↓
2. VariableGrid.svelte detecta mudança (plant recalculado)
   ↓
3. Loop {#each plant.variables} cria um VariableCard por variável
   ↓
4. VariableCard.svelte:
   - Calcula configuração via ChartBuilder
   - Passa series, getData(), opts para PlotlyChart
   ↓
5. PlotlyChart.svelte:
   - $effect monitora displayTick
   - Cada frame: getData() retorna dados atuais
   ↓
6. uPlot instance:
   - Recebe arrays de dados (time, PV, SP, MV)
   - Renderiza no canvas
   ↓
7. Resultado: Gráfico vivo e atualizado 60x por segundo
```

### **P: Como os gráficos no Analyzer são criados?**

**R: Flow diferente do Plotter:**

```
FILE UPLOAD
   ↓
analyzerBackend.processJSONFile(json)
   ↓
Retorna: ProcessedVariableData[] com dados históricos
   ↓
AnalyzerStore.openFile(processed) — Adiciona à lista de abas
   ↓
AnalyzerTabs renderiza abas
   ↓
VariableSelectorPanel mostra checkbox de variáveis
   ↓
Usuário seleciona variáveis (ex: Temperatura, Pressão)
   ↓
Para cada variável selecionada:
   - VariableChart renderiza
   - PlotlyChart recebe dados históricos (já prontos)
   - uPlot renderiza (não há atualização em tempo real)
   ↓
Resultado: Gráficos estáticos de dados históricos
```

**Diferença chave**: No Plotter, `getData()` retorna dados atualizados a cada frame. No Analyzer, retorna sempre o mesmo array (histórico).

### **P: Como mudo as cores dos gráficos?**

**R: Menu de contexto (clique direito em um VariableCard):**

```
ChartContextMenu.svelte:
  - Mostra lista de séries da variável
  - Cada série tem:
    • Toggle de visibilidade (olho)
    • Seletor de cor (color picker)
  - Ao selecionar cor nova:
    • chartState.seriesConfig[varIndex] atualiza
    • ChartBuilder recalcula com cor nova
    • PlotlyChart re-renderiza
```

### **P: Como adiciono uma nova série (ex: desvio padrão)?**

**R: 3 arquivos a modificar:**

```typescript
// 1. types/chart.ts — Adicione ao tipo SeriesType
export type SeriesType = 'pv' | 'sp' | 'mv' | 'stddev';

// 2. utils/chartBuilder.ts — Adicione método ao builder
ChartBuilder.prototype.addStddevSeries = function(variable, config, stats) {
  return this.addSeries('StdDev', { stroke: config.color, dash: [5,5] });
}

// 3. VariableCard.svelte — Use no builder
chart = createChartBuilder(theme)
  .addPvSeries(...)
  .addSpSeries(...)
  .addMvSeries(...)
  .addStddevSeries(...)  // ← Nova série
  .buildOpts(...)
```

Feito! O gráfico vai mostrar a série nova automaticamente.

### **P: Qual é o limite de variáveis/gráficos por planta?**

**R: Técnico:**
- **Limite prático**: ~20-30 variáveis antes de performance degradar
- **Razão**: Cada `VariableCard` = uma instância uPlot
- **Solução**: Use modo `overlay` (todas variáveis em 1 gráfico) para plantas grandes
- **Performance**: Modo grid com 4 variáveis = ~60 FPS. Modo grid com 16 = ~30 FPS.

### **P: Como os dados históricos são salvos?**

**R: Exportação**

```
PlotterToolbar → Botão "Exportar JSON"
   ↓
export.ts: buildPlantExportJSON(plant)
   ↓
Converte plantData (Map) para PlantExportJSON
   ↓
Download de arquivo (no frontend atual) ou save dialog (com Tauri)
   ↓
Usuário pode abrir no Analyzer para análise
```

Dados são **apenas em memória** durante a execução. Para persistir:
- Implementar salvar periódico no backend Rust
- Ou usar IndexedDB no frontend

---

## 8.9 Checklist: Como Customizar Gráficos em 5 Passos

Se você quer **modificar a aparência ou comportamento dos gráficos**, este é o checklist:

### **1️⃣  Mudança Visual (Cores, Estilos)**

- [ ] Abra `utils/chartBuilder.ts`
- [ ] Encontre o método `.addPvSeries()`, `.addSpSeries()`, etc
- [ ] Modifique `stroke`, `width`, `dash` (linha tracejada)
- [ ] Salve → Mude a cor/estilo automaticamente

**Exemplo**:
```typescript
.addPvSeries(...) {
  return this.addSeries(..., {
    stroke: '#FF0000',  // Mudou para vermelho
    width: 2,            // Mais grossa
    dash: [10, 5]        // Tracejada
  })
}
```

### **2️⃣  Adicionar Nova Série (PV, SP, MV, Outra)**

- [ ] Adicione tipo em `types/chart.ts`
- [ ] Adicione método em `utils/chartBuilder.ts`
- [ ] Use no `VariableCard.svelte` via builder
- [ ] Teste após reload

### **3️⃣  Mudar Layout (Grid, Stacked, Overlay, Focused)**

- [ ] Abra `VariableGrid.svelte`
- [ ] Modifique CSS ou lógica condicional
- [ ] Atualize `viewMode` em `PlotterModule`
- [ ] Novos layouts aparecem em tempo real

### **4️⃣  Adicionar Informação ao Card (Stats, Valores)**

- [ ] Abra `VariableCard.svelte`
- [ ] Adicione `{variable.maxValue}` ao template HTML
- [ ] Ou chame `getVariableStats()` para stats calculadas
- [ ] CSS styling conforme necessário

### **5️⃣  Mudar Comportamento do Gráfico (Zoom, Pan, Range)**

- [ ] `PlotlyChart.svelte` → Modifique `onRangeChange` callback
- [ ] `VariableCard.svelte` → Chame `ChartBuilder.buildOpts()` com opções diferentes
- [ ] uPlot suporta: `scales`, `bounds`, `range`, `redraw`, etc

**Referência uPlot**: [https://uplot.js.org/docs/api](https://uplot.js.org/docs/api)

---

## 9. Guia de Integração com Backend Real

### 9.1 Visão Geral da Migração

A interface está preparada para substituir os mocks por chamadas Tauri IPC. O processo é:

1. **Não mexa nos componentes** — Eles importam de `$lib/services/api`
2. **Substitua as funções nos arquivos de serviço** — Cada `services/*.ts` tem funções mock para substituir
3. **Mantenha os mesmos tipos de request/response** — Os contratos estão definidos em `types/`

### 9.2 Passo a Passo por Serviço

#### plantBackend.ts — Substituir cada função mock

```typescript
// ANTES (mock):
export async function createPlant(request: CreatePlantRequest): Promise<CreatePlantResponse> {
  await mockDelay();
  // lógica mock...
  return { success: true, plant: mockPlant };
}

// DEPOIS (Tauri IPC):
import { invoke } from '@tauri-apps/api/core';

export async function createPlant(request: CreatePlantRequest): Promise<CreatePlantResponse> {
  return await invoke('create_plant', { request });
}
```

**Funções a substituir:**
- `createPlant(request)` → `invoke('create_plant', { request })`
- `openPlant(request)` → `invoke('open_plant', { request })`
- `saveDriver(request)` → `invoke('save_driver', { request })`
- `listDrivers()` → `invoke('list_drivers')`
- `listControllerTemplates()` → `invoke('list_controller_templates')`

#### pluginBackend.ts

- `listPlugins(kind?)` → `invoke('list_plugins', { kind })`
- `getPlugin(id)` → `invoke('get_plugin', { id })`
- `registerPlugin(plugin)` → `invoke('register_plugin', { plugin })`
- `validatePluginFile(json)` → `invoke('validate_plugin_file', { json })`
- `validatePluginInstanceConfig(pluginId, config)` → `invoke('validate_plugin_config', { pluginId, config })`
- `validateDriverSourceCode(code, className)` → `invoke('validate_driver_source', { code, className })`

#### fileDialog.ts

Substituir por `@tauri-apps/plugin-dialog`:

```typescript
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';

export async function openFileDialog(options: OpenFileOptions): Promise<FileResult | null> {
  const selected = await open({
    title: options.title,
    filters: options.filters?.map(f => ({ name: f.name, extensions: f.extensions })),
  });
  if (!selected) return null;
  const content = await readTextFile(selected);
  return {
    name: selected.split('/').pop() || 'file',
    file: content,
  };
}
```

#### export.ts

Substituir `downloadBlob()` por save dialog do Tauri:

```typescript
import { save } from '@tauri-apps/plugin-dialog';
import { writeTextFile } from '@tauri-apps/plugin-fs';

// Em vez de criar blob + URL.createObjectURL, usar:
const filePath = await save({ defaultPath: 'dados.csv', filters: [{ name: 'CSV', extensions: ['csv'] }] });
if (filePath) await writeTextFile(filePath, csvContent);
```

#### simulation.ts — Substituir por dados reais

A simulação atual gera dados fictícios a cada 100ms. Para integração real:

1. **Remover** `startSimulation()` de `+page.svelte`
2. **Criar listener** de dados do hardware via Tauri event:

```typescript
import { listen } from '@tauri-apps/api/event';

// Em +page.svelte ou no PlotterModule:
listen<PlantDataPoint>('plant-data', (event) => {
  const plantId = event.payload.plantId;
  const data = getPlantData(plantId);
  data.push(event.payload.point);
  if (data.length > MAX_POINTS) data.shift();
});
```

3. **No Rust**, emitir eventos de dados quando o driver envia novos valores.

### 9.3 Fluxo de Dados Real (após integração)

```
Hardware → Driver Python → Rust Backend → Tauri Event
                                              ↓
                                    listen('plant-data')
                                              ↓
                                    plantData (Map) → UI
```

### 9.4 Checklist de Integração

- [ ] Instalar plugins Tauri: `@tauri-apps/plugin-dialog`, `@tauri-apps/plugin-fs`
- [ ] Implementar comandos Rust: `create_plant`, `open_plant`, `save_driver`, `list_drivers`, `list_controller_templates`
- [ ] Implementar comandos Rust: `list_plugins`, `register_plugin`, `validate_plugin_file`, `validate_plugin_config`, `validate_driver_source`
- [ ] Substituir `fileDialog.ts` por Tauri dialog API
- [ ] Substituir `export.ts` download por Tauri save dialog
- [ ] Substituir `simulation.ts` por listener de eventos Tauri
- [ ] Remover `mockDelay()` e dados mock de todos os serviços
- [ ] Manter `analyzerBackend.ts` (processamento local de JSON pode permanecer)
- [ ] Manter `api.ts` como fachada de re-exports (não precisa mudar)
- [ ] Testar: criar planta → conectar driver → receber dados → visualizar no gráfico

### 9.5 Pontos Importantes

- **`api.ts` não muda.** Ele apenas re-exporta. Os componentes continuam importando dele.
- **Os tipos não mudam.** Os contratos em `types/` são o contrato entre frontend e backend.
- **`plantData.ts` é o ponto de injeção de dados.** Use `getPlantData(plantId).push(point)` para inserir dados reais.
- **O `displayTick` no PlotterModule já usa rAF.** Dados inseridos no Map serão renderizados automaticamente no próximo frame.
- **Ambos módulos estão sempre montados.** Não precisa se preocupar com lifecycle — os efeitos são controlados pelo prop `active`.
