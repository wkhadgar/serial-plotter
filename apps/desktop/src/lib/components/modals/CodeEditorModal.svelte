<script lang="ts">
  /**
   * ============================================================================
   * CODE EDITOR MODAL - Editor de Código com Syntax Highlighting
   * ============================================================================
   *
   * Modal global e reutilizável para edição de código.
   * Suporta:
   * - Syntax highlighting para Python e Rust (highlight.js)
   * - Seletor de linguagem
   * - Campo de nome do arquivo
   * - Edição em textarea sincronizada com preview highlight
   * - Atalho Tab para indentação
   *
   * Projetado para ser usado em qualquer contexto da aplicação.
   */
  import { X, Check, FileCode } from 'lucide-svelte';
  import { untrack } from 'svelte';
  import hljs from 'highlight.js/lib/core';
  import python from 'highlight.js/lib/languages/python';

  // Registra linguagem
  hljs.registerLanguage('python', python);

  // ─── Types ──────────────────────────────────────────────────────────────────

  export type CodeLanguage = 'python';

  export interface CodeEditorResult {
    code: string;
    language: CodeLanguage;
    fileName: string;
  }

  export const CODE_LANGUAGE_LABELS: Record<CodeLanguage, string> = {
    python: 'Python',
  };

  export const CODE_LANGUAGE_EXTENSIONS: Record<CodeLanguage, string> = {
    python: '.py',
  };

  // ─── Props ──────────────────────────────────────────────────────────────────

  interface Props {
    visible: boolean;
    /** Código inicial */
    initialCode?: string;
    /** Nome do arquivo inicial */
    initialFileName?: string;
    /** Título do modal */
    title?: string;
    onClose: () => void;
    onSave: (result: CodeEditorResult) => void;
  }

  let {
    visible = $bindable(),
    initialCode = '',
    initialFileName = '',
    title = 'Editor de Código',
    onClose,
    onSave,
  }: Props = $props();

  // ─── State ──────────────────────────────────────────────────────────────────

  let code = $state('');
  const language: CodeLanguage = 'python';
  let fileName = $state('');
  let highlightedHtml = $state('');
  let textareaEl = $state<HTMLTextAreaElement | null>(null);
  let preEl = $state<HTMLPreElement | null>(null);

  // ─── Init on visible ───────────────────────────────────────────────────────

  $effect(() => {
    if (visible) {
      untrack(() => {
        code = initialCode;
        fileName = initialFileName;
      });
    }
  });

  // ─── Highlight ──────────────────────────────────────────────────────────────

  function updateHighlight() {
    if (!code) {
      highlightedHtml = '\n'; // preserve min height
      return;
    }
    try {
      const result = hljs.highlight(code, { language });
      highlightedHtml = result.value + '\n'; // trailing newline for scroll parity
    } catch {
      highlightedHtml = escapeHtml(code) + '\n';
    }
  }

  function escapeHtml(str: string): string {
    return str.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  $effect(() => {
    // Re-highlight when code changes
    code;
    updateHighlight();
  });

  // ─── Sync scroll ───────────────────────────────────────────────────────────

  function handleScroll() {
    if (textareaEl && preEl) {
      preEl.scrollTop = textareaEl.scrollTop;
      preEl.scrollLeft = textareaEl.scrollLeft;
    }
  }

  // ─── Tab key ───────────────────────────────────────────────────────────────

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Tab') {
      e.preventDefault();
      const textarea = e.target as HTMLTextAreaElement;
      const start = textarea.selectionStart;
      const end = textarea.selectionEnd;
      const indent = '    ';
      code = code.slice(0, start) + indent + code.slice(end);
      // Restore cursor after Svelte re-renders
      requestAnimationFrame(() => {
        textarea.selectionStart = textarea.selectionEnd = start + indent.length;
      });
    }
  }

  // ─── Actions ────────────────────────────────────────────────────────────────

  function handleSave() {
    let finalName = fileName.trim();
    if (!finalName) {
      finalName = 'main.py';
    } else if (!finalName.endsWith('.py')) {
      finalName = finalName.replace(/\.[^.]+$/, '') + '.py';
    }
    onSave({ code, language, fileName: finalName });
    onClose();
  }

  function handleClose() {
    onClose();
  }
</script>

