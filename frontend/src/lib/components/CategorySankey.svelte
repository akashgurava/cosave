<script lang="ts">
  import { onMount } from "svelte";
  import type * as echartsType from "echarts";
  import {
    categoryStore,
    type SankeyNodeData,
    type SankeyLinkData,
    type SelectedCategoryNode,
  } from "$lib/categories";
  import { themeStore } from "$lib/theme";

  interface Props {
    activeFilter?: string | string[];
    onSelectNode?: (node: SelectedCategoryNode) => void;
  }

  let { activeFilter = "All", onSelectNode }: Props = $props();

  let chartContainer: HTMLDivElement | null = $state(null);
  let chartInstance: echartsType.ECharts | null = null;
  let echartsCore: typeof import("$lib/echarts-sankey").default | null = null;

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
        if (params.dataType === "node") {
          const rawName = params.name as string;
          handleNodeClick(rawName);
        } else if (params.dataType === "edge") {
          const edge = params.data as SankeyLinkData;
          if (edge?.target) {
            handleNodeClick(edge.target);
          }
        }
      });
    }

    const isDark = themeStore.isDark;
    const labelColor = isDark ? "#e2e8f0" : "#1e293b";
    const tooltipBg = isDark ? "rgba(9, 9, 11, 0.95)" : "rgba(255, 255, 255, 0.95)";
    const tooltipBorder = isDark ? "#27272a" : "#e4e4e7";
    const tooltipText = isDark ? "#f4f4f5" : "#09090b";

    const { nodes, links } = categoryStore.getSankeyData(activeFilter);

    // Build map of nodeId to node details for labels and tooltips
    const nodeMap: Record<string, SankeyNodeData> = {};
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
          const p = params as {
            dataType: string;
            name: string;
            data: SankeyLinkData | SankeyNodeData;
          };

          if (p.dataType === "node") {
            const node = nodeMap[p.name];
            if (!node) return p.name;

            const color = node.itemStyle?.color ?? "#10b981";
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
                <div style="font-size: 11px; color: ${isDark ? "#a1a1aa" : "#71717a"}; line-height: 1.5;">
                  <div>Type: <strong style="color:${isDark ? "#f4f4f5" : "#18181b"}">${node.type}</strong></div>
                  <div>Categories: <strong style="color:${isDark ? "#f4f4f5" : "#18181b"}">${catList.length}</strong></div>
                  <div>Subcategories: <strong style="color:${isDark ? "#f4f4f5" : "#18181b"}">${subCount}</strong></div>
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
                <div style="font-size: 11px; color: ${isDark ? "#a1a1aa" : "#71717a"}; line-height: 1.5;">
                  <div>Type: <strong style="color:${isDark ? "#f4f4f5" : "#18181b"}">${node.type}</strong></div>
                  <div>Subcategories: <strong style="color:${isDark ? "#f4f4f5" : "#18181b"}">${subCount}</strong></div>
                </div>
              `;
            }

            if (node.level === "subcategory") {
              return `
                <div style="font-weight: 600; margin-bottom: 4px; display: flex; align-items: center;">
                  ${colorDot}${node.displayName}
                </div>
                <div style="font-size: 11px; color: ${isDark ? "#a1a1aa" : "#71717a"}; line-height: 1.5;">
                  <div>Type: <strong style="color:${isDark ? "#f4f4f5" : "#18181b"}">${node.type}</strong></div>
                  <div>Category: <strong style="color:${isDark ? "#f4f4f5" : "#18181b"}">${node.categoryName ?? "—"}</strong></div>
                </div>
              `;
            }
          }

          if (p.dataType === "edge") {
            const edge = p.data as SankeyLinkData;
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
              const color = sourceNode.itemStyle?.color ?? "#10b981";
              const colorDot = `<span style="display:inline-block;width:8px;height:8px;border-radius:50%;background-color:${color};margin-right:6px;"></span>`;

              return `
                <div style="font-weight: 600; margin-bottom: 4px; display: flex; align-items: center;">
                  ${colorDot}${sourceNode.displayName} &rarr; ${targetNode.displayName}
                </div>
                <div style="font-size: 11px; color: ${isDark ? "#a1a1aa" : "#71717a"};">
                  Subcategories: <strong style="color:${isDark ? "#f4f4f5" : "#18181b"}">${subCount}</strong>
                </div>
              `;
            }

            // Arm 2: Category -> Subcategory
            if (sourceNode.level === "category" && targetNode.level === "subcategory") {
              const color = targetNode.itemStyle?.color ?? sourceNode.itemStyle?.color ?? "#10b981";
              const colorDot = `<span style="display:inline-block;width:8px;height:8px;border-radius:50%;background-color:${color};margin-right:6px;"></span>`;

              return `
                <div style="font-weight: 600; margin-bottom: 4px; display: flex; align-items: center;">
                  ${colorDot}${targetNode.type} &rarr; ${sourceNode.displayName} &rarr; ${targetNode.displayName}
                </div>
              `;
            }

            return "";
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
          draggable: true,
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
    if (nodeId.startsWith("type:")) {
      const typeName = nodeId.replace("type:", "");
      const foundType = categoryStore.getType(typeName);
      const node: SelectedCategoryNode = {
        id: foundType ? foundType.id : nodeId,
        type: typeName,
        kind: "type",
        name: typeName,
      };
      if (onSelectNode) onSelectNode(node);
      categoryStore.setSelectedNode(node);
    } else if (nodeId.startsWith("cat:")) {
      const catId = nodeId.replace("cat:", "");
      const cat = categoryStore.categories.find((c) => c.id === catId);
      if (cat) {
        const node: SelectedCategoryNode = {
          id: cat.id,
          type: cat.type,
          kind: "category",
          name: cat.name,
          parentName: cat.type,
        };
        if (onSelectNode) onSelectNode(node);
        categoryStore.setSelectedNode(node);
      }
    } else if (nodeId.startsWith("sub:")) {
      const parts = nodeId.split(":");
      const catId = parts[1];
      const subId = parts[2];
      const cat = categoryStore.categories.find((c) => c.id === catId);
      if (cat) {
        const sub = cat.subcategories.find((s) => s.id === subId);
        if (sub) {
          const node: SelectedCategoryNode = {
            id: sub.id,
            type: cat.type,
            kind: "subcategory",
            name: sub.name,
            parentName: cat.name,
            categoryId: cat.id,
          };
          if (onSelectNode) onSelectNode(node);
          categoryStore.setSelectedNode(node);
        }
      }
    }
  }

  onMount(() => {
    void renderChart();

    const handleResize = () => {
      chartInstance?.resize();
    };

    window.addEventListener("resize", handleResize);

    return () => {
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

<div bind:this={chartContainer} class="size-full"></div>
