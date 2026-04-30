import { useMemo } from "react";
import uPlot from "uplot";
import UplotReact from "uplot-react";
import "uplot/dist/uPlot.min.css";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Activity } from "lucide-react";

interface GraphSeries {
  label: string;
  values: number[];
  stroke: string;
}

interface GraphProps {
  title: string;
  description?: string;
  time?: number[];
  series?: GraphSeries[];
}

export const LiveTestGraph = ({
  title,
  description = "Real-time DAQ feed",
  time = [],
  series = [],
}: GraphProps) => {
  const data = useMemo(
    () => [time, ...series.map((item) => item.values)] as uPlot.AlignedData,
    [time, series],
  );
  const hasData = time.length > 0 && series.some((item) => item.values.length > 0);

  const options: uPlot.Options = useMemo(
    () => ({
      width: 500,
      height: 300,
      series: [
        {},
        ...series.map((item) => ({
          label: item.label,
          stroke: item.stroke,
          width: 2,
        })),
      ],
      axes: [
        { stroke: "oklch(0.553 0.013 58.071)" },
        { stroke: "oklch(0.553 0.013 58.071)" },
      ],
      legend: {
        show: series.length > 1,
      },
    }),
    [series],
  );

  return (
    <Card className="w-full bg-zinc-950 border-zinc-800">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <div className="space-y-1">
          <CardTitle className="text-xl font-bold tracking-tight uppercase">
            {title}
          </CardTitle>
          <CardDescription>{description}</CardDescription>
        </div>
        <Badge
          variant="outline"
          className={
            hasData
              ? "text-green-500 border-green-500/20"
              : "text-muted-foreground border-zinc-700"
          }
        >
          <Activity className="w-3 h-3 mr-1" />
          {hasData ? "LIVE" : "WAITING"}
        </Badge>
      </CardHeader>
      <CardContent className="flex justify-center items-center">
        <UplotReact options={options} data={data} />
      </CardContent>
    </Card>
  );
};
