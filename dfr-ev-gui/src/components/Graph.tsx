// src/components/Graph.tsx
import * as React from "react"
import uPlot from 'uplot';
import UplotReact from 'uplot-react';
import 'uplot/dist/uPlot.min.css';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Activity } from "lucide-react"

// Define the "contract" for what this component needs
interface GraphProps {
  title: string;
  description?: string; // The '?' means this is optional
}

export const LiveTestGraph = ({ title, description = "Real-time DAQ feed" }: GraphProps) => {
  const [data, setData] = React.useState<[number[], number[]]>([[], []]);
  
  // (Simulation logic remains the same...)
  React.useEffect(() => {
    const interval = setInterval(() => {
      const now = Date.now() / 1000;
      const randomVal = Math.floor(Math.random() * (9000 - 1000 + 1) + 1000);
      setData(prev => [
        [...prev[0], now].slice(-100),
        [...prev[1], randomVal].slice(-100)
      ]);
    }, 100);
    return () => clearInterval(interval);
  }, []);

  const options: uPlot.Options = {
    width: 500, // Reduced slightly for 2-column grid
    height: 300,
    series: [{}, { label: title, stroke: "oklch(0.646 0.222 41.116)", width: 2 }],
    axes: [{ stroke: "oklch(0.553 0.013 58.071)" }, { stroke: "oklch(0.553 0.013 58.071)" }],
  };

  return (
    <Card className="w-full bg-zinc-950 border-zinc-800">
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-4">
        <div className="space-y-1">
          {/* Use the 'title' prop here */}
          <CardTitle className="text-xl font-bold tracking-tight uppercase">
            {title}
          </CardTitle>
          <CardDescription>{description}</CardDescription>
        </div>
        <Badge variant="outline" className="text-green-500 border-green-500/20">
          <Activity className="w-3 h-3 mr-1 animate-pulse" />
          LIVE
        </Badge>
      </CardHeader>
      <CardContent className="flex justify-center items-center">
        <UplotReact options={options} data={data} />
      </CardContent>
    </Card>
  );
};
