import * as echarts from "echarts/core";
// @ts-expect-error - Subpath import for minimal bundle size and true tree-shaking
import { install as SankeyChart } from "echarts/lib/chart/sankey/install.js";
// @ts-expect-error - Subpath import for minimal bundle size and true tree-shaking
import { install as TooltipComponent } from "echarts/lib/component/tooltip/install.js";
// @ts-expect-error - Subpath import for minimal bundle size and true tree-shaking
import { install as CanvasRenderer } from "echarts/lib/renderer/installCanvasRenderer.js";

echarts.use([SankeyChart, TooltipComponent, CanvasRenderer]);

export type ECharts = typeof echarts;
export default echarts;
