import type { PlantDataPoint } from '$lib/types/plant';

/**
 * Exporta dados de uma planta para CSV e dispara download.
 * Retorna false se não há dados para exportar.
 */
export function exportPlantDataCSV(plantName: string, data: PlantDataPoint[]): boolean {
  if (data.length === 0) return false;

  const csvContent =
    'data:text/csv;charset=utf-8,' +
    'Time,Setpoint,PV,MV\n' +
    data
      .map((e) => `${e.time.toFixed(2)},${e.sp},${e.pv.toFixed(2)},${e.mv.toFixed(2)}`)
      .join('\n');

  const encodedUri = encodeURI(csvContent);
  const link = document.createElement('a');
  link.setAttribute('href', encodedUri);
  link.setAttribute('download', `${plantName.replace(/\s+/g, '_')}_data.csv`);
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  return true;
}
