import { ExplainedProcess, ExplainedPersistence } from "../types/explained";

function downloadBlob(content: string, mime: string, filename: string) {
  const blob = new Blob([content], { type: mime });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function timestampForFilename(): string {
  return new Date().toISOString().slice(0, 19).replace(/[:T]/g, "-");
}

function csvEscape(value: string): string {
  if (/[",\n]/.test(value)) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

function toCsv(header: string[], rows: string[][]): string {
  return [header, ...rows].map((row) => row.map(csvEscape).join(",")).join("\n");
}

export function exportProcessesJson(processes: ExplainedProcess[]) {
  downloadBlob(
    JSON.stringify(processes, null, 2),
    "application/json",
    `wthiromc-processes-${timestampForFilename()}.json`
  );
}

export function exportProcessesCsv(processes: ExplainedProcess[]) {
  const header = [
    "Name",
    "Publisher",
    "Risk Level",
    "Score",
    "CPU %",
    "Memory (MB)",
    "Exe Path",
    "New",
    "Marked Safe",
    "Summary",
    "Explanations",
  ];
  const rows = processes.map((p) => [
    p.name,
    p.publisher ?? "",
    p.riskLevel,
    String(p.score),
    p.cpuUsage.toFixed(1),
    (p.memoryBytes / (1024 * 1024)).toFixed(0),
    p.exePath,
    p.isNew ? "Yes" : "No",
    p.userMarkedSafe ? "Yes" : "No",
    p.summary,
    p.explanations.join(" | "),
  ]);
  downloadBlob(toCsv(header, rows), "text/csv", `wthiromc-processes-${timestampForFilename()}.csv`);
}

export function exportStartupJson(entries: ExplainedPersistence[]) {
  downloadBlob(
    JSON.stringify(entries, null, 2),
    "application/json",
    `wthiromc-startup-${timestampForFilename()}.json`
  );
}

export function exportStartupCsv(entries: ExplainedPersistence[]) {
  const header = [
    "Name",
    "Source",
    "Publisher",
    "Risk Level",
    "Score",
    "Command",
    "New",
    "Marked Safe",
    "Summary",
    "Explanations",
  ];
  const rows = entries.map((e) => [
    e.name,
    e.source,
    e.publisher ?? "",
    e.riskLevel,
    String(e.score),
    e.command,
    e.isNew ? "Yes" : "No",
    e.userMarkedSafe ? "Yes" : "No",
    e.summary,
    e.explanations.join(" | "),
  ]);
  downloadBlob(toCsv(header, rows), "text/csv", `wthiromc-startup-${timestampForFilename()}.csv`);
}

