/**
 * Tipo de controlador disponível
 */
export type ControllerType = 'PID' | 'Flow' | 'Level';

/**
 * Modo de operação do controlador
 */
export type ControllerMode = 'auto' | 'manual' | 'alarm';

/**
 * Tipo de parâmetro do controlador
 */
export type ParamType = 'number' | 'boolean' | 'string';

/**
 * Definição de um parâmetro do controlador
 */
export interface ControllerParam {
  type: ParamType;
  value: number | boolean | string;
  label: string;
}

/**
 * Parâmetros PID
 */
export interface PIDParams {
  kp: ControllerParam;
  ki: ControllerParam;
  kd: ControllerParam;
  manualMode: ControllerParam;
}

/**
 * Representa um controlador industrial
 */
export interface Controller {
  id: string;
  name: string;
  type: ControllerType;
  active: boolean;
  params: PIDParams | Record<string, ControllerParam>;
}

/**
 * Configuração de um controlador PID
 */
export interface PIDConfig {
  kp: number;
  ki: number;
  kd: number;
  minOut: number;
  maxOut: number;
}
