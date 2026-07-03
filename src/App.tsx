import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ExplainedProcess } from "./types/explained";
import ProcessList from "./components/ProcessList";
import ProcessDetail from "./components/ProcessDetail";

function summarize(processes: ExplainedProcess[]) {
  const safe = processes.filter((p) => p.riskLevel === "Green").length;
  const warning = processes.filter(
    (p) => p.riskLevel === "Yellow" || p.riskLevel === "Orange"
  ).length;
  const critical = processes.filter(
    (p) => p.riskLevel === "Red" || p.riskLevel === "Black"
  ).length;
  return { safe, warning, critical, total: processes.length };
}

export default function App() {
  const [processes, setProcesses] = useState<ExplainedProcess[]>([]);
  const [selectedPid, setSelectedPid] = useState<number | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const loadProcesses = useCallback(async () => {
    try {
      setError(null);
      const result = await invoke<ExplainedProcess[]>("scan_processes");
      setProcesses(result);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadProcesses();
    const interval = setInterval(loadProcesses, 5000);
    return () => clearInterval(interval);
  }, [loadProcesses]);

  const selected = processes.find((p) => p.pid === selectedPid) ?? null;
  const summary = summarize(processes);

  return (
    <div className="h-screen flex flex-col">
      <header className="border-b border-neutral-800 px-4 py-3 flex items-center justify-between">
        <div>
          <h1 className="text-sm font-semibold tracking-wide">WTHIROMC</h1>
          <p className="text-xs text-neutral-500">What The Hell Is Running On My Computer</p>
        </div>
        <div className="flex items-center gap-4">
          {!loading && (
            <p className="text-xs text-neutral-400">
              <span className="text-neutral-500">{summary.total} processes</span>
              {summary.warning > 0 && (
                <span className="text-risk-yellow ml-2">{summary.warning} to review</span>
              )}
              {summary.critical > 0 && (
                <span className="text-risk-red ml-2">{summary.critical} critical</span>
              )}
            </p>
          )}
          <button
            onClick={loadProcesses}
            className="text-xs rounded-md border border-neutral-700 px-2.5 py-1 hover:bg-neutral-900"
          >
            Rescan
          </button>
        </div>
      </header>

      {error && (
        <div className="bg-risk-red/10 text-risk-red text-xs px-4 py-2 border-b border-neutral-800">
          Failed to scan processes: {error}
        </div>
      )}

      <main className="flex flex-1 overflow-hidden">
        <div className="flex-1 overflow-y-auto border-r border-neutral-800">
          {loading ? (
            <p className="p-4 text-sm text-neutral-500">Scanning...</p>
          ) : (
            <ProcessList
              processes={processes}
              selectedPid={selectedPid}
              onSelect={setSelectedPid}
            />
          )}
        </div>
        <div className="w-96 overflow-y-auto">
          <ProcessDetail process={selected} />
        </div>
      </main>
    </div>
  );
}
