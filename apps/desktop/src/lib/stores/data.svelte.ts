import type { Plant, PlantVariable } from '$lib/types/plant';
import { createDefaultVariable } from '$lib/types/plant';
import type { Controller, ControllerParam } from '$lib/types/controller';
import type { TabKey } from '$lib/types/ui';
import type { AppState } from '$lib/types/app';

class AppStore {
  state = $state<AppState>({
    theme: 'dark',
    activeModule: 'plotter',
    activePlantId: null,
    sidebarCollapsed: true,
    showGlobalSettings: false,
    showControllerPanel: false,
    plants: []
  });

  setTheme(theme: 'dark' | 'light') {
    this.state.theme = theme;
  }

  toggleTheme() {
    this.state.theme = this.state.theme === 'dark' ? 'light' : 'dark';
  }

  setActiveModule(module: TabKey) {
    this.state.activeModule = module;
  }

  setActivePlantId(id: string) {
    this.state.activePlantId = id;
  }

  setSidebarCollapsed(collapsed: boolean) {
    this.state.sidebarCollapsed = collapsed;
  }

  toggleSidebar() {
    this.state.sidebarCollapsed = !this.state.sidebarCollapsed;
  }

  setShowGlobalSettings(show: boolean) {
    this.state.showGlobalSettings = show;
  }

  setShowControllerPanel(show: boolean) {
    this.state.showControllerPanel = show;
  }

  setPlants(plants: Plant[]) {
    this.state.plants = plants;
    if (!plants.some((plant) => plant.id === this.state.activePlantId)) {
      this.state.activePlantId = plants[0]?.id ?? null;
    }
  }

  addPlant(plant: Plant) {
    this.state.plants = [plant, ...this.state.plants.filter((entry) => entry.id !== plant.id)];
    if (!this.state.activePlantId) {
      this.state.activePlantId = plant.id;
    }
  }

  upsertPlant(plant: Plant) {
    const index = this.state.plants.findIndex((entry) => entry.id === plant.id);
    if (index >= 0) {
      this.state.plants[index] = plant;
    } else {
      this.state.plants.unshift(plant);
    }
  }

  removePlant(plantId: string) {
    const idx = this.state.plants.findIndex(p => p.id === plantId);
    if (idx > -1) {
      this.state.plants.splice(idx, 1);
      if (this.state.activePlantId === plantId) {
        this.state.activePlantId = this.state.plants[0]?.id ?? null;
      }
    }
  }

  updatePlant(plantId: string, updates: Partial<Plant>) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) Object.assign(plant, updates);
  }

  toggleConnect(plantId: string) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) plant.connected = !plant.connected;
  }

  togglePause(plantId: string) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) plant.paused = !plant.paused;
  }

  addController(plantId: string, controller: Omit<Controller, 'id'>) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) {
      plant.controllers.push({
        ...controller,
        id: crypto.randomUUID().substring(0, 9)
      });
    }
  }

  deleteController(plantId: string, controllerId: string) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) {
      const idx = plant.controllers.findIndex(c => c.id === controllerId);
      if (idx > -1) plant.controllers.splice(idx, 1);
    }
  }

  updateControllerMeta(plantId: string, controllerId: string, field: string, value: any) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) {
      const ctrl = plant.controllers.find(c => c.id === controllerId);
      if (ctrl) (ctrl as any)[field] = value;
    }
  }

  updateControllerParam(plantId: string, controllerId: string, paramKey: string, value: any) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) {
      const ctrl = plant.controllers.find(c => c.id === controllerId);
      if (ctrl && ctrl.params) {
        const param = (ctrl.params as Record<string, ControllerParam>)[paramKey];
        if (param) param.value = value;
      }
    }
  }

  updateVariableSetpoint(plantId: string, variableIndex: number, setpoint: number) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant && plant.variables[variableIndex]) {
      plant.variables[variableIndex].setpoint = setpoint;
    }
  }

  addVariable(plantId: string, variable?: Partial<PlantVariable>) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) {
      const index = plant.variables.length;
      plant.variables.push({
        ...createDefaultVariable(index),
        ...variable,
        id: `var_${index}`,
      });
    }
  }

  removeVariable(plantId: string, variableIndex: number) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant && plant.variables.length > 1) {
      plant.variables.splice(variableIndex, 1);
    }
  }
}

export const appStore = new AppStore();
