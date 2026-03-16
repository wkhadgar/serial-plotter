import type { Plant, PlantVariable } from '$lib/types/plant';
import { createDefaultVariable } from '$lib/types/plant';
import { normalizeControllerParamValue, type Controller, type ControllerParam } from '$lib/types/controller';
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

  private findPlant(plantId: string): Plant | undefined {
    return this.state.plants.find((plant) => plant.id === plantId);
  }

  private withPlant<T>(plantId: string, updater: (plant: Plant) => T): T | undefined {
    const plant = this.findPlant(plantId);
    if (!plant) {
      return undefined;
    }

    return updater(plant);
  }

  private findController(plant: Plant, controllerId: string): Controller | undefined {
    return plant.controllers.find((controller) => controller.id === controllerId);
  }

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
    this.withPlant(plantId, (plant) => Object.assign(plant, updates));
  }

  toggleConnect(plantId: string) {
    this.withPlant(plantId, (plant) => {
      plant.connected = !plant.connected;
    });
  }

  togglePause(plantId: string) {
    this.withPlant(plantId, (plant) => {
      plant.paused = !plant.paused;
    });
  }

  addController(plantId: string, controller: Omit<Controller, 'id'>) {
    this.withPlant(plantId, (plant) => {
      plant.controllers.push({
        ...controller,
        id: crypto.randomUUID().substring(0, 9)
      });
    });
  }

  deleteController(plantId: string, controllerId: string) {
    this.withPlant(plantId, (plant) => {
      const idx = plant.controllers.findIndex((controller) => controller.id === controllerId);
      if (idx > -1) plant.controllers.splice(idx, 1);
    });
  }

  updateControllerMeta(plantId: string, controllerId: string, field: string, value: any) {
    this.withPlant(plantId, (plant) => {
      const controller = this.findController(plant, controllerId);
      if (controller) (controller as any)[field] = value;
    });
  }

  updateControllerParam(plantId: string, controllerId: string, paramKey: string, value: any): boolean {
    return this.withPlant(plantId, (plant) => {
      const controller = this.findController(plant, controllerId);
      if (controller?.params) {
        const param = (controller.params as Record<string, ControllerParam>)[paramKey];
        if (!param) return false;

        const normalizedValue = normalizeControllerParamValue(param, value);
        if (normalizedValue === null) {
          return false;
        }

        param.value = normalizedValue;
        return true;
      }

      return false;
    }) ?? false;
  }

  updateVariableSetpoint(plantId: string, variableIndex: number, setpoint: number) {
    this.withPlant(plantId, (plant) => {
      if (plant.variables[variableIndex]) {
        plant.variables[variableIndex].setpoint = setpoint;
      }
    });
  }

  addVariable(plantId: string, variable?: Partial<PlantVariable>) {
    this.withPlant(plantId, (plant) => {
      const index = plant.variables.length;
      plant.variables.push({
        ...createDefaultVariable(index),
        ...variable,
        id: `var_${index}`,
      });
    });
  }

  removeVariable(plantId: string, variableIndex: number) {
    this.withPlant(plantId, (plant) => {
      if (plant.variables.length > 1) {
        plant.variables.splice(variableIndex, 1);
      }
    });
  }
}

export const appStore = new AppStore();
