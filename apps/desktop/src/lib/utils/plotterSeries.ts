import { DEFAULT_LINE_COLORS } from '$lib/types/chart';
import { getVariableKeys, type Plant, type PlantVariable, type PlantSeriesDescriptor } from '$lib/types/plant';

export type SeriesStyle = {
  color: string;
  visible: boolean;
  label: string;
};

export type SeriesControl = {
  key: string;
  label: string;
  color: string;
  visible: boolean;
};

export const ACTUATOR_PALETTE = ['#10b981', '#06b6d4', '#8b5cf6', '#f97316', '#ec4899', '#14b8a6'];

function getSeriesLabel(
  catalogByKey: Map<string, PlantSeriesDescriptor>,
  key: string,
  fallback: string
): string {
  return catalogByKey.get(key)?.label ?? fallback;
}

export function buildSeriesStyles(
  plant: Plant,
  current: Record<string, SeriesStyle>,
  catalogByKey: Map<string, PlantSeriesDescriptor>
): Record<string, SeriesStyle> {
  const next: Record<string, SeriesStyle> = {};
  let actuatorColorIndex = 0;

  plant.variables.forEach((variable, index) => {
    const keys = getVariableKeys(index);
    const currentPv = current[keys.pv];
    const currentSp = current[keys.sp];

    if (variable.type === 'sensor') {
      next[keys.pv] = {
        color: currentPv?.color ?? DEFAULT_LINE_COLORS.pv,
        visible: currentPv?.visible ?? true,
        label: getSeriesLabel(catalogByKey, keys.pv, variable.name),
      };
      next[keys.sp] = {
        color: currentSp?.color ?? DEFAULT_LINE_COLORS.sp,
        visible: currentSp?.visible ?? true,
        label: 'Setpoint',
      };
      return;
    }

    next[keys.pv] = {
      color: currentPv?.color ?? ACTUATOR_PALETTE[actuatorColorIndex % ACTUATOR_PALETTE.length],
      visible: currentPv?.visible ?? true,
      label: getSeriesLabel(catalogByKey, keys.pv, variable.name),
    };
    actuatorColorIndex += 1;
  });

  return next;
}

export function buildContextSeriesControls(params: {
  plant: Plant;
  contextSensor: { variable: PlantVariable; index: number };
  seriesStyles: Record<string, SeriesStyle>;
  catalogByKey: Map<string, PlantSeriesDescriptor>;
}): SeriesControl[] {
  const { plant, contextSensor, seriesStyles, catalogByKey } = params;
  const controls: SeriesControl[] = [];
  const sensorKeys = getVariableKeys(contextSensor.index);

  const pvStyle = seriesStyles[sensorKeys.pv];
  const spStyle = seriesStyles[sensorKeys.sp];

  controls.push({
    key: sensorKeys.pv,
    label: pvStyle?.label ?? getSeriesLabel(catalogByKey, sensorKeys.pv, `${contextSensor.variable.name} PV`),
    color: pvStyle?.color ?? DEFAULT_LINE_COLORS.pv,
    visible: pvStyle?.visible ?? true,
  });

  controls.push({
    key: sensorKeys.sp,
    label: spStyle?.label ?? 'Setpoint',
    color: spStyle?.color ?? DEFAULT_LINE_COLORS.sp,
    visible: spStyle?.visible ?? true,
  });

  plant.variables.forEach((variable, index) => {
    if (variable.type !== 'atuador' || !variable.linkedSensorIds?.includes(contextSensor.variable.id)) {
      return;
    }

    const actuatorKey = getVariableKeys(index).pv;
    const actuatorStyle = seriesStyles[actuatorKey];

    controls.push({
      key: actuatorKey,
      label: actuatorStyle?.label ?? getSeriesLabel(catalogByKey, actuatorKey, variable.name),
      color: actuatorStyle?.color ?? DEFAULT_LINE_COLORS.mv,
      visible: actuatorStyle?.visible ?? true,
    });
  });

  return controls;
}
