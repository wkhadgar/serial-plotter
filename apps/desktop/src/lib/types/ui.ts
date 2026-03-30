export type TabKey = 'plotter' | 'analyzer' | 'plugins';

export const MODULE_TABS = {
  plotter: { label: 'Plotter', icon: 'TrendingUp' },
  analyzer: { label: 'Analyzer', icon: 'BarChart3' },
  plugins: { label: 'Plugins', icon: 'Puzzle' }
} as const;