<!-- highlight.js theme (GitHub Dark) injected inline for dark mode -->
<svelte:head>
  {#if visible}
    <link
      rel="stylesheet"
      href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.11.1/styles/github-dark.min.css"
    />
  {/if}
</svelte:head>

{#if visible}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-[80] flex items-center justify-center bg-black/60 backdrop-blur-sm"
    onclick={handleClose}
  >
    <div
      class="bg-white dark:bg-[#0c0c0e] rounded-2xl shadow-2xl w-full max-w-4xl h-[85vh] flex flex-col overflow-hidden border border-slate-200 dark:border-white/10"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-5 py-3 border-b border-slate-200 dark:border-white/5 shrink-0">
        <div class="flex items-center gap-3">
          <FileCode size={18} class="text-blue-500" />
          <h2 class="text-sm font-bold text-slate-800 dark:text-white">{title}</h2>
        </div>
        <button
          onclick={handleClose}
          class="p-1.5 rounded-lg hover:bg-slate-100 dark:hover:bg-white/5 text-slate-400 transition-colors"
        >
          <X size={18} />
        </button>
      </div>

      <!-- Toolbar -->
      <div class="flex items-center gap-3 px-5 py-2.5 border-b border-slate-200 dark:border-white/5 bg-slate-50 dark:bg-white/[0.02] shrink-0">
        <!-- Nome do arquivo -->
        <div class="flex-1">
          <input
            type="text"
            bind:value={fileName}
            placeholder="main.py"
            class="w-full h-8 px-3 rounded-md border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500/50"
          />
        </div>
        <!-- Linguagem (badge) -->
        <span class="h-8 px-3 flex items-center rounded-md border border-slate-200 dark:border-white/10 bg-white dark:bg-[#18181b] text-sm text-slate-500 dark:text-zinc-400 select-none">
          Python
        </span>
      </div>

      <!-- Editor area -->
      <div class="flex-1 relative overflow-hidden bg-[#0d1117] min-h-0">
        <!-- Line numbers + code container -->
        <div class="absolute inset-0 flex">
          <!-- Line numbers -->
          <div class="shrink-0 w-12 bg-[#0d1117] border-r border-white/5 overflow-hidden select-none" aria-hidden="true">
            <div class="pt-4 pb-4 text-right pr-3">
              {#each code.split('\n') as _, idx}
                <div class="text-[11px] leading-[1.65] text-zinc-600 font-mono">{idx + 1}</div>
              {/each}
            </div>
          </div>
          <!-- Highlight + textarea overlay -->
          <div class="flex-1 relative min-w-0 overflow-hidden">
            <!-- Highlighted code (background) -->
            <pre
              bind:this={preEl}
              class="absolute inset-0 pt-4 pb-4 pl-4 pr-4 overflow-auto text-[13px] leading-[1.65] font-mono text-white whitespace-pre pointer-events-none m-0 bg-transparent"
              aria-hidden="true"
            >{@html highlightedHtml}</pre>
            <!-- Textarea (foreground, transparent text) -->
            <textarea
              bind:this={textareaEl}
              bind:value={code}
              onscroll={handleScroll}
              onkeydown={handleKeydown}
              spellcheck="false"
              autocomplete="off"
              autocapitalize="off"
              class="absolute inset-0 w-full h-full pt-4 pb-4 pl-4 pr-4 text-[13px] leading-[1.65] font-mono bg-transparent text-transparent caret-white resize-none focus:outline-none selection:bg-blue-500/30 overflow-auto whitespace-pre"
            ></textarea>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-between px-5 py-3 border-t border-slate-200 dark:border-white/5 bg-slate-50 dark:bg-white/[0.02] shrink-0">
        <div class="text-xs text-slate-400 dark:text-zinc-500">
          {code.split('\n').length} linhas · {code.length} caracteres · {CODE_LANGUAGE_LABELS[language]}
        </div>
        <div class="flex items-center gap-2">
          <button
            onclick={handleClose}
            class="px-4 py-1.5 rounded-lg text-sm font-medium text-slate-600 dark:text-zinc-400 hover:bg-slate-200 dark:hover:bg-white/10 transition-colors"
          >
            Cancelar
          </button>
          <button
            onclick={handleSave}
            class="px-5 py-1.5 rounded-lg text-sm font-bold bg-blue-600 hover:bg-blue-700 text-white transition-colors flex items-center gap-1.5"
          >
            <Check size={14} />
            Salvar
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
