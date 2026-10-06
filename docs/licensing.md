# Licensing

Issue: #23  
Status: Active after merge  
Updated: 2026-10-07

## Project license

AI CLI Orchestrator is licensed under the **Apache License, Version 2.0**.

SPDX identifier:

```text
Apache-2.0
```

The authoritative project license text is in the repository root:

- [LICENSE](../LICENSE)
- [NOTICE](../NOTICE)

## Why Apache-2.0

The project is intended to be usable by individuals, companies, and external contributors.

Apache-2.0 was selected because it is a permissive open-source license and includes an explicit patent license from contributors for patent claims covered by their contributions.

Compared with the MIT License, Apache-2.0 is longer and imposes more redistribution/notice requirements, but the explicit patent terms are useful for a project expected to accept external contributions and enterprise use.

## Contributions

Unless a contributor explicitly states otherwise in a manner accepted by the project, an intentional contribution submitted for inclusion in this repository is provided under the Apache License 2.0 terms, consistent with Section 5 of the license.

A separate Contributor License Agreement (CLA) is **not required at this stage**.

The project may revisit CLA/DCO policy later if governance, corporate contributions, or legal requirements make it necessary.

## Third-party dependencies

Apache-2.0 applies to the AI CLI Orchestrator project's own source code, documentation, and other original project materials unless a file states otherwise.

It does **not** replace or override the licenses of third-party dependencies.

When implementation dependencies are added:

- retain required copyright/license notices;
- record license metadata;
- verify redistribution obligations before binary releases;
- generate or maintain a third-party license inventory for distributable builds;
- do not assume a dependency is compatible merely because it is available on npm, crates.io, GitHub, or another package registry.

## Provider CLIs and APIs

Codex, Claude Code, Grok, Google Antigravity, and other provider software/services remain subject to their respective vendors' licenses, terms of service, API terms, distribution rules, and usage restrictions.

AI CLI Orchestrator's Apache-2.0 license does not grant rights to:

- redistribute a provider CLI when its terms do not permit redistribution;
- copy proprietary provider code;
- bypass provider usage limits or access controls;
- use provider trademarks beyond applicable nominative/descriptive use and trademark policies;
- relicense provider software or hosted services.

Provider binaries should generally be discovered and invoked from the user's existing installation rather than bundled, unless a future legal/technical review confirms redistribution is allowed.

## Trademarks

Names such as OpenAI, Codex, Anthropic, Claude, xAI, Grok, Google, and Antigravity may be trademarks of their respective owners.

Their mention in this project is descriptive and does not imply endorsement, sponsorship, or affiliation.

The Apache-2.0 license itself does not grant trademark rights.

## NOTICE

The project includes a root `NOTICE` file.

When redistribution obligations require preservation of NOTICE information, downstream distributors should retain the applicable notices as required by Apache-2.0.

Third-party notices may be added to NOTICE or to a generated third-party notices artifact when dependencies and binary distribution are introduced.

## Package metadata

When package manifests are introduced, use:

```text
license = "Apache-2.0"
```

or the ecosystem-equivalent SPDX field.

Examples include:
- Rust `Cargo.toml`
- npm `package.json`
- installer/update metadata
- release/SBOM metadata

## Binary releases

Before the first public binary release, create a release licensing check that covers:

1. dependency license inventory;
2. required attribution/NOTICE files;
3. bundled assets/fonts/icons;
4. provider binary redistribution rules;
5. installer/updater third-party components;
6. SBOM generation;
7. source offer/notice requirements for any dependency whose license requires them.

## References

- Apache License 2.0: https://www.apache.org/licenses/LICENSE-2.0.html
- Applying Apache License 2.0: https://www.apache.org/legal/apply-license
- SPDX identifier: `Apache-2.0`
- MIT License reference considered during selection: https://opensource.org/license/mit

## Disclaimer

This document records the project's licensing policy and engineering decisions. It is not individualized legal advice.
