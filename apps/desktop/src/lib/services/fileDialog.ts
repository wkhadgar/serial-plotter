export interface FileFilter {
  name: string;
  extensions: readonly string[] | string[];
}

export interface OpenFileOptions {
  title?: string;
  filters?: readonly FileFilter[];
  multiple?: boolean;
}

export interface FileResult {
  file: File;
  name: string;
  path: string;
  extension: string;
}

export function openFileDialog(options: OpenFileOptions = {}): Promise<FileResult | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.style.display = 'none';
    let settled = false;
    
    if (options.filters && options.filters.length > 0) {
      const accept = options.filters
        .flatMap(f => f.extensions.map(ext => `.${ext}`))
        .join(',');
      input.accept = accept;
    }
    
    input.multiple = options.multiple ?? false;

    const cleanup = () => {
      if (input.parentNode) {
        input.parentNode.removeChild(input);
      }
      window.removeEventListener('focus', handleFocus);
    };

    const finish = (result: FileResult | null) => {
      if (settled) return;
      settled = true;
      resolve(result);
      cleanup();
    };

    input.onchange = () => {
      const file = input.files?.[0];
      if (file) {
        const extension = file.name.split('.').pop()?.toLowerCase() ?? '';
        finish({
          file,
          name: file.name,
          path: file.name,
          extension,
        });
      } else {
        finish(null);
      }
    };
    
    input.oncancel = () => {
      finish(null);
    };
    
    function handleFocus() {
      setTimeout(() => {
        if (!input.files?.length) {
          finish(null);
        }
      }, 300);
    }
    
    document.body.appendChild(input);
    window.addEventListener('focus', handleFocus);
    input.click();
  });
}

export function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error('Erro ao ler arquivo'));
    reader.readAsText(file);
  });
}

export async function readFileAsJSON<T = unknown>(file: File): Promise<T> {
  const text = await readFileAsText(file);
  return JSON.parse(text) as T;
}

/**
 * Filtros pré-definidos para tipos comuns de arquivos.
 */
export const FILE_FILTERS = {
  plant: [{ name: 'Planta', extensions: ['plant', 'json'] }],
  csv: [{ name: 'CSV', extensions: ['csv'] }],
  json: [{ name: 'JSON', extensions: ['json'] }],
  all: [{ name: 'Todos', extensions: ['*'] }],
} as const;
