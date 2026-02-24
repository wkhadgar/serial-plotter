import type { Plant } from '$lib/types/plant';
import { getPlantData, getPlantStats, setPlantStats } from '$lib/stores/plantData';

const SIM_INTERVAL = 100;
const DT = SIM_INTERVAL / 1000;

const thermalState = new Map<string, { x1: number; x2: number }>();

function simulatePlant(plant: Plant): void {
  const data = getPlantData(plant.id);
  const last =
    data.length > 0
      ? data[data.length - 1]
      : { pv: 0, mv: 0, time: 0, sp: plant.setpoint };

  const time = last.time + DT;
  let totalMv = 0;
  const error = plant.setpoint - last.pv;

  plant.controllers.forEach((ctrl) => {
    if (!ctrl.active) return;
    const p = ctrl.params as any;
    if (p.manualMode?.value) {
      totalMv = 50;
      return;
    }
    const kp = Number(p.kp?.value) || 0;
    const ki = Number(p.ki?.value) || 0;
    const kd = Number(p.kd?.value) || 0;
    const prevPv = data.length > 1 ? data[data.length - 2].pv : last.pv;
    totalMv += kp * error + ki * error * DT - kd * (last.pv - prevPv) / DT;
  });

  totalMv = Math.max(0, Math.min(100, totalMv + (Math.random() * 1.5 - 0.75)));

  let newPv: number;
  if (plant.id === 'p2') {
    const k = 1.63,
      t1 = 0.003,
      t2 = 3.03;
    const st = thermalState.get(plant.id) ?? { x1: 0, x2: 0 };
    const x1 = st.x1 + DT * (-t1 * st.x1 + t1 * totalMv);
    const x2 = st.x2 + DT * (-t2 * st.x2 + t2 * x1);
    thermalState.set(plant.id, { x1, x2 });
    newPv = k * x2 + (Math.random() * 0.2 - 0.1);
  } else {
    newPv = last.pv * 0.94 + totalMv * 0.06 + (Math.random() * 0.4 - 0.2);
  }

  data.push({ time, sp: plant.setpoint, pv: newPv, mv: totalMv });

  const prev = getPlantStats(plant.id);
  setPlantStats(plant.id, {
    errorAvg: prev.errorAvg * 0.95 + Math.abs(error) * 0.05,
    stability: Math.max(0, 100 - (prev.errorAvg * 0.95 + Math.abs(error) * 0.05) * 2),
    uptime: prev.uptime + 1,
  });
}

/**
 * Inicia o loop de simulação para todas as plantas conectadas/ativas.
 * Retorna a função de cleanup para parar o intervalo.
 */
export function startSimulation(getPlants: () => Plant[]): () => void {
  const interval = setInterval(() => {
    getPlants().forEach((plant) => {
      if (!plant.connected || plant.paused) return;
      simulatePlant(plant);
    });
  }, SIM_INTERVAL);

  return () => clearInterval(interval);
}
