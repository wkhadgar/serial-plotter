export type TabKey = 'plotter' | 'poles';

export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' },
  poles: { label: 'Polos', icon: 'Activity' }
} as const;

export type ThemeMode = 'dark' | 'light';

export const BREAKPOINT_TABLET = 768;
export const BREAKPOINT_MEDIUM = 1024;
export const BREAKPOINT_LARGE = 1280;

/**
 * Estados de visibilidade de painéis
 */
export interface PanelStates {
  leftOpen: boolean;
  rightOpen: boolean;
}

/**
 * Configurações de tema
 */
export interface ThemeConfig {
  bgApp: string;
  bgPanel: string;
  bgSurface: string;
  border: string;
  textMain: string;
  textMuted: string;
}

/**
 * Cores para gráficos
 */
export interface ChartColors {
  grid: string;
  axis: string;
  text: string;
  tooltipBg: string;
  border: string;
  limitLine: string;
}
