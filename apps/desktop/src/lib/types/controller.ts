export type ControllerType = 'PID' | 'Flow' | 'Level';

export type ControllerMode = 'auto' | 'manual' | 'alarm';

export type ParamType = 'number' | 'boolean' | 'string';

export interface ControllerParam {
  type: ParamType;
  value: number | boolean | string;
  label: string;
}

export interface PIDParams {
  kp: ControllerParam;
  ki: ControllerParam;
  kd: ControllerParam;
  manualMode: ControllerParam;
}

export interface Controller {
  id: string;
  name: string;
  type: ControllerType;
  active: boolean;
  params: PIDParams | Record<string, ControllerParam>;
}

export interface PIDConfig {
  kp: number;
  ki: number;
  kd: number;
  minOut: number;
  maxOut: number;
}
