import type { Controller } from '$lib/types/controller';
import type { Plant, PlantVariable } from '$lib/types/plant';
import type { PluginDefinition, PluginInstance, SchemaFieldValue } from '$lib/types/plugin';

export function cloneVariable(variable: PlantVariable): PlantVariable {
  return {
    ...variable,
    linkedSensorIds: variable.linkedSensorIds ? [...variable.linkedSensorIds] : undefined,
  };
}

export function cloneController(controller: Controller): Controller {
  return {
    ...controller,
    inputVariableIds: [...(controller.inputVariableIds ?? [])],
    outputVariableIds: [...(controller.outputVariableIds ?? [])],
    params: Object.fromEntries(
      Object.entries(controller.params ?? {}).map(([key, param]) => [key, { ...param }])
    ),
  };
}

export function cloneDriver(instance: PluginInstance): PluginInstance {
  return {
    ...instance,
    config: { ...instance.config },
  };
}

export function normalizeVariables(nextVariables: PlantVariable[]): PlantVariable[] {
  const sensorIdMap = new Map<string, string>();

  const normalized = nextVariables.map((variable, index) => {
    const nextId = `var_${index}`;
    if (variable.type === 'sensor') {
      sensorIdMap.set(variable.id, nextId);
    }

    return {
      ...variable,
      id: nextId,
      linkedSensorIds: variable.type === 'atuador' ? [...(variable.linkedSensorIds ?? [])] : undefined,
    };
  });

  return normalized.map((variable) => {
    if (variable.type !== 'atuador') {
      return variable;
    }

    const linkedSensorIds = Array.from(
      new Set(
        (variable.linkedSensorIds ?? [])
          .map((sensorId) => sensorIdMap.get(sensorId))
          .filter((sensorId): sensorId is string => !!sensorId)
      )
    );

    return {
      ...variable,
      linkedSensorIds,
    };
  });
}

export function buildDriverAutoConfig(currentVariables: PlantVariable[]): Record<string, SchemaFieldValue> {
  return {
    num_sensors: currentVariables.filter((variable) => variable.type === 'sensor').length,
    num_actuators: currentVariables.filter((variable) => variable.type === 'atuador').length,
  };
}

export function syncDriverWithVariables(
  instance: PluginInstance,
  currentVariables: PlantVariable[]
): PluginInstance {
  return {
    ...instance,
    config: {
      ...instance.config,
      ...buildDriverAutoConfig(currentVariables),
    },
  };
}

export function createDriverPlaceholder(
  driverId: string,
  availablePlugins: PluginDefinition[],
  currentVariables: PlantVariable[]
): PluginInstance | null {
  const plugin = availablePlugins.find((entry) => entry.id === driverId);
  if (!plugin) {
    return null;
  }

  return {
    pluginId: driverId,
    pluginName: plugin.name,
    pluginKind: plugin.kind,
    config: buildDriverAutoConfig(currentVariables),
  };
}

function areSchemaFieldValuesEqual(left: SchemaFieldValue, right: SchemaFieldValue): boolean {
  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right) || left.length !== right.length) {
      return false;
    }

    return left.every((value, index) => areSchemaFieldValuesEqual(value, right[index]));
  }

  return left === right;
}

export function arePluginInstancesEqual(left: PluginInstance, right: PluginInstance): boolean {
  if (
    left.pluginId !== right.pluginId ||
    left.pluginName !== right.pluginName ||
    left.pluginKind !== right.pluginKind
  ) {
    return false;
  }

  const leftEntries = Object.entries(left.config);
  const rightEntries = Object.entries(right.config);

  if (leftEntries.length !== rightEntries.length) {
    return false;
  }

  return leftEntries.every(([key, value]) =>
    key in right.config && areSchemaFieldValuesEqual(value, right.config[key])
  );
}

export function buildInitialPlantForm(
  plant: Plant | null,
  availablePlugins: PluginDefinition[],
  fallbackVariable: PlantVariable
) {
  const nextVariables = normalizeVariables(
    (plant?.variables ?? [fallbackVariable]).map(cloneVariable)
  );

  return {
    plantName: plant?.name ?? '',
    sampleTimeMs: Math.max(1, Math.round(plant?.sampleTimeMs ?? 100)),
    variables: nextVariables,
    selectedControllers: (plant?.controllers ?? []).map(cloneController),
    driverInstance: plant?.driver
      ? syncDriverWithVariables(cloneDriver(plant.driver), nextVariables)
      : plant?.driverId
        ? createDriverPlaceholder(plant.driverId, availablePlugins, nextVariables)
        : null,
  };
}
