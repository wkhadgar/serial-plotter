# Senamby Desktop — Arquitetura & Guia do Desenvolvedor

> **Stack:** Tauri 2.0 · SvelteKit 2.0 · Svelte 5 (runes) · TypeScript · uPlot · Tailwind CSS  
> **Última atualização:** Março 2026

---

## Sumário

1. [Visão Geral](#1-visão-geral)
2. [Estrutura de Pastas](#2-estrutura-de-pastas)
3. [Arquitetura de Estado](#3-arquitetura-de-estado)
4. [Árvore de Componentes](#4-árvore-de-componentes)
5. [Fluxo de Dados](#5-fluxo-de-dados)
6. [Detalhamento dos Componentes](#6-detalhamento-dos-componentes)
7. [Sistema de Tipos](#7-sistema-de-tipos)
8. [Sistema Universal de Gráficos](#8-sistema-universal-de-gráficos)
9. [Como Fazer: Receitas Comuns](#9-como-fazer-receitas-comuns)
   - [Adicionar um módulo na sidebar](#91-adicionar-um-novo-módulo-na-sidebar)
   - [Enviar dados para plotagem](#92-enviar-dados-para-plotagem)
   - [Remover um módulo existente](#93-remover-um-módulo-existente)
   - [Alterar taxa de atualização](#94-alterar-a-taxa-de-atualização-do-plot)
   - [Integrar dados reais do backend](#95-integrar-dados-reais-do-backend-tauri)
   - [Adicionar uma nova variável ao gráfico](#96-adicionar-uma-nova-variável-ao-gráfico)
   - [Criar um novo tipo de controlador](#97-criar-um-novo-tipo-de-controlador)
   - [Usar o módulo Analyzer](#98-usar-o-módulo-analyzer)

---

## 1. Visão Geral

O Senamby Desktop é uma aplicação SCADA (Supervisory Control and Data Acquisition) que roda como app desktop via Tauri. A interface permite monitorar plantas industriais em tempo real com gráficos de tendência, controladores PID configuráveis, e análise de dados históricos via CSV.

### 1.1 Arquitetura de Módulos

A aplicação utiliza uma arquitetura modular onde cada **botão na Sidebar** representa um **módulo independente**. Cada módulo é uma tela completa com funcionalidades específicas que preenchem a área principal da aplicação.

| Módulo | Ícone | Descrição | Funcionalidades |
|--------|-------|-----------|-----------------|
| **Plotter** | 📈 TrendingUp | Monitoramento em tempo real | Gráficos de tendência, multi-plantas, controladores PID, exportação CSV |
| **Analyzer** | 📊 BarChart3 | Análise de dados históricos | Upload de CSV, visualização de variáveis, zoom/pan, multi-abas |

**Características dos módulos:**
- **Isolados:** Cada módulo gerencia seu próprio estado interno
- **Independentes:** Não há comunicação direta entre módulos
- **Extensíveis:** Novos módulos podem ser adicionados via `MODULE_TABS` em `types/ui.ts`
- **Consistentes:** Todos usam o mesmo sistema de gráficos (`PlotlyChart` + `ChartBuilder`)

**Navegação:** O estado `activeModule` em `AppStore` controla qual módulo está visível. Apenas um módulo é renderizado por vez (renderização condicional em `+page.svelte`).

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri (Rust)                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │              SvelteKit (Frontend)                 │  │
│  │  ┌──────┐  ┌──────────────────────────────────┐   │  │
│  │  │      │  │         Módulo Ativo             │   │  │
│  │  │  S   │  │  ┌────────────────────────────┐  │   │  │
│  │  │  I   │  │  │ PlotterModule              │  │   │  │
│  │  │  D   │  │  │  ┌──────┐ ┌─────────────┐ │  │   │  │
│  │  │  E   │  │  │  │Chart │ │ Controller  │ │  │   │  │
│  │  │  B   │  │  │  │(uPlot)│ │   Panel     │ │  │   │  │
│  │  │  A   │  │  │  └──────┘ └─────────────┘ │  │   │  │
│  │  │  R   │  │  └────────────────────────────┘  │   │  │
│  │  │      │  │  OU: AnalyzerModule              │   │  │
│  │  └──────┘  └──────────────────────────────────┘   │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 2. Estrutura de Pastas

```
src/
├── routes/                          # Páginas SvelteKit
│   ├── +layout.ts                   # SSR desabilitado (ssr = false)
│   ├── +layout.svelte               # Layout raiz (vazio, só <slot>)
│   └── +page.svelte                 # Página única — orquestra tudo
│
├── lib/
│   ├── components/
│   │   ├── charts/                  # Componentes de renderização de gráficos
│   │   │   └── PlotlyChart.svelte   # Wrapper uPlot (473 linhas)
│   │   │
│   │   ├── layout/                  # Componentes de layout global
│   │   │   ├── Sidebar.svelte       # Barra lateral com navegação
│   │   │   └── SidebarBtn.svelte    # Botão individual da sidebar
│   │   │
│   │   ├── modals/                  # Todos os modais da aplicação
│   │   │   ├── GlobalSettingsModal.svelte   # Preferências globais
│   │   │   ├── PlantRemovalModal.svelte     # Confirmação de remoção de planta
│   │   │   └── GenericModal.svelte          # Modal genérico reutilizável (info/error/warning/success)
│   │   │
│   │   ├── modules/                 # Módulos de feature (telas principais)
│   │   │   ├── PlotterModule.svelte         # Tela de tendências (principal)
│   │   │   └── AnalyzerModule.svelte        # Tela de análise de CSV
│   │   │
│   │   ├── analyzer/                # Sub-componentes do Analyzer
│   │   │   ├── AnalyzerTabs.svelte          # Abas de arquivos CSV
│   │   │   ├── VariableSelectorPanel.svelte # Painel de seleção de variáveis
│   │   │   └── VariableChart.svelte         # Gráficos de variável (sensor+target+actuator)
│   │   │
│   │   ├── plotter/                 # Sub-componentes exclusivos do Plotter
│   │   │   ├── PlantTabs.svelte             # Abas das plantas
│   │   │   ├── PlotterToolbar.svelte        # Toolbar (conectar, pausar, exportar)
│   │   │   ├── ChartContextMenu.svelte      # Menu de contexto (botão direito)
│   │   │   └── ControllerPanel.svelte       # Painel lateral de controladores
│   │   │
│   │   └── ui/                      # Primitivas reutilizáveis
│   │       ├── DynamicParamInput.svelte     # Input dinâmico para parâmetros
│   │       └── SimpleToggle.svelte          # Toggle simples on/off
│   │
│   ├── services/                    # Lógica de negócio (sem UI)
│   │   ├── simulation.ts           # Simulação PID + transferência térmica
│   │   ├── export.ts               # Exportação CSV
│   │   └── analyzerBackend.ts      # Backend mockado (processa CSV) - TODO: integrar com Rust
│   │
│   ├── stores/                      # Estado global
│   │   ├── data.svelte.ts           # AppStore (estado reativo com $state)
│   │   ├── plantData.ts             # Buffer de dados plain (Map, sem reatividade)
│   │   └── analyzerStore.svelte.ts  # Estado persistente do Analyzer (abas, seleções)
│   │
│   ├── types/                       # Interfaces e tipos TypeScript
│   │   ├── app.ts                   # AppState
│   │   ├── analyzer.ts              # AnalyzerVariable, ProcessedVariableData
│   │   ├── chart.ts                 # ChartConfig, ChartSeries, ChartStateType
│   │   ├── controller.ts            # Controller, ControllerParam, PIDParams
│   │   ├── plant.ts                 # Plant, PlantDataPoint, PlantStats
│   │   └── ui.ts                    # TabKey, MODULE_TABS
│   │
│   └── utils/                       # Funções puras utilitárias
│       ├── format.ts                # formatTime, generateId
│       └── chartBuilder.ts          # ChartBuilder (classe para criar gráficos facilmente)
```

**Convenções de pasta:**

| Pasta | Regra |
|-------|-------|
| `components/charts/` | Componentes que renderizam gráficos (wrappers de libs externas) |
| `components/layout/` | Componentes de estrutura visual que aparecem em todas as telas |
| `components/modals/` | Qualquer dialog/modal overlay |
| `components/modules/` | Telas inteiras que preenchem a `<main>` — uma por item da sidebar |
| `components/plotter/` | Sub-componentes exclusivos do PlotterModule |
| `components/ui/` | Primitivas genéricas reutilizáveis (botões, toggles, inputs) |
| `services/` | Lógica de negócio pura (sem import de Svelte) |
| `stores/` | Estado reativo global |
| `types/` | Interfaces TypeScript, sem lógica |
| `utils/` | Funções puras utilitárias |

---

## 3. Arquitetura de Estado

O app usa **duas camadas de estado** complementares:

### 3.1 AppStore (`stores/data.svelte.ts`) — Estado Reativo

Classe singleton com `$state<AppState>()` do Svelte 5. Contém **tudo que a UI precisa reagir**:

```typescript
interface AppState {
  theme: 'dark' | 'light';        // Tema atual
  activeModule: TabKey;            // Módulo ativo ('plotter' | 'analyzer')
  activePlantId: string;           // Planta selecionada
  sidebarCollapsed: boolean;       // Sidebar expandida/recolhida
  showGlobalSettings: boolean;     // Modal de preferências
  showControllerPanel: boolean;    // Painel de controladores
  plants: Plant[];                 // Lista de plantas (config + controllers)
}
```

**Acesso:** `appStore.state.plants`, `appStore.state.theme`, etc.  
**Mutação:** Sempre via métodos: `appStore.toggleTheme()`, `appStore.addPlant(...)`, etc.

### 3.2 plantData (`stores/plantData.ts`) — Buffer de Dados Plain

`Map<string, PlantDataPoint[]>` **sem reatividade** Svelte. Armazena os pontos de dados brutos que alimentam os gráficos.

```typescript
getPlantData(plantId)    // → PlantDataPoint[] (referência direta ao array)
getPlantStats(plantId)   // → PlantStats
setPlantStats(plantId, stats)
clearPlant(plantId)      // Limpa dados e stats
```

**Por que plain Map?** Dados de série temporal a 10Hz gerariam milhares de invalidações reativas por segundo. O array plain é mutado via `.push()` e o gráfico lê diretamente via `setInterval(33ms)` a ~30fps — sem overhead de reatividade.

### 3.3 Como a UI lê dados não-reativos

O `PlotterModule` usa um **tick de display** para forçar re-leitura periódica:

```typescript
let _displayTick = $state(0);
onMount(() => { _displayTimer = setInterval(() => _displayTick++, 33); });

const currentPV = $derived.by(() => {
  _displayTick; // Toca a reatividade a cada 33ms (~30fps)
  const data = getPlantData(activePlantId);
  return data.length > 0 ? data[data.length - 1].pv : 0;
});
```

O `PlotlyChart` (uPlot) usa seu próprio `setInterval(33ms)` interno (~30fps) para checar se há dados novos e chamar `chart.setData()`.

### 3.4 analyzerStore (`stores/analyzerStore.svelte.ts`) — Estado do Analyzer

Classe singleton com `$state` para gerenciar o estado do módulo Analyzer. **Estado persiste ao trocar de módulo.**

```typescript
interface AnalyzerTab {
  id: string;
  name: string;                      // "Unnamed" ou nome do arquivo
  processedVariables: ProcessedVariableData[];
  selectedVariablesIndexes: number[];
}

class AnalyzerStore {
  tabs = $state<AnalyzerTab[]>([]);           // Abas abertas
  activeTabId = $state<string>('');           // Aba ativa
  chartStates = $state<Record<string, ChartStateType>>({});  // Zoom/pan por aba
  showVariablePanel = $state<boolean>(false); // Painel lateral visível
  
  // Getters
  get activeTab(): AnalyzerTab | undefined;
  get chartState(): ChartStateType;
  get selectedVariables(): ProcessedVariableData[];
  get isActiveTabEmpty(): boolean;
  
  // Métodos
  createEmptyTab(): void;                     // Cria aba "Unnamed"
  loadFileToActiveTab(fileName, data): void;  // Carrega CSV na aba ativa
  removeTab(tabId): void;                     // Remove aba (mantém pelo menos 1)
  selectTab(tabId): void;
  toggleVariable(index): void;
  setRange(xMin, xMax): void;
  resetZoom(): void;
}
```

**Comportamento de abas:**
- Ao abrir o Analyzer: sempre tem pelo menos 1 aba ("Unnamed" se vazia)
- Ao clicar "+": cria nova aba "Unnamed" com área de drop
- Ao carregar CSV: renomeia aba para nome do arquivo
- Ao fechar última aba: limpa dados (não remove a aba)

---

## 4. Árvore de Componentes

```
+page.svelte
├── Sidebar
│   └── SidebarBtn (× N módulos + tema + ajustes)
│
├── PlotterModule                ← quando activeModule === 'plotter'
│   ├── PlantTabs                    Abas de plantas (add/remove/select)
│   ├── PlotterToolbar               Conectar, pausar, exportar, stats, status
│   ├── ChartContextMenu             Menu direito (eixos X/Y, cores, visibilidade)
│   ├── PlotlyChart (PV+SP)          Gráfico uPlot superior (2/3 da altura)
│   ├── PlotlyChart (MV)             Gráfico uPlot inferior (1/3 da altura)
│   ├── ControllerPanel              Painel lateral (setpoint, PIDs)
│   └── PlantRemovalModal            Modal de confirmação de remoção
│
├── AnalyzerModule               ← quando activeModule === 'analyzer'
│   ├── AnalyzerTabs                 Abas de arquivos CSV (add/remove/select)
│   ├── ChartContextMenu             Menu direito (zoom, reset, etc.)
│   ├── VariableSelectorPanel        Painel lateral (selecionar variáveis)
│   ├── VariableChart (grid)         Grid de gráficos (sensor+target+actuator)
│   └── GenericModal                 Modal de erro ao processar CSV
│
└── GlobalSettingsModal              Preferências gerais (gridlines, etc.)
```

**Fluxo de props:** Top-down. O `+page.svelte` passa `appStore.state.*` como props. Mutações sobem via callbacks (`onToggleConnect`, `onExport`, etc.) que chamam `appStore.*()`.

---

## 5. Fluxo de Dados

### 5.1 Simulação → Gráfico (atual, com simulação local)

```
┌──────────────┐    push()     ┌──────────────┐  setInterval   ┌──────────────┐
│  simulation  │──────────────→│  plantData   │───(33ms)──────→│  PlotlyChart │
│   .ts        │               │  Map<id,[]>  │   ~30fps       │   (uPlot)    │
│  (10Hz)      │               │  plain array │                │  setData()   │
└──────────────┘               └──────────────┘                └──────────────┘
       ↑                              │
       │ lê plant config              │ _displayTick (33ms)
       │                              ↓
┌──────────────┐               ┌──────────────┐
│   AppStore   │←──callbacks───│  PlotterModule│
│   $state     │               │  (orchestrator│
└──────────────┘               └──────────────┘
```

### 5.2 Interação do Usuário

```
Usuário clica "LIGAR" no PlotterToolbar
  → PlotterToolbar.onToggleConnect()
    → PlotterModule.handleToggleConnect()
      → appStore.toggleConnect(plantId)
        → plant.connected = true  ($state reage)
          → simulation.ts vê plant.connected === true
            → começa a gerar dados para esse plantId
```

### 5.3 Zoom/Pan no Gráfico

```
Usuário faz drag-select no gráfico
  → uPlot setSelect hook
    → PlotlyChart.onRangeChange(xMin, xMax)
      → PlotterModule.handleRangeChange()
        → chartState.xMode = 'manual'
        → chartState.xMin/xMax atualizados
          → pvSpConfig/mvConfig (derived) reatualizam
            → PlotlyChart $effect detecta mudança
              → syncScaleRef() + updateChart()

Usuário dá duplo-clique / clica "Home"
  → PlotterModule.resetZoom()
    → chartState.xMode = 'auto', yMin=0, yMax=100
```

---

## 6. Detalhamento dos Componentes

### 6.1 `+page.svelte` (65 linhas)

**Papel:** Orquestrador raiz. Responsabilidades mínimas:
- Aplica tema dark/light no `<html>` e `<body>`
- Inicia simulação (`startSimulation`)
- Renderiza `Sidebar` + módulo ativo + `GlobalSettingsModal`

### 6.2 `PlotterModule.svelte` (380 linhas)

**Papel:** Orquestrador do módulo de tendências. Gerencia:
- Estado do gráfico por planta (`chartStates: Record<string, ChartStateType>`)
- Cores das linhas (`lineColors`)
- Estado do menu de contexto e modal de remoção
- Derivações: `currentPV`, `currentMV`, `pvSpSeries`, `mvSeries`, configs
- Handlers: add/remove plant, connect, pause, export, zoom, controllers

### 6.3 `PlotlyChart.svelte` (473 linhas)

**Papel:** Wrapper do uPlot. Totalmente genérico — recebe `series[]` e `config`.
- Gerencia ciclo de vida do uPlot (`initChart`, `updateChart`, `destroy`)
- Tooltip customizado com posicionamento inteligente
- Zoom por scroll (wheel), pan por Shift+drag, drag-select para zoom
- `_scaleRef` (plain object) evita stale closures nos callbacks do uPlot
- `$effect` separados para: mudanças que recriam o chart (tema, séries) vs. que só atualizam (eixos)

### 6.4 `Sidebar.svelte` (77 linhas)

**Papel:** Navegação entre módulos. Cada botão na seção principal representa um **módulo** (tela completa com funcionalidades específicas).

**Estrutura:**
- **Logo/Toggle:** Primeiro elemento, permite expandir/recolher a sidebar. Exibe "Senamby" quando expandida.
- **Área de Módulos:** Botões `SidebarBtn` mapeados de `MODULE_TABS`. Cada botão ativa um módulo diferente.
- **Área de Utilidades:** Botões de tema e configurações globais (não são módulos).

**Módulos disponíveis:**
| Botão | Módulo | Descrição |
|-------|--------|-----------|
| 📈 Tendências | `PlotterModule` | Monitoramento em tempo real |
| 📊 Analyzer | `AnalyzerModule` | Análise de dados CSV |

**Comportamento:**
- Colapsável (64px ↔ 256px) via `appStore.toggleSidebar()`
- Módulo ativo indicado por destaque visual no botão
- Seção inferior: toggle tema + ajustes (separador visual)

### 6.5 `ControllerPanel.svelte` (125 linhas)

**Papel:** Painel lateral deslizante. Permite configurar:
- Setpoint (slider 0–100%)
- Controladores (add, remove, toggle, editar parâmetros via `DynamicParamInput`)

### 6.6 `ChartContextMenu.svelte` (115 linhas)

**Papel:** Menu de contexto (botão direito na área do chart). Permite:
- Alternar modo do eixo X (Auto/Janela/Manual)
- Alternar modo do eixo Y (Auto/Manual) com campos min/max
- Toggle visibilidade e cor de cada variável (PV/SP/MV)

### 6.7 `AnalyzerModule.svelte` (~350 linhas)

**Papel:** Módulo de análise de dados históricos via CSV. Permite carregar múltiplos arquivos CSV em abas separadas e visualizar variáveis com as mesmas funcionalidades do PlotterModule (zoom, pan, seleção, botão direito).

**Arquitetura:** Segue exatamente o padrão do PlotterModule:
- Estado de tabs local (`tabs: AnalyzerTab[]`)
- Estado do gráfico por aba (`chartStates: Record<string, ChartStateType>`)
- Mesmos handlers de zoom/pan/context menu
- Usa o mesmo `ChartContextMenu` e `PlotlyChart`

**Fluxo:**
1. Upload de arquivo CSV via input ou drag-and-drop
2. **Backend mockado** (`analyzerBackend.ts`) processa o CSV:
   - Valida formato
   - Extrai variáveis (grupos sensor/actuator/target)
   - Calcula ranges ótimos para cada gráfico
   - Retorna dados estruturados prontos
3. Nova aba é criada com dados processados
4. Seleção interativa de variáveis via `VariableSelectorPanel`
5. Grid responsível com múltiplas variáveis simultaneamente

**Funcionalidades (idênticas ao PlotterModule):**
- ✅ Zoom por seleção (drag para criar área)
- ✅ Pan (Shift+drag ou middle-click)
- ✅ Double-click para resetar zoom
- ✅ Botão direito abre `ChartContextMenu`
- ✅ Modos Auto/Sliding/Manual
- ✅ Cores customizáveis por variável
- ✅ Multi-abas com estado independente
- ✅ Drag-and-drop de arquivos CSV

**Error handling:** Usa `GenericModal` ao invés de `alert()` para mensagens de erro.

**Formato CSV esperado:**
```
seconds, sensor_0, actuator_0, target_0, sensor_1, actuator_1, target_1, ...
0.0, 25.3, 12.5, 20.0, 30.1, 8.2, 28.0, ...
0.1, 25.5, 13.1, 20.0, 30.3, 8.5, 28.0, ...
```

### 6.8 `VariableSelectorPanel.svelte` (~70 linhas)

**Papel:** Painel lateral deslizante do Analyzer. **Estilo consistente com `ControllerPanel`:**
- Mesma animação (slide from right)
- Botão de fechar no header
- Cards de seleção com checkbox visual
- Indicadores de cor para sensor/actuator/target
- `$bindable` para controle do estado `visible`

### 6.9 `VariableChart.svelte` (~100 linhas)

**Papel:** Card de visualização de uma variável. Recebe **dados pré-processados** do backend:
- Gráfico superior: sensor (azul) + target (laranja tracejado) com range otimizado
- Gráfico inferior: actuator (verde) com range otimizado
- Usa 2 instâncias de `PlotlyChart` com `yMode: 'manual'` e ranges calculados pelo backend
- Zero processamento de dados (apenas mapeamento para formato do chart)

### 6.10 `GenericModal.svelte` (~60 linhas)

**Papel:** Modal genérico reutilizável para mensagens ao usuário. Substitui `alert()` e `confirm()`.

**Tipos suportados:**
- `info`: mensagens informativas (azul)
- `error`: erros (vermelho)
- `warning`: avisos (âmbar)
- `success`: sucessos (verde)

**Props:**
```typescript
{
  visible: boolean;
  type: 'info' | 'error' | 'warning' | 'success';
  title: string;
  message: string;
  confirmLabel?: string;  // padrão: 'OK'
  onConfirm?: () => void;
  onClose?: () => void;   // botão Cancelar só aparece se onClose estiver definido
}
```

---

## 7. Sistema de Tipos

### `types/plant.ts`

```typescript
PlantDataPoint { time, sp, pv, mv, [key: string]: number }
PlantStats     { errorAvg, stability, uptime }
Plant          { id, name, connected, paused, data, setpoint, stats, controllers }
```

### `types/controller.ts`

```typescript
ControllerType  = 'PID' | 'Flow' | 'Level'
ControllerParam { type: 'number'|'boolean'|'string', value, label }
PIDParams       { kp, ki, kd, manualMode }  (cada um é ControllerParam)
Controller      { id, name, type, active, params }
```

### `types/chart.ts`

```typescript
ChartDataPoint  { time, [key: string]: number }
ChartSeries     { key, label, color, visible, data, dataKey, type?, strokeWidth?, dashed? }
ChartConfig     { yMin, yMax, yMode, xMode, windowSize, xMin?, xMax?, showGrid, ... }
ChartStateType  { xMode, yMode, xMin, xMax, yMin, yMax, windowSize, visible: {pv,sp,mv} }
LineColors      { pv, sp, mv }
```

### `types/app.ts`

```typescript
AppState { theme, activeModule, activePlantId, sidebarCollapsed, showGlobalSettings, showControllerPanel, plants }
```

### `types/analyzer.ts`

```typescript
AnalyzerVariable        { index, name, sensor, actuator, target, selected }
ProcessedVariableData   { variable, sensorData, targetData, actuatorData, sensorRange, actuatorRange }
```

**Importante:** O Analyzer processa dados do backend e armazena em `ProcessedVariableData`.

### `types/ui.ts`

```typescript
TabKey = 'plotter' | 'analyzer'
MODULE_TABS = { plotter: { label, icon }, analyzer: { label, icon } }
```

---

## 8. Sistema Universal de Gráficos

O Senamby utiliza um **sistema de gráficos universal e reutilizável** baseado em **uPlot** que pode ser usado em qualquer módulo da aplicação. Todos os gráficos compartilham as mesmas funcionalidades:

### 8.1. Funcionalidades Incluídas

Todos os gráficos criados com o sistema têm automaticamente:

- ✅ **Zoom**: Scroll do mouse sobre o gráfico
- ✅ **Pan**: Shift+Drag ou Botão do meio do mouse
- ✅ **Seleção de Área**: Drag para selecionar área e dar zoom
- ✅ **Tooltip Hover**: Mostra valores ao passar o mouse
- ✅ **Auto-resize**: Gráfico se ajusta ao container automaticamente
- ✅ **Temas**: Suporta dark/light mode
- ✅ **Performance**: Renderização otimizada com uPlot

### 8.2. Arquitetura do Sistema

```
┌─────────────────────────────────────────────────────────┐
│                  Sistema de Gráficos                     │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  ┌────────────────┐         ┌─────────────────────┐    │
│  │   ChartBuilder │         │   PlotlyChart.svelte│    │
│  │  (Utility)     │────────▶│   (Componente)      │    │
│  │                │         │                     │    │
│  │  Cria config.  │         │  Renderiza uPlot    │    │
│  │  com API fluente│        │  + Interatividade   │    │
│  └────────────────┘         └─────────────────────┘    │
│         ▲                            ▲                  │
│         │                            │                  │
│   ┌─────┴──────┐              ┌─────┴──────┐          │
│   │ ChartSeries│              │ChartConfig │          │
│   │  (Tipos)   │              │  (Tipos)   │          │
│   └────────────┘              └────────────┘          │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

### 8.3. Componentes do Sistema

#### **PlotlyChart.svelte** — Componente de Renderização

Componente Svelte que renderiza gráficos uPlot. Aceita:

**Props:**
- `series: ChartSeries[]` — Array de séries de dados
- `config: ChartConfig` — Configuração do gráfico
- `theme: 'dark' | 'light'` — Tema visual
- `onRangeChange?: (xMin, xMax) => void` — Callback para mudanças de zoom/pan

**Localização:** `lib/components/charts/PlotlyChart.svelte`

**Exemplo de uso direto (não recomendado, use ChartBuilder):**

```svelte
<script>
  import PlotlyChart from '$lib/components/charts/PlotlyChart.svelte';
  
  const series = [
    { key: 'temp', label: 'Temperature', color: '#3b82f6', visible: true, data, dataKey: 'temp', type: 'line' }
  ];
  
  const config = {
    yMin: 0, yMax: 100, yMode: 'manual',
    xMode: 'auto', windowSize: 30, showGrid: true, showHover: true
  };
</script>

<PlotlyChart {series} {config} theme="dark" />
```

#### **ChartBuilder** — Classe Utilitária

Classe com **API fluente** para construir configurações de gráfico facilmente.

**Localização:** `lib/utils/chartBuilder.ts`

**Métodos principais:**

| Método | Descrição |
|--------|-----------|
| `addLineSeries(key, data, dataKey, label, color, options?)` | Adiciona linha normal |
| `addStepSeries(key, data, dataKey, label, color, options?)` | Adiciona linha em degrau (para setpoints) |
| `addAreaSeries(key, data, dataKey, label, color, options?)` | Adiciona área preenchida |
| `setYAxis(mode, min?, max?)` | Configura eixo Y: 'auto' ou 'manual' |
| `setXAxis(mode, windowSize?, min?, max?)` | Configura eixo X: 'auto', 'sliding', 'manual' |
| `enableGrid()` / `disableGrid()` | Liga/desliga grid |
| `enableHover()` / `disableHover()` | Liga/desliga tooltip |
| `toggleSeries(key, visible)` | Mostra/esconde série |
| `setSeriesColor(key, color)` | Altera cor de série |
| `build()` | Retorna `{ series, config }` |

**Métodos estáticos (presets):**

| Método | Descrição |
|--------|-----------|
| `ChartBuilder.createRealtimeConfig(data, colors, visible)` | Cria config para gráfico PV/SP/MV |
| `ChartBuilder.createAnalyzerConfig(...)` | Cria config para gráfico Sensor/Target/Actuator |
| `ChartBuilder.from(series, config)` | Clona configuração existente |

### 8.4. Como Criar um Gráfico (Passo a Passo)

#### **Método 1: Usando ChartBuilder (Recomendado)**

```typescript
import { createChart } from '$lib/utils/chartBuilder';
import PlotlyChart from '$lib/components/charts/PlotlyChart.svelte';

// 1. Preparar dados
const data = [
  { time: 0, temperature: 25.3, setpoint: 25.0 },
  { time: 1, temperature: 25.8, setpoint: 25.0 },
  { time: 2, temperature: 26.1, setpoint: 30.0 },
];

// 2. Criar configuração com ChartBuilder
const { series, config } = createChart()
  .addLineSeries('temp', data, 'temperature', 'Temperatura', '#3b82f6')
  .addStepSeries('sp', data, 'setpoint', 'Setpoint', '#f59e0b', { dashed: true })
  .setYAxis('auto')
  .setXAxis('auto')
  .enableGrid()
  .enableHover()
  .build();

// 3. Usar no componente Svelte
<PlotlyChart {series} {config} theme="dark" />
```

#### **Método 2: Usando Presets**

```typescript
import { ChartPresets } from '$lib/utils/chartBuilder';

// Para gráfico de tempo real (PV/SP/MV)
const { pvsp, mv } = ChartPresets.realtime(
  plantData,
  { pv: '#3b82f6', sp: '#f59e0b', mv: '#10b981' },
  { pv: true, sp: true, mv: true }
);

// Renderizar dois gráficos:
<PlotlyChart series={pvsp.series} config={pvsp.config} theme="dark" />
<PlotlyChart series={mv.series} config={mv.config} theme="dark" />
```

### 8.5. Configuração de Eixos

#### **Eixo Y:**

```typescript
// Auto: calcula min/max automaticamente dos dados
.setYAxis('auto')

// Manual: valores fixos
.setYAxis('manual', 0, 100)
```

#### **Eixo X:**

```typescript
// Auto: mostra todos os dados
.setXAxis('auto')

// Sliding: janela móvel (últimos N segundos)
.setXAxis('sliding', 30)  // últimos 30 segundos

// Manual: range fixo
.setXAxis('manual', 30, 0, 60)  // de 0 a 60 segundos
```

### 8.6. Controle de Visibilidade

```typescript
const builder = createChart()
  .addLineSeries('line1', data, 'value1', 'Line 1', '#3b82f6')
  .addLineSeries('line2', data, 'value2', 'Line 2', '#10b981');

// Esconder line2
builder.toggleSeries('line2', false);

// O build() retorna a config atualizada
const { series, config } = builder.build();
```

### 8.7. Zoom e Pan Programáticos

Para controlar zoom/pan programaticamente, use `onRangeChange`:

```svelte
<script>
  let xMin = $state(0);
  let xMax = $state(30);
  
  function handleRangeChange(newMin: number, newMax: number) {
    xMin = newMin;
    xMax = newMax;
  }
  
  // Atualize config quando xMin/xMax mudarem
  $effect(() => {
    config.xMode = 'manual';
    config.xMin = xMin;
    config.xMax = xMax;
  });
</script>

<PlotlyChart {series} {config} theme="dark" onRangeChange={handleRangeChange} />

<!-- Botões de controle -->
<button onclick={() => { xMin = 0; xMax = 30; }}>Reset Zoom</button>
```

### 8.8. Tipos do Sistema

#### **ChartDataPoint**

```typescript
interface ChartDataPoint {
  time: number;              // Eixo X (em segundos)
  [key: string]: number;     // Valores das séries
}

// Exemplo:
const point: ChartDataPoint = {
  time: 10.5,
  temperature: 25.3,
  pressure: 101.3,
  flow: 42.7
};
```

#### **ChartSeries**

```typescript
interface ChartSeries {
  key: string;               // Identificador único
  label: string;             // Nome no tooltip
  color: string;             // Cor hex: '#3b82f6'
  visible: boolean;          // Se está visível
  data: ChartDataPoint[];    // Array de dados
  dataKey: string;           // Chave do valor: 'temperature'
  type?: 'line' | 'step' | 'area';  // Tipo de visualização
  strokeWidth?: number;      // Espessura da linha
  dashed?: boolean;          // Se é tracejada
}
```

#### **ChartConfig**

```typescript
interface ChartConfig {
  yMin: number;              // Mínimo do eixo Y
  yMax: number;              // Máximo do eixo Y
  yMode: 'auto' | 'manual';  // Modo de escala Y
  xMode: 'auto' | 'sliding' | 'manual';  // Modo de escala X
  windowSize: number;        // Tamanho da janela (segundos) para 'sliding'
  xMin?: number | null;      // Mínimo do eixo X (manual)
  xMax?: number | null;      // Máximo do eixo X (manual)
  showGrid: boolean;         // Mostrar grid
  showHover?: boolean;       // Mostrar tooltip
}
```

### 8.9. Exemplos Práticos

#### **Exemplo 1: Gráfico de Temperatura Simples**

```typescript
const temperatureData = [
  { time: 0, temp: 20 },
  { time: 1, temp: 22 },
  { time: 2, temp: 25 },
  { time: 3, temp: 24 },
];

const { series, config } = createChart()
  .addLineSeries('temp', temperatureData, 'temp', 'Temperatura °C', '#ef4444')
  .setYAxis('auto')
  .setXAxis('auto')
  .enableGrid()
  .build();
```

#### **Exemplo 2: Múltiplas Variáveis com Janela Móvel**

```typescript
const sensorData = [
  { time: 0, temp: 25, humidity: 60, pressure: 1013 },
  { time: 1, temp: 26, humidity: 62, pressure: 1012 },
  // ... mais dados
];

const { series, config } = createChart()
  .addLineSeries('temp', sensorData, 'temp', 'Temperatura', '#ef4444')
  .addLineSeries('humidity', sensorData, 'humidity', 'Umidade', '#3b82f6')
  .addLineSeries('pressure', sensorData, 'pressure', 'Pressão', '#10b981')
  .setYAxis('auto')  // Cada série pode ter sua própria escala
  .setXAxis('sliding', 60)  // Últimos 60 segundos
  .enableGrid()
  .enableHover()
  .build();
```

#### **Exemplo 3: Gráfico com Setpoint (Degrau)**

```typescript
const controlData = [
  { time: 0, pv: 20, sp: 25 },
  { time: 5, pv: 23, sp: 25 },
  { time: 10, pv: 25, sp: 30 },  // Setpoint muda instantaneamente
  { time: 15, pv: 28, sp: 30 },
];

const { series, config } = createChart()
  .addLineSeries('pv', controlData, 'pv', 'Variável de Processo', '#3b82f6', { strokeWidth: 2 })
  .addStepSeries('sp', controlData, 'sp', 'Setpoint', '#f59e0b', { dashed: true })
  .setYAxis('manual', 0, 100)
  .setXAxis('auto')
  .build();
```

#### **Exemplo 4: Área Preenchida (Output de Controlador)**

```typescript
const outputData = [
  { time: 0, output: 50 },
  { time: 1, output: 60 },
  { time: 2, output: 55 },
];

const { series, config } = createChart()
  .addAreaSeries('output', outputData, 'output', 'Saída do Controlador', '#10b981')
  .setYAxis('manual', 0, 100)
  .setXAxis('auto')
  .build();
```

### 8.10. Integração com Módulos

#### **PlotterModule (Tempo Real)**

O PlotterModule já usa o sistema de gráficos. Para customizar:

```typescript
// Em PlotterModule.svelte

// Séries PV/SP
const pvSpSeries = $derived([
  {
    key: 'pv',
    label: 'PV (Process Variable)',
    color: lineColors.pv,
    visible: chartState.visible.pv,
    data: plantData,
    dataKey: 'pv' as const,
    type: 'line' as const,
    strokeWidth: 2
  },
  {
    key: 'sp',
    label: 'SP (Setpoint)',
    color: lineColors.sp,
    visible: chartState.visible.sp,
    data: plantData,
    dataKey: 'sp' as const,
    type: 'step' as const,
    strokeWidth: 1.5,
    dashed: true
  }
]);
```

#### **AnalyzerModule (Dados Históricos)**

O AnalyzerModule usa os presets para criar gráficos:

```typescript
// Em VariableChart.svelte
import { ChartPresets } from '$lib/utils/chartBuilder';

const charts = $derived(
  ChartPresets.analyzer(
    sensorData,
    targetData,
    actuatorData,
    processedData.sensorRange,
    processedData.actuatorRange
  )
);

// Renderizar:
<PlotlyChart series={charts.sensor.series} config={charts.sensor.config} theme={theme} />
<PlotlyChart series={charts.actuator.series} config={charts.actuator.config} theme={theme} />
```

### 8.11. Boas Práticas

1. **Use ChartBuilder**: Sempre prefira `ChartBuilder` ao invés de criar `series` e `config` manualmente
2. **Use Presets**: Para casos comuns (tempo real, análise), use `ChartPresets`
3. **Keys únicas**: Cada série precisa de um `key` único
4. **dataKey correto**: Certifique-se que `dataKey` existe nos dados
5. **Performance**: Limite o buffer de dados (ex: max 50.000 pontos)
6. **Cores consistentes**: Use as cores padrão ou defina um tema consistente
7. **Tipos TypeScript**: Sempre use `ChartDataPoint`, `ChartSeries`, `ChartConfig`

### 8.12. Troubleshooting

**Problema: Gráfico não aparece**
- ✓ Container tem altura definida? (`h-full`, `min-h-0`, ou `height: 300px`)
- ✓ Dados têm propriedade `time`?
- ✓ `dataKey` corresponde às propriedades dos dados?

**Problema: Zoom não funciona**
- ✓ Configurou `onRangeChange`?
- ✓ Está atualizando `config.xMin` e `config.xMax`?
- ✓ `config.xMode` está em 'manual' quando definir range?

**Problema: Cores não aparecem no dark mode**
- ✓ Passou prop `theme` corretamente?
- ✓ Cores são válidas (hex: '#3b82f6')?

**Problema: Performance ruim com muitos pontos**
- ✓ Limite o buffer de dados (splice quando > 50k pontos)
- ✓ Use `xMode: 'sliding'` ao invés de 'auto' para janela móvel

---

## 9. Como Fazer: Receitas Comuns

### 9.1. Adicionar um Novo Módulo na Sidebar

**Exemplo:** Adicionar um módulo "Histórico" com ícone `Clock`.

**Passo 1 — Registrar a chave do módulo em `types/ui.ts`:**

```typescript
// ANTES:
export type TabKey = 'plotter';
export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' }
} as const;

// DEPOIS:
export type TabKey = 'plotter' | 'history';
export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' },
  history: { label: 'Histórico', icon: 'Clock' }
} as const;
```

**Passo 2 — Criar o componente do módulo em `components/modules/HistoryModule.svelte`:**

```svelte
<script lang="ts">
  let { theme }: { theme: 'dark' | 'light' } = $props();
</script>

<div class="p-8 h-full flex flex-col bg-slate-50 dark:bg-zinc-950">
  <header class="mb-6 pb-4 border-b border-slate-200 dark:border-white/5">
    <h2 class="text-2xl font-bold text-slate-800 dark:text-zinc-100">Histórico</h2>
  </header>
  <div class="flex-1 flex items-center justify-center text-slate-400">
    Conteúdo do módulo aqui
  </div>
</div>
```

**Passo 3 — Adicionar o botão na `Sidebar.svelte`:**

```svelte
<script lang="ts">
  // Adicionar o import do ícone:
  import { TrendingUp, Clock, Sun, Moon, Settings as SettingsIcon } from 'lucide-svelte';
</script>

<!-- No <nav>, adicionar após o botão de plotter: -->
<SidebarBtn
  icon={Clock}
  label={MODULE_TABS.history.label}
  active={activeModule === 'history'}
  collapsed={sidebarCollapsed}
  onclick={() => appStore.setActiveModule('history')}
/>
```

**Passo 4 — Renderizar o módulo no `+page.svelte`:**

```svelte
<script lang="ts">
  // Adicionar import:
  import HistoryModule from '$lib/components/modules/HistoryModule.svelte';
</script>

<!-- No bloco {#if}, adicionar: -->
{:else if appStore.state.activeModule === 'history'}
  <HistoryModule theme={appStore.state.theme || 'dark'} />
```

**Pronto.** São 4 arquivos tocados: `ui.ts`, novo `HistoryModule.svelte`, `Sidebar.svelte`, `+page.svelte`.

---

### 9.2. Enviar Dados para Plotagem

O sistema de plotagem é desacoplado da fonte de dados. Qualquer fonte (simulação, serial, WebSocket, arquivo) só precisa fazer `push()` no buffer plain.

**Passo 1 — Importar o buffer:**

```typescript
import { getPlantData, setPlantStats } from '$lib/stores/plantData';
import type { PlantDataPoint } from '$lib/types/plant';
```

**Passo 2 — Enviar um ponto de dado:**

```typescript
const data = getPlantData(plantId);
data.push({
  time: elapsedSeconds,   // tempo em segundos (eixo X)
  pv: sensorValue,        // variável de processo (ex: temperatura lida)
  sp: setpointValue,      // setpoint desejado
  mv: controllerOutput,   // saída do controlador (0-100%)
});
```

**O que acontece automaticamente:**
- `PlotlyChart` detecta `data.length` mudou a cada 33ms (~30fps) e chama `setData()`
- `PlotterModule` atualiza `currentPV`, `currentMV` e stats via `_displayTick` (33ms)
- O gráfico renderiza o novo ponto imediatamente no próximo frame

**Passo 3 (opcional) — Limitar tamanho do buffer:**

```typescript
const MAX_POINTS = 50000;
if (data.length > MAX_POINTS) {
  data.splice(0, data.length - MAX_POINTS);
}
```

**Passo 4 (opcional) — Atualizar estatísticas:**

```typescript
import { setPlantStats } from '$lib/stores/plantData';
setPlantStats(plantId, {
  errorAvg: calculatedError,
  stability: stabilityIndex,
  uptime: uptimeSeconds,
});
```

---

### 9.4. Alterar a Taxa de Atualização do Plot

A taxa de renderização do gráfico é controlada por **dois timers independentes**:

| Timer | Arquivo | Variável | Controla |
|-------|---------|----------|----------|
| Render timer | `PlotlyChart.svelte` | `renderTimer` | Frequência de `setData()` do uPlot |
| Display tick | `PlotterModule.svelte` | `_displayTimer` | Atualização dos valores numéricos (PV, MV, stats) na toolbar |

**Para mudar a taxa de plotagem (ambos devem usar o mesmo intervalo):**

```typescript
// PlotlyChart.svelte — onMount
renderTimer = setInterval(() => {
  const n = series.length > 0 ? series[0].data.length : 0;
  if (n !== prevLen) {
    prevLen = n;
    updateChart();
  }
}, 33);  // 33ms ≈ 30fps | 16ms ≈ 60fps | 100ms ≈ 10fps

// PlotterModule.svelte — onMount
_displayTimer = setInterval(() => _displayTick++, 33);  // deve acompanhar o render timer
```

**Valores recomendados:**

| FPS | Intervalo (ms) | Uso recomendado |
|-----|---------------|------------------|
| 60  | 16            | Animações ultra-suaves, alto consumo de CPU |
| 30  | 33            | Padrão — bom equilíbrio entre fluidez e performance |
| 10  | 100           | Dados lentos, economia de CPU |
| 5   | 200           | Dados muito lentos, mínimo consumo |

> **Nota:** O timer só chama `updateChart()` se houver dados novos (`n !== prevLen`), então CPU idle é zero mesmo com o timer rodando.

### 9.3. Remover um Módulo Existente

**Exemplo:** Remover um módulo customizado "History".

**Passo 1 — Remover de `types/ui.ts`:**

```typescript
// Remover 'history' do TabKey e MODULE_TABS:
export type TabKey = 'plotter';
export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' }
} as const;
```

**Passo 2 — Remover da `Sidebar.svelte`:**

```svelte
<!-- Remover o bloco do botão: -->
<SidebarBtn
  icon={Clock}
  label={MODULE_TABS.history.label}
  active={activeModule === 'history'}
  collapsed={sidebarCollapsed}
  onclick={() => appStore.setActiveModule('history')}
/>

<!-- E remover o import Clock se não for mais usado -->
```

**Passo 3 — Remover do `+page.svelte`:**

```svelte
<!-- Remover import: -->
import HistoryModule from '$lib/components/modules/HistoryModule.svelte';

<!-- Remover bloco condicional: -->
{:else if appStore.state.activeModule === 'history'}
  <HistoryModule theme={appStore.state.theme || 'dark'} />
```

**Passo 4 — Deletar o arquivo:**

```bash
rm src/lib/components/modules/HistoryModule.svelte
```

**Passo 5 — Garantir que o `activeModule` padrão no `data.svelte.ts` não aponte para o módulo removido** (já é `'plotter'`, então OK).

---

### 9.5. Integrar Dados Reais do Backend (Tauri)

Atualmente o app usa simulação local (`services/simulation.ts`). Para conectar ao backend real via Tauri:

**Passo 1 — Criar o serviço de comunicação em `services/serial.ts`:**

```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getPlantData, setPlantStats } from '$lib/stores/plantData';
import type { PlantDataPoint, PlantStats } from '$lib/types/plant';

/** Inicia escuta de dados seriais do backend Rust. Retorna unsubscribe. */
export async function startSerialListener(plantId: string): Promise<() => void> {
  const unlisten = await listen<{ time: number; pv: number; sp: number; mv: number }>(
    `serial-data-${plantId}`,
    (event) => {
      const { time, pv, sp, mv } = event.payload;
      const data = getPlantData(plantId);
      data.push({ time, pv, sp, mv });

      // Opcional: limitar tamanho do buffer
      if (data.length > 50000) {
        data.splice(0, data.length - 50000);
      }
    }
  );
  return unlisten;
}

/** Conecta/desconecta porta serial via comando Tauri */
export async function connectSerial(port: string, baudRate: number): Promise<void> {
  await invoke('connect_serial', { port, baudRate });
}

export async function disconnectSerial(): Promise<void> {
  await invoke('disconnect_serial');
}
```

**Passo 2 — Substituir a simulação no `+page.svelte`:**

```svelte
<script lang="ts">
  // REMOVER:
  import { startSimulation } from '$lib/services/simulation';

  // ADICIONAR:
  import { startSerialListener } from '$lib/services/serial';

  // SUBSTITUIR o $effect da simulação:
  $effect(() => {
    if (typeof window === 'undefined') return;
    const cleanups: (() => void)[] = [];

    // Para cada planta conectada, escutar eventos do backend
    appStore.state.plants.forEach(async (plant) => {
      if (plant.connected) {
        const unlisten = await startSerialListener(plant.id);
        cleanups.push(unlisten);
      }
    });

    return () => cleanups.forEach(fn => fn());
  });
</script>
```

**Passo 3 — No lado Rust (src-tauri/src/), emitir eventos:**

```rust
// O backend Rust precisa emitir eventos no formato:
app_handle.emit(&format!("serial-data-{}", plant_id), SerialPayload {
    time: elapsed_seconds,
    pv: sensor_value,
    sp: setpoint,
    mv: controller_output,
})?;
```

**O que NÃO precisa mudar:**
- `PlotterModule` — já lê de `getPlantData()`, continua igual
- `PlotlyChart` — já recebe series/config via props, continua igual
- `plantData.ts` — já é o buffer, continua igual
- Toda a UI de gráficos, toolbar, etc. — zero mudanças

**Resumo do acoplamento:** Apenas `+page.svelte` e o novo `serial.ts` mudam. Todo o resto é desacoplado dos dados.

---

### 9.6. Adicionar uma Nova Variável ao Gráfico

**Exemplo:** Adicionar variável "Temperatura" (`temp`) ao gráfico superior.

**Passo 1 — Estender `PlantDataPoint` em `types/plant.ts`:**

```typescript
export interface PlantDataPoint {
  time: number;
  sp: number;
  pv: number;
  mv: number;
  temp?: number;  // Nova variável
  [key: string]: number;
}
```

**Passo 2 — Adicionar visibilidade/cor em `types/chart.ts`:**

```typescript
export interface ChartStateType {
  // ... campos existentes ...
  visible: { pv: boolean; sp: boolean; mv: boolean; temp: boolean };
}

export function defaultChartState(): ChartStateType {
  return {
    // ... campos existentes ...
    visible: { pv: true, sp: true, mv: true, temp: true },
  };
}

export interface LineColors {
  pv: string; sp: string; mv: string; temp: string;
}

export const DEFAULT_LINE_COLORS: Readonly<LineColors> = Object.freeze({
  pv: '#3b82f6', sp: '#f59e0b', mv: '#10b981', temp: '#ef4444',
});
```

**Passo 3 — Adicionar a série no `PlotterModule.svelte` (no bloco `pvSpSeries`):**

```typescript
const pvSpSeries = $derived([
  // ... PV e SP existentes ...
  {
    key: 'temp',
    label: 'Temperatura',
    color: lineColors.temp,
    visible: chartState.visible.temp,
    data: plantData,
    dataKey: 'temp' as const,
    type: 'line' as const,
    strokeWidth: 1.5
  }
]);
```

**Passo 4 — Adicionar toggle no `ChartContextMenu.svelte`** (copiar um bloco existente de PV/SP/MV e substituir `pv` → `temp`).

**Passo 5 — Garantir que os dados incluam `temp`** no push (simulação ou backend).

---

### 9.7. Criar um Novo Tipo de Controlador

**Exemplo:** Adicionar controlador tipo "Fuzzy".

**Passo 1 — Registrar o tipo em `types/controller.ts`:**

```typescript
export type ControllerType = 'PID' | 'Flow' | 'Level' | 'Fuzzy';
```

**Passo 2 — Criar a interface de parâmetros:**

```typescript
export interface FuzzyParams {
  sensitivity: ControllerParam;
  ruleSet: ControllerParam;
  outputScale: ControllerParam;
}
```

**Passo 3 — Atualizar a union de params no `Controller`:**

```typescript
export interface Controller {
  // ...
  params: PIDParams | FuzzyParams | Record<string, ControllerParam>;
}
```

**Passo 4 — No `PlotterModule.svelte`, atualizar `addController()` para permitir escolher o tipo** (ou manter como default PID e adicionar um select no `ControllerPanel`).

**Passo 5 — Implementar a lógica do controlador** no serviço de simulação (`services/simulation.ts`) ou no backend Rust.

---

### 9.8. Usar o Módulo Analyzer

O módulo Analyzer permite analisar dados históricos de múltiplas variáveis a partir de arquivos CSV. **Importante:** Todo processamento é feito pelo backend (atualmente mockado).

**Arquitetura:**

```
Upload CSV → Backend Mockado → Dados Processados → UI Renderiza
            (analyzerBackend.ts)
            - Valida formato
            - Extrai variáveis
            - Calcula ranges
            - Otimiza dados
```

**Formato do CSV:**

```csv
seconds, sensor_0, actuator_0, target_0, sensor_1, actuator_1, target_1
0.0, 25.3, 12.5, 20.0, 30.1, 8.2, 28.0
0.1, 25.5, 13.1, 20.0, 30.3, 8.5, 28.0
0.2, 25.8, 14.2, 20.0, 30.5, 9.1, 28.0
```

**Regras:**
- Primeira coluna deve ser `seconds` (tempo em segundos)
- Cada variável é um grupo de 3 colunas: `sensor_N`, `actuator_N`, `target_N`
- N deve ser sequencial (0, 1, 2, ...)
- Todas as 3 colunas do grupo devem estar presentes

**Uso:**

1. **Carregar CSV:** Clique em "Carregar CSV" no header do Analyzer
2. **Selecionar variáveis:** Use o painel lateral para marcar as variáveis desejadas
3. **Visualizar:** As variáveis selecionadas aparecem em grid, cada uma com:
   - Gráfico superior: sensor (azul) + target (laranja tracejado)
   - Gráfico inferior: actuator (verde)
4. **Layout responsivo:** O grid se ajusta automaticamente ao número de variáveis

**Exemplo de criação de CSV via Python:**

```python
import pandas as pd
import numpy as np

# Simular 3 variáveis por 100 segundos
time = np.arange(0, 100, 0.1)
data = {
    'seconds': time,
    'sensor_0': 20 + 5 * np.sin(time * 0.1) + np.random.randn(len(time)),
    'actuator_0': 50 + 10 * np.cos(time * 0.05),
    'target_0': np.full(len(time), 20),
    'sensor_1': 30 + 8 * np.sin(time * 0.15) + np.random.randn(len(time)),
    'actuator_1': 60 + 15 * np.cos(time * 0.08),
    'target_1': np.full(len(time), 30),
}

df = pd.DataFrame(data)
df.to_csv('plant_data.csv', index=False)
```

---

## Apêndice: Interações entre Camadas

```
┌─────────────────────────────────────────────────────────────────┐
│                         TIPOS (types/)                         │
│  app.ts  chart.ts  controller.ts  plant.ts  ui.ts              │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │   STORES     │     │  SERVICES    │     │    UTILS     │    │
│  │ data.svelte  │     │ simulation   │     │  format.ts   │    │
│  │ plantData    │     │ export       │     │              │    │
│  └──────┬───────┘     └──────┬───────┘     └──────┬───────┘    │
│         │                    │                    │             │
│         ├────────────────────┼────────────────────┘             │
│         ▼                    ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    COMPONENTS                            │   │
│  │                                                          │   │
│  │  +page.svelte (raiz)                                     │   │
│  │    ├── Sidebar ← MODULE_TABS, appStore                   │   │
│  │    ├── PlotterModule ← appStore, plantData               │   │
│  │    │     ├── PlotlyChart ← series, config (genérico)     │   │
│  │    │     ├── PlotterToolbar ← plant, callbacks           │   │
│  │    │     ├── ChartContextMenu ← chartState, lineColors   │   │
│  │    │     ├── ControllerPanel ← plant, callbacks          │   │
│  │    │     └── PlantRemovalModal ← visible, callbacks      │   │
│  │    ├── PoleAnalysisModule ← theme                        │   │
│  │    └── GlobalSettingsModal ← showGlobalSettings          │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

**Regras de dependência:**
- `types/` → nada (folha)
- `utils/` → `types/` apenas
- `stores/` → `types/` apenas
- `services/` → `types/` + `stores/`
- `components/` → tudo acima
- Componentes filhos **nunca** importam `appStore` diretamente (exceto `Sidebar` e `GlobalSettingsModal` que são de layout global) — recebem dados via props e emitem via callbacks
