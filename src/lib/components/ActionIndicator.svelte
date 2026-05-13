<script>
  import { formatToolName, summarizeToolArgs } from '$lib/utils/formatting.js';

  let { toolName, args } = $props();
</script>

<div class="flex items-center gap-2 px-3 py-2 my-2 mx-4 bg-[var(--color-surface-tertiary)] border border-[var(--color-brand-green)]/20 rounded-lg animate-pulse">
  <div class="w-2 h-2 bg-[var(--color-brand-green)] rounded-full animate-ping"></div>
  <span class="text-xs text-[var(--color-brand-green)] font-medium">
    {formatToolName(toolName)}
  </span>
  <span class="text-xs text-[var(--color-text-muted)] truncate">
    {summarizeToolArgs(args)}
  </span>
</div>
