<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import { BackendStatusDot } from "$components";
  import { authStore } from "$lib/auth";
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import HomeIcon from "@lucide/svelte/icons/home";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import WalletIcon from "@lucide/svelte/icons/wallet";
  import UserIcon from "@lucide/svelte/icons/user";

  const isHomeActive = $derived(page.url.pathname === resolve("/"));
  const isSettingsActive = $derived(page.url.pathname === resolve("/settings"));
  const username = $derived(authStore.currentUser?.name ?? "User");
</script>

<Sidebar.Root collapsible="icon">
  <Sidebar.Header>
    <div class="flex items-center justify-between gap-2 p-2">
      <div class="flex items-center gap-2 overflow-hidden">
        <div
          class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-emerald-600 text-white shadow-sm"
        >
          <WalletIcon class="size-4" />
        </div>
        <div class="flex flex-col truncate">
          <span class="truncate text-sm leading-tight font-semibold">CoSave</span>
          <span class="truncate text-xs text-muted-foreground">Finance Hub</span>
        </div>
      </div>
      <div class="group-data-[collapsible=icon]:hidden">
        <BackendStatusDot />
      </div>
    </div>
  </Sidebar.Header>

  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          <Sidebar.MenuItem>
            <Sidebar.MenuButton tooltipContent="Home" isActive={isHomeActive}>
              {#snippet child({ props })}
                <a href={resolve("/")} {...props}>
                  <HomeIcon />
                  <span>Home</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>

  <Sidebar.Footer>
    <Sidebar.Menu>
      <Sidebar.MenuItem>
        <Sidebar.MenuButton tooltipContent="Settings" isActive={isSettingsActive}>
          {#snippet child({ props })}
            <a href={resolve("/settings")} {...props}>
              <SettingsIcon />
              <span>Settings</span>
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>

      <Sidebar.MenuItem>
        <Sidebar.MenuButton tooltipContent="Log Out" onclick={() => authStore.logout()}>
          <LogOutIcon />
          <span>Log Out</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>

    <div
      class="mt-2 flex items-center gap-2 rounded-md border border-border/40 p-2 text-xs group-data-[collapsible=icon]:hidden"
    >
      <div
        class="flex size-6 shrink-0 items-center justify-center rounded-full bg-muted text-muted-foreground"
      >
        <UserIcon class="size-3.5" />
      </div>
      <div class="flex flex-col truncate">
        <span class="truncate font-medium text-foreground">{username}</span>
        <span class="truncate text-[10px] text-muted-foreground">Logged in</span>
      </div>
    </div>
  </Sidebar.Footer>
</Sidebar.Root>
