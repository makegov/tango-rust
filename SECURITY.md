# Security Policy

## Supported versions

Only the latest minor release line of `tango-rust` receives security fixes.

| Version | Supported |
| ------- | --------- |
| 1.x     | yes       |
| 0.x     | no        |

## Reporting a vulnerability

**Please do not open a public GitHub issue for security problems.**

Email **security@makegov.com** with:

- a description of the vulnerability and its impact,
- steps to reproduce (or a proof-of-concept),
- the affected version(s) of `tango-rust`,
- any suggested mitigations.

You'll get an acknowledgement within **3 business days** and a remediation plan within **10 business days** of triage.

## Disclosure

We follow a **90-day coordinated disclosure** window from initial report. We'll work with you on credit in the release notes if you'd like it. If we agree on an earlier disclosure date for a coordinated fix across sibling SDKs (`tango-go`, `tango-node`, `tango-python`), we'll let you know.

## Out of scope

- Vulnerabilities in third-party services the SDK talks to (report those upstream).
- Issues that require physical access to the user's machine or non-default Rust toolchain misconfigurations.
- Reports from automated scanners with no proof-of-concept.
