export type ControllerType = 'PID' | 'Flow' | 'Level' | (string & {});

export type ParamType = 'number' | 'boolean' | 'string';

export interface ControllerParam {
  type: ParamType;
  value: number | boolean | string;
  label: string;
}

export function isValidControllerParamValue(type: ParamType, value: unknown): value is ControllerParam['value'] {
  if (type === 'number') {
    return typeof value === 'number' && Number.isFinite(value);
  }

  if (type === 'boolean') {
    return typeof value === 'boolean';
  }

  return typeof value === 'string';
}

export function normalizeControllerParamValue(
  param: ControllerParam,
  value: unknown
): ControllerParam['value'] | null {
  if (isValidControllerParamValue(param.type, value)) {
    return value;
  }

  if (param.type === 'number' && typeof value === 'string') {
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : null;
  }

  if (param.type === 'string' && value != null) {
    return String(value);
  }

  return null;
}

export interface PIDParams {
  kp: ControllerParam;
  ki: ControllerParam;
  kd: ControllerParam;
  manualMode: ControllerParam;
}

export interface Controller {
  id: string;
  pluginId?: string;
  pluginName?: string;
  name: string;
  type: ControllerType;
  active: boolean;
  inputVariableIds: string[];
  outputVariableIds: string[];
  params: PIDParams | Record<string, ControllerParam>;
}
