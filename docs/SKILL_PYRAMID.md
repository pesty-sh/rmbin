# Skill pyramid

Read modules from the bottom of this tree upward. Each layer uses the one below it. The function numbers are the roadmap ids in [`planning/ROADMAP.md`](planning/ROADMAP.md). 0.0.1 stops at F14.

```text
L6  Concurrency, signals, long jobs, networking      → later: free-space wipe, server cleanup
└─ L5  Handles and ownership, race tests             → later
   └─ L4  Traits and platform cfg                    → later: rename, one filesystem in full
      ├─ L4  Serde and hashing                       → later: JSON plan
      └─ L3  Error discipline, fail closed           → F14 report, F06 banner
         ├─ L2  Walking trees                        → F08 plan, F09 dry run, F10 confirm
         └─ L2  File I/O: OpenOptions, Write, sync   → F11 overwrite, F12 truncate, F13 delete
            ├─ L1  CLI args                          → F01 flags, F02 list split, F03 pattern
            ├─ L1  std::fs metadata, no follow       → F05 classify, F07 protected paths
            └─ L1  Tests
               └─ L0  Cargo, strings, Result, Path
```

```mermaid
graph BT
  L0A["L0 Cargo, crates, fn main"] --> L1A
  L0B["L0 Strings and slices"] --> L1A
  L0C["L0 Result and Error"] --> L1A
  L0D["L0 Paths"] --> L1B
  L1A["L1 CLI args — F01 F02 F03"] --> L2A
  L1B["L1 metadata, no follow — F05 F07"] --> L2A
  L1C["L1 Tests"] --> L2A
  L2A["L2 Walking trees — F08 F09 F10"] --> L3A
  L2B["L2 File I/O sync_all — F11 F12 F13"] --> L3A
  L1B --> L2B
  L3A["L3 Fail closed — F06 F14"]
```

0.0.1 teaches L0 through L3. L4 and above are named so you can see what is deliberately left out.
