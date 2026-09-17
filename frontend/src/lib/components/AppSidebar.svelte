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

  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";

  const isHomeActive = $derived(page.url.pathname === resolve("/"));
  const isConfigurationActive = $derived(page.url.pathname === resolve("/configuration"));
  const isSettingsActive = $derived(page.url.pathname === resolve("/settings"));
  const username = $derived(authStore.currentUser?.name ?? "User");
</script>

<Sidebar.Root collapsible="icon">
  <Sidebar.Header>
    <div
      class="flex items-center justify-between gap-2 p-2 group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:items-center group-data-[collapsible=icon]:gap-2 group-data-[collapsible=icon]:px-0"
    >
      <div
        class="flex items-center gap-2 overflow-hidden group-data-[collapsible=icon]:overflow-visible"
      >
        <div
          class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-emerald-600 text-white shadow-sm"
        >
          <WalletIcon class="size-4" />
        </div>
        <div class="flex flex-col truncate group-data-[collapsible=icon]:hidden">
          <span class="truncate text-sm leading-tight font-semibold">CoSave</span>
          <span class="text-muted-foreground truncate text-xs">Finance Hub</span>
        </div>
      </div>
      <div
        class="flex items-center gap-1 group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
      >
        <div class="group-data-[collapsible=icon]:hidden">
          <BackendStatusDot />
        </div>
        <Sidebar.Trigger />
      </div>
    </div>
  </Sidebar.Header>

  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu class="group-data-[collapsible=icon]:items-center">
          <Sidebar.MenuItem
            class="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
          >
            <Sidebar.MenuButton tooltipContent="Home" isActive={isHomeActive}>
              {#snippet child({ props })}
                <a href={resolve("/")} {...props}>
                  <HomeIcon />
                  <span>Home</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>

          <Sidebar.MenuItem
            class="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
          >
            <Sidebar.MenuButton tooltipContent="Configuration" isActive={isConfigurationActive}>
              {#snippet child({ props })}
                <a href={resolve("/configuration")} {...props}>
                  <SlidersHorizontalIcon />
                  <span>Configuration</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>

  <Sidebar.Footer>
    <Sidebar.Menu class="group-data-[collapsible=icon]:items-center">
      <Sidebar.MenuItem
        class="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
      >
        <Sidebar.MenuButton tooltipContent="Settings" isActive={isSettingsActive}>
          {#snippet child({ props })}
            <a href={resolve("/settings")} {...props}>
              <SettingsIcon />
              <span>Settings</span>
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>

      <Sidebar.MenuItem
        class="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
      >
        <Sidebar.MenuButton tooltipContent="Log Out" onclick={() => authStore.logout()}>
          <LogOutIcon />
          <span>Log Out</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>

    <div
      class="border-border/40 mt-2 flex items-center gap-2 rounded-md border p-2 text-xs group-data-[collapsible=icon]:hidden"
    >
      <div
        class="bg-muted text-muted-foreground flex size-6 shrink-0 items-center justify-center rounded-full"
      >
        <UserIcon class="size-3.5" />
      </div>
      <div class="flex flex-col truncate">
        <span class="text-foreground truncate font-medium">{username}</span>
        <span class="text-muted-foreground truncate text-[10px]">Logged in</span>
      </div>
    </div>
  </Sidebar.Footer>
</Sidebar.Root>
