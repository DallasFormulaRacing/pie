import { BrowserRouter, Routes, Route } from "react-router-dom";
import { DaqMonitor } from "./pages/DaqMonitor";
import { BmsMonitor } from "./pages/BmsMonitor";
import { LogsPage } from "./pages/Logs";
import { ThemeProvider } from "./components/ThemeProvider";
import { Homepage } from "./pages/Homepage";

export default function App() {
  return (
    <ThemeProvider defaultTheme="dark">
        <BrowserRouter>
        <Routes>
            {/* domain.com/ shows the homepage with live graphs */}
            <Route path="/" element={<Homepage />} />
            {/* domain.com/daq shows the main dashboard */}
            <Route path="/daq" element={<DaqMonitor />} />
            {/* domain.com/bms shows the battery stats */}
            <Route path="/bms" element={<BmsMonitor />} />
            {/* domain.com/logs shows the CAN logs */}
            <Route path="/logs" element={<LogsPage />} />
        </Routes>
        </BrowserRouter>
    </ThemeProvider>
  );
}