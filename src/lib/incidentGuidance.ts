import { RiskLevel } from "../types/explained";


export function getIncidentGuidance(level: RiskLevel): string[] | null {
  if (level === "Red") {
    return [
      "A high score means this is worth a closer look, not a confirmed infection, WTHIROMC works from signals, not certainty.",
      "Search the exact program name and publisher online, or upload the file to virustotal.com for a second opinion from multiple antivirus engines at once.",
      "If you don't recognize this program or didn't install it, run a full scan with your existing antivirus (e.g. Windows Defender), not just a quick scan.",
      "If it's a startup item you want to stop, you can disable it yourself via Task Manager's Startup tab, the Task Scheduler app, or by removing the shortcut/registry entry, WTHIROMC only explains what it finds, it doesn't remove anything for you.",
      "If you're not comfortable handling this yourself, ask a trusted IT-savvy friend or a professional.",
    ];
  }

  if (level === "Black") {
    return [
      "This combination of signals is unusual enough to take seriously. Consider disconnecting this device from the internet until you've looked into it further.",
      "Avoid entering passwords or other sensitive information on this device until you've run a full antivirus scan.",
      "Search the exact program name and publisher online, or upload the file to virustotal.com for a second opinion from multiple antivirus engines at once.",
      "Run a full scan with your existing antivirus (e.g. Windows Defender), not just a quick scan.",
      "If you're not comfortable handling this yourself, contact a professional. A potentially compromised device is worth taking seriously.",
    ];
  }

  return null;
}

