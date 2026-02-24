import type { Plant } from '$lib/types/plant';
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
        data: [],
        setpoint: 50,
        stats: { errorAvg: 0, stability: 100, uptime: 0 },
        controllers: [
          {
            id: 'c1',
            name: 'PID Principal',
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
        data: [],
        setpoint: 60,
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

  addPlant(plant: Omit<Plant, 'data' | 'stats'>) {
    this.state.plants.push({
      ...plant,
      data: [],
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

  updateSetpoint(plantId: string, setpoint: number) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) plant.setpoint = setpoint;
  }

  updatePlantData(plantId: string, data: Plant['data']) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) plant.data = data;
  }

  updatePlantStats(plantId: string, stats: Plant['stats']) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) plant.stats = stats;
  }

  updatePlantDataAndStats(plantId: string, data: Plant['data'], stats: Plant['stats']) {
    const plant = this.state.plants.find(p => p.id === plantId);
    if (plant) {
      plant.data = data;
      plant.stats = stats;
    }
  }
}

export const appStore = new AppStore();

