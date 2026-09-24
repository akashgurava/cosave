<script lang="ts">
  import { onMount } from "svelte";
  import type * as echartsType from "echarts";
  import { categoryStore } from "../store";
  import type { SankeyNodeData, SankeyLinkData, SelectedCategoryNode } from "../types";
  import { themeStore } from "$lib/theme";

  interface Props {
    activeFilter?: string | string[];
    onSelectNode?: (node: SelectedCategoryNode) => void;
  }

  let { activeFilter = "All", onSelectNode }: Props = $props();

  let chartContainer: HTMLDivElement | null = $state(null);
  let chartInstance: echartsType.ECharts | null = null;
  let echartsCore: typeof import("$lib/echarts-sankey").default | null = null;
  let nodeMap: Record<string, SankeyNodeData> = {};

  async function getEcharts(): Promise<typeof import("$lib/echarts-sankey").default> {
    if (!echartsCore) {
      const { default: echarts } = await import("$lib/echarts-sankey");
      echartsCore = echarts;
    }
    return echartsCore;
  }

  async function renderChart() {
    if (!chartContainer) return;
    const echarts = await getEcharts();
    if (!chartContainer) return;

    if (!chartInstance) {
      chartInstance = echarts.init(chartContainer, undefined, {
        renderer: "canvas",
      });

      chartInstance.on("click", (params) => {
        if (typeof params === "object" && params !== null) {
          const p = params as { dataType?: string; name?: unknown; data?: unknown };
          if (p.dataType === "node" && typeof p.name === "string") {
            handleNodeClick(p.name);
          } else if (p.dataType === "edge" && typeof p.data === "object" && p.data !== null) {
            const edge = p.data as { target?: unknown };
            if (typeof edge.target === "string") {
              handleNodeClick(edge.target);
            }
          }
        }
      });
    }

    const isDark = themeStore.isDark;
    const labelColor = isDark ? "#ffffff" : "#000000";
    const tooltipBg = isDark ? "rgba(10, 10, 10, 0.95)" : "rgba(255, 255, 255, 0.95)";
    const tooltipBorder = isDark ? "rgba(255, 255, 255, 0.15)" : "rgba(0, 0, 0, 0.15)";
    const tooltipText = isDark ? "#ffffff" : "#000000";
    const tooltipMuted = isDark ? "rgba(255, 255, 255, 0.65)" : "rgba(0, 0, 0, 0.65)";
    const tooltipStrong = isDark ? "#ffffff" : "#000000";

    const { nodes, links } = categoryStore.getSankeyData(activeFilter);

    // Build map of nodeId to node details for labels, tooltips, and click handling
    nodeMap = {};
    for (const n of nodes) {
      nodeMap[n.name] = n;
    }

    const option: echartsType.EChartsOption = {
      backgroundColor: "transparent",
      tooltip: {
        trigger: "item",
        backgroundColor: tooltipBg,
        borderColor: tooltipBorder,
        borderWidth: 1,
        textStyle: {
          color: tooltipText,
          fontSize: 12,
        },
        padding: [8, 12],
        extraCssText:
          "border-radius: 8px; box-shadow: 0 10px 25px -5px rgba(0, 0, 0, 0.3), 0 8px 10px -6px rgba(0, 0, 0, 0.2); backdrop-filter: blur(8px);",
        formatter: (params: unknown) => {
          if (!params || typeof params !== "object") return "";
          const p = params as {
            dataType?: string;
            name?: unknown;
            data?: unknown;
          };

          if (p.dataType === "node" && typeof p.name === "string") {
            const node = nodeMap[p.name];
            if (!node) return p.name;

            const color = node.itemStyle?.color ?? categoryStore.types[0]?.color ?? "#10b981";
            const colorDot = `<span style="display:inline-block;width:8px;height:8px;border-radius:50%;background-color:${color};margin-right:6px;"></span>`;

            if (node.level === "type") {
              const catList = categoryStore.categories.filter(
                (c) => c.type.toLowerCase() === node.type.toLowerCase(),
              );
              const subCount = catList.reduce((acc, c) => acc + c.subcategories.length, 0);
              return `
                <div style="font-weight: 600; margin-bottom: 4px; display: flex; align-items: center;">
                  ${colorDot}${node.displayName}
                </div>
                <div style="font-size: 11px; color: ${tooltipMuted}; line-height: 1.5;">
                  <div>Type: <strong style="color:${tooltipStrong}">${node.type}</strong></div>
                  <div>Categories: <strong style="color:${tooltipStrong}">${catList.length}</strong></div>
                  <div>Subcategories: <strong style="color:${tooltipStrong}">${subCount}</strong></div>
                </div>
              `;
            }

            if (node.level === "category") {
              const cat = categoryStore.categories.find(
                (c) =>
                  c.name.toLowerCase() === node.displayName.toLowerCase() &&
                  c.type.toLowerCase() === node.type.toLowerCase(),
              );
              const subCount = cat ? cat.subcategories.length : 0;
              return `
                <div style="font-weight: 600; margin-bottom: 4px; display: flex; align-items: center;">
                  ${colorDot}${node.displayName}
                </div>
                <div style="font-size: 11px; color: ${tooltipMuted}; line-height: 1.5;">
                  <div>Type: <strong style="color:${tooltipStrong}">${node.type}</strong></div>
                  <div>Subcategories: <strong style="color:${tooltipStrong}">${subCount}</strong></div>
                </div>
              `;
            }

            if (node.level === "subcategory") {
              return `
                <div style="font-weight: 600; margin-bottom: 4px; display: flex; align-items: center;">
                  ${colorDot}${node.displayName}
                </div>
                <div style="font-size: 11px; color: ${tooltipMuted}; line-height: 1.5;">
                  <div>Type: <strong style="color:${tooltipStrong}">${node.type}</strong></div>
                  <div>Category: <strong style="color:${tooltipStrong}">${node.categoryName ?? "—"}</strong></div>
                </div>
              `;
            }
          }

          if (p.dataType === "edge" && typeof p.data === "object" && p.data !== null) {
            const edge = p.data as Partial<SankeyLinkData>;
            if (typeof edge.source === "string" && typeof edge.target === "string") {
              const sourceNode = nodeMap[edge.source];
              const targetNode = nodeMap[edge.target];

              if (!sourceNode || !targetNode) return "";

              // Arm 1: Type -> Category
              if (sourceNode.level === "type" && targetNode.level === "category") {
                const cat = categoryStore.categories.find(
                  (c) =>
                    c.name.toLowerCase() === targetNode.displayName.toLowerCase() &&
                    c.type.toLowerCase() === sourceNode.type.toLowerCase(),
                );
                const subCount = cat ? cat.subcategories.length : 0;
                const color =
                  sourceNode.itemStyle?.color ?? categoryStore.types[0]?.color ?? "#10b981";
                const colorDot = `<span style="display:inline-block;width:8px;height:8px;border-radius:50%;background-color:${color};margin-right:6px;"></span>`;

                return `
                  <div style="font-weight: 600; margin-bottom: 4px; display: flex; align-items: center;">
                    ${colorDot}${sourceNode.displayName} &rarr; ${targetNode.displayName}
                  </div>
                  <div style="font-size: 11px; color: ${tooltipMuted};">
                    Subcategories: <strong style="color:${tooltipStrong}">${subCount}</strong>
                  </div>
                `;
              }

              // Arm 2: Category -> Subcategory
              if (sourceNode.level === "category" && targetNode.level === "subcategory") {
                const color =
                  targetNode.itemStyle?.color ??
                  sourceNode.itemStyle?.color ??
                  categoryStore.types[0]?.color ??
                  "#10b981";
                const colorDot = `<span style="display:inline-block;width:8px;height:8px;border-radius:50%;background-color:${color};margin-right:6px;"></span>`;

                return `
                  <div style="font-weight: 600; margin-bottom: 4px; display: flex; align-items: center;">
                    ${colorDot}${sourceNode.displayName} &rarr; ${targetNode.displayName}
                  </div>
                  <div style="font-size: 11px; color: ${tooltipMuted};">
                    Subcategory mapped directly under category
                  </div>
                `;
              }
            }
          }

          return "";
        },
      },
      series: [
        {
          type: "sankey",
          emphasis: {
            focus: "adjacency",
            itemStyle: {
              shadowBlur: 8,
              opacity: 0.95,
            },
            lineStyle: {
              opacity: 0.65,
              shadowBlur: 6,
            },
          },
          nodeAlign: "left",
          nodeGap: 18,
          nodeWidth: 16,
          draggable: false,
          orient: "horizontal",
          top: "5%",
          bottom: "5%",
          left: "3%",
          right: "12%",
          data: nodes.map((n) => ({
            name: n.name,
            value: n.value,
            depth: n.depth,
            itemStyle: n.itemStyle,
            label: {
              color: labelColor,
              fontSize: 12,
              fontWeight: n.level === "type" ? "bold" : n.level === "category" ? 500 : 400,
              fontFamily: "Inter, system-ui, sans-serif",
              formatter: () => n.displayName,
              distance: 8,
            },
          })),
          links: links.map((l) => ({
            source: l.source,
            target: l.target,
            value: l.value,
            lineStyle: l.lineStyle,
          })),
          lineStyle: {
            curveness: 0.5,
          },
        },
      ],
    };

    chartInstance.setOption(option, true);
  }

  function handleNodeClick(nodeId: string) {
    const node = nodeMap[nodeId];
    if (node?.entity) {
      if (onSelectNode) onSelectNode(node.entity);
      categoryStore.setSelectedNode(node.entity);
    }
  }

  const minChartHeight = $derived(
    Math.max(560, (categoryStore.categories.length + categoryStore.types.length) * 36),
  );

  onMount(() => {
    void renderChart();

    const handleResize = () => {
      chartInstance?.resize();
    };

    window.addEventListener("resize", handleResize);

    let resizeObserver: ResizeObserver | null = null;
    if (typeof ResizeObserver !== "undefined" && chartContainer) {
      resizeObserver = new ResizeObserver(() => {
        chartInstance?.resize();
      });
      resizeObserver.observe(chartContainer);
    }

    return () => {
      resizeObserver?.disconnect();
      window.removeEventListener("resize", handleResize);
      chartInstance?.dispose();
      chartInstance = null;
    };
  });

  $effect(() => {
    // Re-render chart on activeFilter, theme changes, or store version updates
    const _f = activeFilter;
    const _t = themeStore.mode;
    const _v = categoryStore.version;
    if (_f !== undefined && _t !== undefined && _v !== undefined) {
      void renderChart();
    }
  });
</script>

<div
  bind:this={chartContainer}
  class="size-full min-h-140 min-w-175"
  style="min-height: {minChartHeight}px;"
></div>
