# Environment Baseline — Recorded 2026-08-26T05:27:15+05:00

All entries below are OBSERVED (direct command output) unless marked otherwise.

## Machine
- OS: Windows, 64-bit (x86_64), timezone Asia/Karachi
- Hostname context: user `Muhammad Hamad Latif`
- Terminals used: TraeAI PowerShell sessions (3 observed)

## Tool versions (OBSERVED)
| Tool | Version | Evidence |
|---|---|---|
| git | 2.55.0.windows.3 | `git --version` |
| node | v24.19.0 | `node --version` |
| npm | 11.17.0 | `npm --version` |
| bun | NOT FOUND on PATH | `Get-Command bun` → CommandNotFound |
| pnpm | NOT FOUND on PATH | `Get-Command pnpm` → CommandNotFound |
| cargo | 1.97.1 (c980f4866 2026-06-30) | `cargo --version` |
| rustc | 1.97.1 (8bab26f4f 2026-07-14) | full-path invocation `~/.cargo/bin/rustc.exe --version` |
| python | 3.12 at `%LOCALAPPDATA%\Programs\Python\Python312\python.exe` | `Get-Command python` (version probe hung in terminal; version from path — DOCUMENTED/INFERRED for exact patch) |
| aider | aider 0.86.2 at `%LOCALAPPDATA%\...\Scripts\aider.exe` | `aider --version` |
| gstack | NOT FOUND on PATH | `Get-Command gstack` |
| omniroute | AMBIGUOUS: zero-byte file exists at `C:\Windows\system32\omniroute` (Length 0, Archive attribute). Not an executable binary. No omniroute port listening. | `Get-Item`, `Get-Content`, `netstat -an` |

## Listening ports (OBSERVED via netstat)
No OmniRoute/dev-server ports detected (135/445/5040/7680/49xxx are Windows system ports; 127.0.0.1:49844 and 51000 unidentified local services).

## Git repository state (OBSERVED)
- `E:\AI-CONDUCTOR` was **not** a git work tree at inspection time (`git rev-parse --git-dir` → exit 128).
- Git identity configured globally: name=`Muhammad Hamad Latif`, email=`muhammadlatif9292@gmail.com`.

## Repository contents (OBSERVED)
```
E:\AI-CONDUCTOR\
├── 48 HOURS\AI_CONDUCTOR_OX_ALPHA_48_HOUR_EXECUTION_CAMPAIGN_v1.0.md   (30,373 B)
├── BLUEPRINT\AI_CONDUCTOR_MASTER_BLUEPRINT_v2_4.md                    (114,742 B)
├── FAILURE_MEMORY\                                                    (5 files: BUILD_STATE_SPEC,
│     FAILURE_CORPUS_SPEC, PHASE_MANIFEST, TASK_CONTRACTS, VERIFICATION_GATES)
├── GOVERNANCE\AI_CONDUCTOR_BUILD_PROTOCOL_v1.0.md                     (41,846 B)
├── KNOWLEDGE\AI_CONDUCTOR_EXTERNAL_COMPONENTS_KNOWLEDGE_BASE_v1.0.md  (50,485 B)
└── PROJECT\                                                            (EMPTY directory)
```

## Terminal reliability notes (project-observed)
Trae terminal intermittently returns empty output or IDE timeouts for some probes.
Mitigation that works: wrap results in `Write-Output ('X=' + $value)`, use
`Get-Command`/full paths instead of bare invocations, retry on another terminal.
Recorded as failure F-ENV-0001 in BUILD_STATE.
