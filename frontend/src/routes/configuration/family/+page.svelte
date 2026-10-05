<script lang="ts">
  import { resolve } from "$app/paths";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import { FamilyView } from "$lib/features/family";
</script>

<svelte:head>
  <title>Family & Accounts Configuration &bull; CoSave</title>
</svelte:head>

<div class="flex flex-col gap-6">
  <!-- Breadcrumb Navigation Bar -->
  <div class="text-muted-foreground flex items-center gap-2 text-xs">
    <a href={resolve("/configuration")} class="hover:text-foreground transition-colors">
      Configuration
    </a>
    <ChevronRightIcon class="text-muted-foreground/60 size-3.5" />
    <span class="text-foreground font-medium">Family & Accounts</span>
  </div>

  <!-- Primary Family Feature View -->
  <FamilyView />
</div>
