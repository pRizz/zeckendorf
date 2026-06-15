# Operability

This page captures default expectations for operational visibility in shipped user-facing products.

## Display Runtime Provenance In User-Facing Product UI

- Level: `must`
- Intent: Make it easy for users, support, and engineers to identify the exact running build during debugging without relying on source access, devtools, or guesswork.
- Rule: For user-facing, app-like websites and apps, display runtime provenance in a stable user-visible surface such as a footer, About screen, Settings page, or Help panel. At minimum, expose app version, source commit, and build identifier or build time. The visible UI may abbreviate the commit for readability, but the same surface must offer a concise copyable one-line summary that includes the exact commit when available. If any required field is unavailable, render `Unavailable` for that field instead of hiding the provenance surface or omitting the field. Do not make this information depend on devtools, hidden gestures, or non-production-only modes. A separate diagnostics endpoint is optional, not required by this rule.
- Rationale: Support and debugging are slower when the running build has to be inferred from screenshots, release notes, CI logs, or a developer's memory. Visible provenance shortens incident triage, reduces ambiguity between staged and deployed artifacts, and gives non-engineers a reliable way to report what they are actually using.
- Good example:

```text
Admin dashboard footer
- Version 2.4.1
- Commit a1b2c3d
- Build 2026-03-25T14:10Z
- "Copy build info" copies: version=2.4.1 commit=a1b2c3d4e5f678901234567890abcdef1234567 build=2026-03-25T14:10Z
```

- Good example:

```text
Desktop app About screen
- Version 5.8.0
- Commit Unavailable
- Build 2026.03.25.2
- Copy action still includes all fields and keeps `commit=Unavailable`
```

- Bad example:

```text
Production customer portal
- footer and settings show no build details
- staging shows a version badge, but production hides it
- support has to ask engineering which commit was deployed
```

- Bad example:

```text
Internal tool
- commit is only visible in browser devtools
- build number appears only after a hidden keyboard shortcut
- missing fields disappear entirely instead of showing `Unavailable`
```

- Exceptions or escape hatches: This rule applies to interactive, app-like websites and apps, including internal or admin UIs. It does not apply by default to background services, CLIs, workers, or simple brochure, marketing, or landing sites. Products without a natural footer may use an About, Settings, Help, or similar stable navigation surface instead. If a repository has a documented security, contractual, or product reason not to expose a specific field, record that decision in `standards-overrides.md` rather than silently omitting the provenance behavior.
- Review questions: Can a normal user or support person identify the running version, commit, and build from the product UI in production? If the visible commit is shortened, can the exact commit still be copied from the same surface? Do missing fields render as `Unavailable` instead of disappearing? Is the provenance shown in stable product chrome rather than devtools, hidden gestures, or environment-specific debug modes?
- Automation potential: Build pipelines and static checks can verify some metadata wiring, but visibility, placement, copy affordances, and scope judgment still require human review.
