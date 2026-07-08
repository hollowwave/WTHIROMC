import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ExplainedProcess } from "../types/explained";
import RiskBadge from "./RiskBadge";

interface Props {
  process: ExplainedProcess | null;
  /** Called after a successful mark/unmark so the parent can re-fetch the scan. */
  onSafetyChanged: () => void;
}

export default function ProcessDetail({ process, onSafetyChanged }: Props) {
  const [copied, setCopied] = useState(false);
  const [working, setWorking] = useState(false);

  if (!process) {
    return (
      <div className="flex h-full items-center justify-center text-neutral-500 text-sm">
        Select a process to see why it's flagged.
      </div>
    );
  }

  const copyPath = async () => {
    try {
      await navigator.clipboard.writeText(process.exePath);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } catch {
      // Clipboard access can fail (permissions, unsupported context) - not
      // worth surfacing an error for a convenience feature.
    }
  };

  const toggleSafe = async () => {
    setWorking(true);
    try {
      if (process.userMarkedSafe) {
        await invoke("unmark_process_safe", { exePath: process.exePath });
      } else {
        await invoke("mark_process_safe", { exePath: process.exePath, name: process.name });
      }
      onSafetyChanged();
    } catch (e) {
      console.error("Failed to update allowlist:", e);
    } finally {
      setWorking(false);
    }
  };

  return (
    <div className="p-4 space-y-4">
      <div>
        <div className="flex items-center gap-2">
          <h2 className="text-lg font-semibold">{process.name}</h2>
          <RiskBadge level={process.riskLevel} />
        </div>
        <button
          onClick={copyPath}
          title="Click to copy"
          className="text-sm text-neutral-400 hover:text-neutral-200 text-left break-all"
        >
          {process.exePath} {copied && <span className="text-risk-green">(copied)</span>}
        </button>
      </div>

      <div className="rounded-md border border-neutral-800 bg-neutral-900 p-3">
        <p className="text-sm">{process.summary}</p>
      </div>

      {process.explanations.length > 0 && (
        <div>
          <h3 className="text-xs uppercase tracking-wide text-neutral-500 mb-2">
            Why this is flagged
          </h3>
          <ul className="space-y-1.5 text-sm text-neutral-300">
            {process.explanations.map((line, i) => (
              <li key={i} className="flex gap-2">
                <span className="text-neutral-600">-</span>
                <span>{line}</span>
              </li>
            ))}
          </ul>
        </div>
      )}

      <button
        onClick={toggleSafe}
        disabled={working}
        className={`text-xs rounded-md border px-3 py-1.5 disabled:opacity-50 ${
          process.userMarkedSafe
            ? "border-neutral-700 text-neutral-400 hover:bg-neutral-900"
            : "border-risk-green/40 text-risk-green hover:bg-risk-green/10"
        }`}
      >
        {process.userMarkedSafe ? "Remove from safe list" : "Mark as safe"}
      </button>
    </div>
  );
}

