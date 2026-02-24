# Senamby Desktop — Arquitetura & Guia do Desenvolvedor

> **Stack:** Tauri 2.0 · SvelteKit 2.0 · Svelte 5 (runes) · TypeScript · uPlot · Tailwind CSS  
> **Última atualização:** Fevereiro 2026

---

## Sumário

1. [Visão Geral](#1-visão-geral)
2. [Estrutura de Pastas](#2-estrutura-de-pastas)
3. [Arquitetura de Estado](#3-arquitetura-de-estado)
4. [Árvore de Componentes](#4-árvore-de-componentes)
5. [Fluxo de Dados](#5-fluxo-de-dados)
6. [Detalhamento dos Componentes](#6-detalhamento-dos-componentes)
7. [Sistema de Tipos](#7-sistema-de-tipos)
8. [Como Fazer: Receitas Comuns](#8-como-fazer-receitas-comuns)
   - [Adicionar um módulo na sidebar](#81-adicionar-um-novo-módulo-na-sidebar)
   - [Remover um módulo existente](#82-remover-um-módulo-existente)
   - [Integrar dados reais do backend](#83-integrar-dados-reais-do-backend-tauri)
   - [Adicionar uma nova variável ao gráfico](#84-adicionar-uma-nova-variável-ao-gráfico)
   - [Criar um novo tipo de controlador](#85-criar-um-novo-tipo-de-controlador)

---

## 1. Visão Geral

O Senamby Desktop é uma aplicação SCADA (Supervisory Control and Data Acquisition) que roda como app desktop via Tauri. A interface permite monitorar plantas industriais em tempo real com gráficos de tendência, controladores PID configuráveis, e análise de lugar das raízes (LGR).

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
│  │  │      │  │  OU: PoleAnalysisModule          │   │  │
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
│   │   │   └── PlantRemovalModal.svelte     # Confirmação de remoção de planta
│   │   │
│   │   ├── modules/                 # Módulos de feature (telas principais)
│   │   │   ├── PlotterModule.svelte         # Tela de tendências (principal)
│   │   │   ├── PoleAnalysisModule.svelte    # Tela de Lugar das Raízes
│   │   │   └── PlaceholderModule.svelte     # Placeholder genérico
│   │   │
│   │   ├── plotter/                 # Sub-componentes exclusivos do Plotter
│   │   │   ├── PlantTabs.svelte             # Abas das plantas
│   │   │   ├── PlotterToolbar.svelte        # Toolbar (conectar, pausar, exportar)
│   │   │   ├── ChartContextMenu.svelte      # Menu de contexto (botão direito)
│   │   │   └── ControllerPanel.svelte       # Painel lateral de controladores
│   │   │
│   │   └── ui/                      # Primitivas reutilizáveis
│   │       ├── Badge.svelte
│   │       ├── Button.svelte
│   │       ├── DynamicParamInput.svelte
│   │       ├── Modal.svelte
│   │       └── SimpleToggle.svelte
│   │
│   ├── services/                    # Lógica de negócio (sem UI)
│   │   ├── simulation.ts           # Simulação PID + transferência térmica
│   │   └── export.ts               # Exportação CSV
│   │
│   ├── stores/                      # Estado global
│   │   ├── data.svelte.ts           # AppStore (estado reativo com $state)
│   │   └── plantData.ts             # Buffer de dados plain (Map, sem reatividade)
│   │
│   ├── types/                       # Interfaces e tipos TypeScript
│   │   ├── app.ts                   # AppState
│   │   ├── chart.ts                 # ChartConfig, ChartSeries, ChartStateType
│   │   ├── controller.ts            # Controller, ControllerParam, PIDParams
│   │   ├── plant.ts                 # Plant, PlantDataPoint, PlantStats
│   │   └── ui.ts                    # TabKey, MODULE_TABS
│   │
│   └── utils/                       # Funções puras utilitárias
│       └── format.ts                # formatTime, generateId
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
  activeModule: TabKey;            // Módulo ativo ('plotter' | 'poles')
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

**Por que plain Map?** Dados de série temporal a 10Hz gerariam milhares de invalidações reativas por segundo. O array plain é mutado via `.push()` e o gráfico lê diretamente via `setInterval(200ms)` — sem overhead de reatividade.

### 3.3 Como a UI lê dados não-reativos

O `PlotterModule` usa um **tick de display** para forçar re-leitura periódica:

```typescript
let _displayTick = $state(0);
onMount(() => { _displayTimer = setInterval(() => _displayTick++, 200); });

const currentPV = $derived.by(() => {
  _displayTick; // Toca a reatividade a cada 200ms
  const data = getPlantData(activePlantId);
  return data.length > 0 ? data[data.length - 1].pv : 0;
});
```

O `PlotlyChart` (uPlot) usa seu próprio `setInterval(200ms)` interno para checar se há dados novos e chamar `chart.setData()`.

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
├── PoleAnalysisModule           ← quando activeModule === 'poles'
│
└── GlobalSettingsModal              Preferências gerais (gridlines, etc.)
```

**Fluxo de props:** Top-down. O `+page.svelte` passa `appStore.state.*` como props. Mutações sobem via callbacks (`onToggleConnect`, `onExport`, etc.) que chamam `appStore.*()`.

---

## 5. Fluxo de Dados

### 5.1 Simulação → Gráfico (atual, com simulação local)

```
┌──────────────┐    push()     ┌──────────────┐  setInterval   ┌──────────────┐
│  simulation  │──────────────→│  plantData   │───(200ms)─────→│  PlotlyChart │
│   .ts        │               │  Map<id,[]>  │                │   (uPlot)    │
│  (10Hz)      │               │  plain array │                │  setData()   │
└──────────────┘               └──────────────┘                └──────────────┘
       ↑                              │
       │ lê plant config              │ _displayTick (200ms)
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

**Papel:** Navegação entre módulos. Mapeia `MODULE_TABS` em botões `SidebarBtn`.
- Colapsável (16px ↔ 256px) com `appStore.toggleSidebar()`
- Seção inferior: toggle tema + ajustes

### 6.5 `ControllerPanel.svelte` (125 linhas)

**Papel:** Painel lateral deslizante. Permite configurar:
- Setpoint (slider 0–100%)
- Controladores (add, remove, toggle, editar parâmetros via `DynamicParamInput`)

### 6.6 `ChartContextMenu.svelte` (115 linhas)

**Papel:** Menu de contexto (botão direito na área do chart). Permite:
- Alternar modo do eixo X (Auto/Janela/Manual)
- Alternar modo do eixo Y (Auto/Manual) com campos min/max
- Toggle visibilidade e cor de cada variável (PV/SP/MV)

### 6.7 `PoleAnalysisModule.svelte` (76 linhas)

**Papel:** Tela de análise de lugar das raízes. Renderiza SVG estático com polos e zeros plotados no plano complexo.

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

### `types/ui.ts`

```typescript
TabKey = 'plotter' | 'poles'
MODULE_TABS = { plotter: { label, icon }, poles: { label, icon } }
```

---

## 8. Como Fazer: Receitas Comuns

### 8.1. Adicionar um Novo Módulo na Sidebar

**Exemplo:** Adicionar um módulo "Histórico" com ícone `Clock`.

**Passo 1 — Registrar a chave do módulo em `types/ui.ts`:**

```typescript
// ANTES:
export type TabKey = 'plotter' | 'poles';
export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' },
  poles: { label: 'Polos', icon: 'Activity' }
} as const;

// DEPOIS:
export type TabKey = 'plotter' | 'poles' | 'history';
export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' },
  poles: { label: 'Polos', icon: 'Activity' },
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
  import { TrendingUp, Activity, Clock, Sun, Moon, Settings as SettingsIcon } from 'lucide-svelte';
</script>

<!-- No <nav>, adicionar após o botão de poles: -->
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

### 8.2. Remover um Módulo Existente

**Exemplo:** Remover o módulo "Lugar das Raízes" (`poles`).

**Passo 1 — Remover de `types/ui.ts`:**

```typescript
// Remover 'poles' do TabKey e MODULE_TABS:
export type TabKey = 'plotter';
export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' }
} as const;
```

**Passo 2 — Remover da `Sidebar.svelte`:**

```svelte
<!-- Remover este bloco inteiro: -->
<SidebarBtn
  icon={Activity}
  label={MODULE_TABS.poles.label}
  active={activeModule === 'poles'}
  collapsed={sidebarCollapsed}
  onclick={() => appStore.setActiveModule('poles')}
/>

<!-- E remover o import Activity se não for mais usado -->
```

**Passo 3 — Remover do `+page.svelte`:**

```svelte
<!-- Remover import: -->
import PoleAnalysisModule from '$lib/components/modules/PoleAnalysisModule.svelte';

<!-- Remover bloco condicional: -->
{:else if appStore.state.activeModule === 'poles'}
  <PoleAnalysisModule theme={appStore.state.theme || 'dark'} />
```

**Passo 4 — Deletar o arquivo:**

```bash
rm src/lib/components/modules/PoleAnalysisModule.svelte
```

**Passo 5 — Garantir que o `activeModule` padrão no `data.svelte.ts` não aponte para o módulo removido** (já é `'plotter'`, então OK).

---

### 8.3. Integrar Dados Reais do Backend (Tauri)

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

### 8.4. Adicionar uma Nova Variável ao Gráfico

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

### 8.5. Criar um Novo Tipo de Controlador

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
