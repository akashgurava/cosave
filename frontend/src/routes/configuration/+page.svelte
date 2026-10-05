<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";

  onMount(() => {
    void goto(resolve("/configuration/family"), { replaceState: true });
  });
</script>

<div class="flex flex-1 items-center justify-center py-20">
  <div
    class="border-border/40 size-8 animate-spin rounded-full border-2 border-t-emerald-500"
  ></div>
</div>
