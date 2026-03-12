export type ControllerType = 'PID' | 'Flow' | 'Level' | (string & {});

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
