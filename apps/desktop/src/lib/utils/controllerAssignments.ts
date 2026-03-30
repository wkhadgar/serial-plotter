import type { Controller } from '$lib/types/controller';
import type { PlantVariable } from '$lib/types/plant';

function buildVariableMap(variables: PlantVariable[]): Map<string, PlantVariable> {
  return new Map(variables.map((variable) => [variable.id, variable]));
}

function formatVariableNames(ids: string[], variables: PlantVariable[]): string {
  const variableMap = buildVariableMap(variables);
  return ids
    .map((id) => variableMap.get(id)?.name ?? id)
    .join(', ');
}

export function validateControllerBindings(
  controller: Controller,
  variables: PlantVariable[]
): string | null {
  if (controller.inputVariableIds.length === 0) {
    return `O controlador "${controller.name}" precisa de pelo menos uma variavel de entrada.`;
  }

  if (controller.outputVariableIds.length === 0) {
    return `O controlador "${controller.name}" precisa de pelo menos uma variavel de saida.`;
  }

  const variableMap = buildVariableMap(variables);

  for (const inputId of controller.inputVariableIds) {
    const variable = variableMap.get(inputId);
    if (!variable) {
      return `O controlador "${controller.name}" referencia uma variavel de entrada invalida.`;
    }

    if (variable.type !== 'sensor') {
      return `A variavel "${variable.name}" nao pode ser usada como entrada do controlador "${controller.name}".`;
    }
  }

  for (const outputId of controller.outputVariableIds) {
    const variable = variableMap.get(outputId);
    if (!variable) {
      return `O controlador "${controller.name}" referencia uma variavel de saida invalida.`;
    }

    if (variable.type !== 'atuador') {
      return `A variavel "${variable.name}" nao pode ser usada como saida do controlador "${controller.name}".`;
    }
  }

  return null;
}

export function getControllerActivationConflict(
  controller: Controller,
  controllers: Controller[],
  variables: PlantVariable[]
): string | null {
  const bindingError = validateControllerBindings(controller, variables);
  if (bindingError) {
    return bindingError;
  }

  const conflictingOutputs = controllers
    .filter((entry) => entry.id !== controller.id && entry.active)
    .flatMap((entry) =>
      entry.outputVariableIds.filter((outputId) => controller.outputVariableIds.includes(outputId))
    );

  if (conflictingOutputs.length === 0) {
    return null;
  }

  const uniqueOutputIds = Array.from(new Set(conflictingOutputs));
  return `Ja existe um controlador ativo para o(s) atuador(es): ${formatVariableNames(uniqueOutputIds, variables)}.`;
}

export function validateControllersForPlant(
  controllers: Controller[],
  variables: PlantVariable[]
): string | null {
  for (const controller of controllers) {
    const bindingError = validateControllerBindings(controller, variables);
    if (bindingError) {
      return bindingError;
    }
  }

  for (const controller of controllers) {
    if (!controller.active) {
      continue;
    }

    const conflict = getControllerActivationConflict(controller, controllers, variables);
    if (conflict) {
      return conflict;
    }
  }

  return null;
}
