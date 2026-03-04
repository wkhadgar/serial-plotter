import type { Plant, PlantDataPoint } from '$lib/types/plant';
import { getVariableKeys } from '$lib/types/plant';
import { getPlantData, getPlantStats, setPlantStats, getVariableStats, setVariableStats } from '$lib/stores/plantData';

const SIM_INTERVAL = 100;
const DT = SIM_INTERVAL / 1000;

const variableState = new Map<string, { x1: number; x2: number }>();

function simulatePlant(plant: Plant): void {
  const data = getPlantData(plant.id);
  const last = data.length > 0 ? data[data.length - 1] : null;
  const time = last ? last.time + DT : 0;

  const newPoint: PlantDataPoint = { time };

  plant.variables.forEach((variable, varIndex) => {
    const keys = getVariableKeys(varIndex);
    const stateKey = `${plant.id}_${varIndex}`;

    const prevPv = last ? last[keys.pv] ?? 0 : 0;
    const prevMv = last ? last[keys.mv] ?? 50 : 50;

    const error = variable.setpoint - prevPv;

    let mv = 0;
    const activeController = plant.controllers.find(c => c.active);

    if (activeController) {
      const p = activeController.params as any;
      if (p.manualMode?.value) {
        mv = 50;
      } else {
        const kp = Number(p.kp?.value) || 1;
        const ki = Number(p.ki?.value) || 0.01;
        const kd = Number(p.kd?.value) || 0;
        const prevPrevPv = data.length > 1 ? (data[data.length - 2][keys.pv] ?? prevPv) : prevPv;
        mv = kp * error + ki * error * DT - kd * (prevPv - prevPrevPv) / DT;
      }
    } else {
      mv = 50 + error * 0.5;
    }

    mv = Math.max(0, Math.min(100, mv + (Math.random() * 1.5 - 0.75)));

    let newPv: number;

    if (varIndex % 2 === 1) {
      const k = 1.63, t1 = 0.003, t2 = 3.03;
      const st = variableState.get(stateKey) ?? { x1: 0, x2: 0 };
      const x1 = st.x1 + DT * (-t1 * st.x1 + t1 * mv);
      const x2 = st.x2 + DT * (-t2 * st.x2 + t2 * x1);
      variableState.set(stateKey, { x1, x2 });
      newPv = k * x2 + (Math.random() * 0.2 - 0.1);
    } else {
      newPv = prevPv * 0.94 + mv * 0.06 + (Math.random() * 0.4 - 0.2);
    }

    newPv = Math.max(variable.pvMin, Math.min(variable.pvMax, newPv));

    newPoint[keys.pv] = newPv;
    newPoint[keys.sp] = variable.setpoint;
    newPoint[keys.mv] = mv;
  });

  data.push(newPoint);

  plant.variables.forEach((variable, varIndex) => {
    const keys = getVariableKeys(varIndex);
    const currentPv = newPoint[keys.pv] ?? 0;
    const prevPv = last ? last[keys.pv] ?? 0 : currentPv;

    const error = Math.abs(variable.setpoint - currentPv);

    const pvDelta = Math.abs(currentPv - prevPv);

    const prev = getVariableStats(plant.id, varIndex);
    const newRipple = prev.ripple * 0.9 + pvDelta * 0.1;

    const stability = Math.max(0, Math.min(100, 100 - newRipple * 20));

    setVariableStats(plant.id, varIndex, {
      errorAvg: prev.errorAvg * 0.95 + error * 0.05,
      stability,
      ripple: newRipple,
    });
  });

  const prev = getPlantStats(plant.id);
  setPlantStats(plant.id, {
    errorAvg: prev.errorAvg,
    stability: prev.stability,
    uptime: prev.uptime + 1,
  });
}

export function startSimulation(getPlants: () => Plant[]): () => void {
  const interval = setInterval(() => {
    getPlants().forEach((plant) => {
      if (!plant.connected || plant.paused) return;
      simulatePlant(plant);
    });
  }, SIM_INTERVAL);

  return () => clearInterval(interval);
}
