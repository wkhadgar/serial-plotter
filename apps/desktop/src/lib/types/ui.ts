export type TabKey = 'plotter' | 'analyzer';

export const MODULE_TABS = {
  plotter: { label: 'Tendências', icon: 'TrendingUp' },
  analyzer: { label: 'Analyzer', icon: 'BarChart3' }
} as const;
