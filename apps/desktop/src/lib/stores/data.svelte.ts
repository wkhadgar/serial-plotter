import type { Plant, PlantVariable } from '$lib/types/plant';
import { createDefaultVariable } from '$lib/types/plant';
import type { Controller, ControllerParam } from '$lib/types/controller';
import type { TabKey } from '$lib/types/ui';
import type { AppState } from '$lib/types/app';

class AppStore {
  state = $state<AppState>({
    theme: 'dark',
    activeModule: 'plotter',
    activePlantId: 'p1',
    sidebarCollapsed: true,
    showGlobalSettings: false,
    showControllerPanel: false,
    plants: [
      {
        id: 'p1',
        name: 'Tanque Misturador T-200',
        connected: false,
        paused: false,
        variables: [
          {
            id: 'var_0',
            name: 'Temperatura',
            type: 'sensor',
            unit: '°C',
            setpoint: 50,
            pvMin: 0,
            pvMax: 100,
            mvMin: 0,
            mvMax: 100,
          },
          {
            id: 'var_1',
            name: 'Pressão',
            type: 'sensor',
            unit: 'bar',
            setpoint: 2.5,
            pvMin: 0,
            pvMax: 10,
            mvMin: 0,
            mvMax: 100,
          },
          {
            id: 'var_2',
            name: 'Válvula V-101',
            type: 'atuador',
            unit: '%',
            setpoint: 0,
            pvMin: 0,
            pvMax: 100,
            mvMin: 0,
            mvMax: 100,
            linkedSensorIds: ['var_0'],
          },
          {
            id: 'var_3',
            name: 'Bomba B-201',
            type: 'atuador',
            unit: '%',
            setpoint: 0,
            pvMin: 0,
            pvMax: 100,
            mvMin: 0,
            mvMax: 100,
            linkedSensorIds: ['var_1'],
          }
        ],
        stats: { errorAvg: 0, stability: 100, uptime: 0 },
        controllers: [
          {
            id: 'c1',
            name: 'PID Temperatura',
            type: 'PID',
            active: true,
            params: {
              kp: { type: 'number', value: 1.5, label: 'Kp (Prop)' },
              ki: { type: 'number', value: 0.02, label: 'Ki (Int)' },
              kd: { type: 'number', value: 0.5, label: 'Kd (Deriv)' },
              manualMode: { type: 'boolean', value: false, label: 'Modo Manual' }
            }
          }
        ]
      },
      {
        id: 'p2',
        name: 'Sistema Termico T-302',
        connected: false,
        paused: false,
        variables: [
          {
            id: 'var_0',
            name: 'Temperatura',
            type: 'sensor',
            unit: '°C',
            setpoint: 60,
            pvMin: 0,
            pvMax: 150,
            mvMin: 0,
            mvMax: 100,
          },
          {
            id: 'var_1',
            name: 'Resistência R-01',
            type: 'atuador',
            unit: '%',
            setpoint: 0,
            pvMin: 0,
            pvMax: 100,
            mvMin: 0,
            mvMax: 100,
            linkedSensorIds: ['var_0'],
          }
        ],
        stats: { errorAvg: 0, stability: 100, uptime: 0 },
        controllers: [
          {
            id: 'c2',
            name: 'PID Termico',
            type: 'PID',
            active: true,
            params: {
              kp: { type: 'number', value: 1.2, label: 'Kp (Prop)' },
              ki: { type: 'number', value: 0.01, label: 'Ki (Int)' },
              kd: { type: 'number', value: 0.2, label: 'Kd (Deriv)' },
              manualMode: { type: 'boolean', value: false, label: 'Modo Manual' }
            }
          }
        ]
      }
    ]
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

  addPlant(plant: Omit<Plant, 'stats'>) {
    this.state.plants.push({
      ...plant,
      stats: { errorAvg: 0, stability: 100, uptime: 0 }
    });
  }

  removePlant(plantId: string) {
    const idx = this.state.plants.findIndex(p => p.id === plantId);
    if (idx > -1) {
      this.state.plants.splice(idx, 1);
      if (this.state.activePlantId === plantId) {
        this.state.activePlantId = this.state.plants[0]?.id || 'p1';
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

