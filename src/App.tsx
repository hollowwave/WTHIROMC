import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ExplainedProcess } from "./types/explained";
import ProcessList from "./components/ProcessList";
import ProcessDetail from "./components/ProcessDetail";

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

  return (
    <div className="h-screen flex flex-col">
      <header className="border-b border-neutral-800 px-4 py-3 flex items-center justify-between">
        <div>
          <h1 className="text-sm font-semibold tracking-wide">WTHIROMC</h1>
          <p className="text-xs text-neutral-500">What The Hell Is Running On My Computer</p>
        </div>
        <button
          onClick={loadProcesses}
          className="text-xs rounded-md border border-neutral-700 px-2.5 py-1 hover:bg-neutral-900"
        >
          Rescan
        </button>
      </header>

      {error && (
        <div className="bg-risk-red/10 text-risk-red text-xs px-4 py-2 border-b border-neutral-800">
          Failed to scan processes: {error}
        </div>
      )}

      <main className="flex flex-1 overflow-hidden">
        <div className="w-96 overflow-y-auto border-r border-neutral-800">
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
        <div className="flex-1 overflow-y-auto">
          <ProcessDetail process={selected} />
        </div>
      </main>
    </div>
  );
}
