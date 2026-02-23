<script lang="ts">
  type BadgeVariant = 'default' | 'success' | 'warning' | 'danger' | 'info';

  interface Props {
    variant?: BadgeVariant;
    text: string;
  }

  let { variant = 'default', text } = $props();

  const variantClasses = {
    default: 'bg-slate-100 dark:bg-white/10 text-slate-900 dark:text-white',
    success: 'bg-emerald-100 dark:bg-emerald-900/30 text-emerald-900 dark:text-emerald-200',
    warning: 'bg-amber-100 dark:bg-amber-900/30 text-amber-900 dark:text-amber-200',
    danger: 'bg-red-100 dark:bg-red-900/30 text-red-900 dark:text-red-200',
    info: 'bg-blue-100 dark:bg-blue-900/30 text-blue-900 dark:text-blue-200'
  } as const;

  const classes = $derived(`inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium ${variantClasses[variant as BadgeVariant]}`);
</script>

<span class={classes}>
  {text}
</span>
