import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ExplainedProcess, ExplainedPersistence, RiskLevel } from "./types/explained";
import ProcessList from "./components/ProcessList";
import ProcessDetail from "./components/ProcessDetail";
import StartupList, { entryKey } from "./components/StartupList";
import StartupDetail from "./components/StartupDetail";

type Tab = "processes" | "startup";

function summarize(items: { riskLevel: RiskLevel }[]) {
  const warning = items.filter(
    (p) => p.riskLevel === "Yellow" || p.riskLevel === "Orange"
  ).length;
  const critical = items.filter(
    (p) => p.riskLevel === "Red" || p.riskLevel === "Black"
  ).length;
  return { warning, critical, total: items.length };
}

export default function App() {
  const [tab, setTab] = useState<Tab>("processes");

  const [processes, setProcesses] = useState<ExplainedProcess[]>([]);
  const [selectedPid, setSelectedPid] = useState<number | null>(null);
  const [processesLoading, setProcessesLoading] = useState(true);
  const [processesError, setProcessesError] = useState<string | null>(null);

  const [startupItems, setStartupItems] = useState<ExplainedPersistence[]>([]);
  const [selectedStartupKey, setSelectedStartupKey] = useState<string | null>(null);
  const [startupLoading, setStartupLoading] = useState(true);
  const [startupError, setStartupError] = useState<string | null>(null);

  const loadProcesses = useCallback(async () => {
    try {
      setProcessesError(null);
      const result = await invoke<ExplainedProcess[]>("scan_processes");
      setProcesses(result);
    } catch (e) {
      setProcessesError(String(e));
    } finally {
      setProcessesLoading(false);
    }
  }, []);

  const loadStartupItems = useCallback(async () => {
    try {
      setStartupError(null);
      const result = await invoke<ExplainedPersistence[]>("scan_startup_items");
      setStartupItems(result);
    } catch (e) {
      setStartupError(String(e));
    } finally {
      setStartupLoading(false);
    }
  }, []);

  // Running processes change constantly, so poll every 5s.
  useEffect(() => {
    loadProcesses();
    const interval = setInterval(loadProcesses, 5000);
    return () => clearInterval(interval);
  }, [loadProcesses]);

  // Startup items rarely change - load once, let the user hit Rescan.
  useEffect(() => {
    loadStartupItems();
  }, [loadStartupItems]);

  // Keyboard shortcuts: Tab switches views, Escape clears the selection.
  // Skipped while an input/button has focus so it doesn't fight normal
  // browser tabbing through interactive elements.
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      const isInteractive = ["INPUT", "TEXTAREA", "BUTTON"].includes(target.tagName);

      if (e.key === "Tab" && !isInteractive) {
        e.preventDefault();
        setTab((t) => (t === "processes" ? "startup" : "processes"));
      } else if (e.key === "Escape") {
        setSelectedPid(null);
        setSelectedStartupKey(null);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, []);

  const selectedProcess = processes.find((p) => p.pid === selectedPid) ?? null;
  const selectedStartup =
    startupItems.find((e) => entryKey(e) === selectedStartupKey) ?? null;

  const isProcessesTab = tab === "processes";
  const activeSummary = summarize(isProcessesTab ? processes : startupItems);
  const activeLoading = isProcessesTab ? processesLoading : startupLoading;
  const activeError = isProcessesTab ? processesError : startupError;
  const activeRescan = isProcessesTab ? loadProcesses : loadStartupItems;

  return (
    <div className="h-screen flex flex-col">
      <header className="border-b border-neutral-800 px-4 py-3 flex items-center justify-between">
        <div>
          <h1 className="text-sm font-semibold tracking-wide">WTHIROMC</h1>
          <p className="text-xs text-neutral-500">What The Hell Is Running On My Computer</p>
        </div>
        <div className="flex items-center gap-4">
          {!activeLoading && (
            <p className="text-xs text-neutral-400">
              <span className="text-neutral-500">{activeSummary.total} items</span>
              {activeSummary.warning > 0 && (
                <span className="text-risk-yellow ml-2">{activeSummary.warning} to review</span>
              )}
              {activeSummary.critical > 0 && (
                <span className="text-risk-red ml-2">{activeSummary.critical} critical</span>
              )}
            </p>
          )}
          <button
            onClick={activeRescan}
            className="text-xs rounded-md border border-neutral-700 px-2.5 py-1 hover:bg-neutral-900"
          >
            Rescan
          </button>
        </div>
      </header>

      <nav className="border-b border-neutral-800 px-4 flex gap-4">
        <button
          onClick={() => setTab("processes")}
          className={`text-xs py-2 border-b-2 transition-colors ${
            isProcessesTab
              ? "border-neutral-100 text-neutral-100"
              : "border-transparent text-neutral-500 hover:text-neutral-300"
          }`}
        >
          Running Processes
        </button>
        <button
          onClick={() => setTab("startup")}
          className={`text-xs py-2 border-b-2 transition-colors ${
            !isProcessesTab
              ? "border-neutral-100 text-neutral-100"
              : "border-transparent text-neutral-500 hover:text-neutral-300"
          }`}
        >
          Startup Items
        </button>
      </nav>

      {activeError && (
        <div className="bg-risk-red/10 text-risk-red text-xs px-4 py-2 border-b border-neutral-800">
          Failed to scan: {activeError}
        </div>
      )}

      <main className="flex flex-1 overflow-hidden">
        <div className="flex-1 overflow-y-auto border-r border-neutral-800">
          {isProcessesTab ? (
            processesLoading ? (
              <p className="p-4 text-sm text-neutral-500">Scanning...</p>
            ) : (
              <ProcessList
                processes={processes}
                selectedPid={selectedPid}
                onSelect={setSelectedPid}
              />
            )
          ) : startupLoading ? (
            <p className="p-4 text-sm text-neutral-500">Scanning...</p>
          ) : (
            <StartupList
              entries={startupItems}
              selectedKey={selectedStartupKey}
              onSelect={setSelectedStartupKey}
            />
          )}
        </div>
        <div className="w-96 overflow-y-auto">
          {isProcessesTab ? (
            <ProcessDetail process={selectedProcess} />
          ) : (
            <StartupDetail entry={selectedStartup} />
          )}
        </div>
      </main>
    </div>
  );
}

