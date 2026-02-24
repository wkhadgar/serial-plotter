export type TabKey = 'plotter' | 'poles';

export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' },
  poles: { label: 'Polos', icon: 'Activity' }
} as const;

export type ThemeMode = 'dark' | 'light';

