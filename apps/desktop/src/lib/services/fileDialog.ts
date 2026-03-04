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
    
    if (options.filters && options.filters.length > 0) {
      const accept = options.filters
        .flatMap(f => f.extensions.map(ext => `.${ext}`))
        .join(',');
      input.accept = accept;
    }
    
    input.multiple = options.multiple ?? false;

    input.onchange = () => {
      const file = input.files?.[0];
      if (file) {
        const extension = file.name.split('.').pop()?.toLowerCase() ?? '';
        resolve({
          file,
          name: file.name,
          path: file.name,
          extension,
        });
      } else {
        resolve(null);
      }
      document.body.removeChild(input);
    };
    
    input.oncancel = () => {
      resolve(null);
      document.body.removeChild(input);
    };
    
    const handleFocus = () => {
      setTimeout(() => {
        if (!input.files?.length) {
          resolve(null);
          document.body.removeChild(input);
        }
        window.removeEventListener('focus', handleFocus);
      }, 300);
    };
    
    document.body.appendChild(input);
    window.addEventListener('focus', handleFocus);
    input.click();
  });
}

export function openFilesDialog(options: OpenFileOptions = {}): Promise<FileResult[]> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.style.display = 'none';
    input.multiple = true;
    
    if (options.filters && options.filters.length > 0) {
      const accept = options.filters
        .flatMap(f => f.extensions.map(ext => `.${ext}`))
        .join(',');
      input.accept = accept;
    }
    
    input.onchange = () => {
      const files = input.files;
      if (files && files.length > 0) {
        const results: FileResult[] = Array.from(files).map(file => ({
          file,
          name: file.name,
          path: file.name,
          extension: file.name.split('.').pop()?.toLowerCase() ?? '',
        }));
        resolve(results);
      } else {
        resolve([]);
      }
      document.body.removeChild(input);
    };
    
    input.oncancel = () => {
      resolve([]);
      document.body.removeChild(input);
    };
    
    document.body.appendChild(input);
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
