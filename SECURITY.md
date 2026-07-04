# Security Policy

## What WTHIROMC is (and isn't)

WTHIROMC is a **visibility and explanation tool**. It reads system state (running processes, startup entries, digital signatures) and explains it in plain English. It is:

- **Not** an antivirus or anti-malware product
- **Not** a replacement for Windows Defender, EDR, or professional security tools
- **Not** capable of removing, quarantining, or blocking anything

It never modifies your system. Everything it does is read-only.

## Reporting a vulnerability

If you find a security issue **in WTHIROMC itself** — for example:

- A bug that could let malware evade detection or hide from the tool
- A privilege escalation issue (WTHIROMC running with more access than it should, or being tricked into running unintended code)
- A way for a malicious process to feed WTHIROMC false information that causes it to mislead the user
- Any vulnerability in how the installer or release artifacts are built/distributed

**Please do not open a public GitHub issue for these.** Instead, open a [private security advisory](../../security/advisories/new) on this repository, or contact the maintainer directly (see repository profile for contact info).

Please include:
- A description of the issue and its potential impact
- Steps to reproduce, if possible
- Your Windows version and WTHIROMC version

I'll acknowledge reports as promptly as I can. This is a small open-source project maintained outside of full-time work, so response times may vary — but security reports are taken seriously and prioritized over feature work.

## What's *not* a security vulnerability (please use a normal issue instead)

- False positives / false negatives in the rule engine — these are accuracy issues, not vulnerabilities. Open a regular issue.
- Missing detection capabilities (e.g. "it doesn't detect X kind of malware") — WTHIROMC is explicitly not trying to be a comprehensive malware scanner; see the [README's Known Limitations](./README.md#known-limitations) and [Roadmap](./ROADMAP.md).
- The tool flagging itself or other unsigned developer tools as risky — this is expected behavior for unsigned binaries, not a bug.

## Verifying release downloads

Starting with the CI/release pipeline (see `.github/workflows/release.yml`), every release includes a `SHA256SUMS.txt` file alongside the installer. To verify your download:

```powershell
Get-FileHash .\WTHIROMC_x.x.x_x64-setup.exe -Algorithm SHA256
```

Compare the output against the corresponding line in `SHA256SUMS.txt` from the same release. If they don't match, don't run the installer — re-download it, and if the mismatch persists, open an issue.

**Note:** the installer is currently **unsigned** (no code-signing certificate). Windows SmartScreen may warn about this. Checksum verification confirms the file matches what CI built from the public source — it does not substitute for code signing. See the README for more on why WTHIROMC is unsigned for now.
