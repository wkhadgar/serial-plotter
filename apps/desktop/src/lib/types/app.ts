import type { Plant } from './plant';
import type { TabKey } from './ui';

export interface AppState {
  theme: 'dark' | 'light';
  activeModule: TabKey;
  activePlantId: string | null;
  sidebarCollapsed: boolean;
  showGlobalSettings: boolean;
  showControllerPanel: boolean;
  plants: Plant[];
}
