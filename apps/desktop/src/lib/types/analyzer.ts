export interface AnalyzerActuatorInfo {
  id: string;
  name: string;
  unit: string;
}

export interface AnalyzerVariable {
  index: number;
  sensorId: string;
  sensorName: string;
  sensorUnit: string;
  setpointId: string;
  actuators: AnalyzerActuatorInfo[];
  selected: boolean;
}

export interface ProcessedVariableData {
  variable: AnalyzerVariable;
  sensorData: Array<{ time: number; value: number }>;
  setpointData: Array<{ time: number; value: number }>;
  actuatorsData: Array<{
    id: string;
    name: string;
    data: Array<{ time: number; value: number }>;
  }>;
  sensorRange: { min: number; max: number };
  actuatorRange: { min: number; max: number };
}
