# Anemorph 0.0.3 performance and security

Measured on 28 October 2026. The updated 0.0.3 downloads use the lower-memory drawing path described first. The earlier release measurements are retained below for comparison.

## Updated 0.0.3: lower memory and correct selection

The earlier GPU-based results below do not describe the new default drawing path. A fresh-profile comparison on the same Windows PC measured about 32.21 MiB working set and 18.53 MiB private memory for the CPU pixel-buffer path, with 0% late idle CPU across the measured samples. The previous DX12 path measured about 101.62 MiB working set; an OpenGL trial measured 139.75 MiB and was rejected as the default.

The new default does not create a GPU device. Full frame drawing averaged 2.76 ms, with a 4.99 ms 95th percentile in the two-logical-processor smoke test. The cold first frame peaked at 185.56 ms. There was one idle repaint in the 1.8-second check. These are local measurements, not measurements from the user's Ryzen laptop. The laptop still needs its own verification.

Follow-up active measurements: the 10,002-file UI workload peaked at 88.34 MiB working set and 73.28 MiB sampled private allocation, with 5.05 ms 95th-percentile full drawing frames. The cold worst frame was 230.91 ms. Interactive scrolling and a real 300-file Rename/Undo button workflow peaked at 42.72 MiB working set and 23.87 MiB sampled private allocation. All original fixture names and contents were restored. Active CPU had short bursts; it must not be described as zero based on the idle sample. These tests used separate disposable files and history.

Explorer-imported files are now checked and highlighted, including unchanged files. Rename eligibility still prevents unchanged or invalid targets from becoming operations. Task Manager's executable description is now exactly Anemorph. 72 workspace tests passed.

## Earlier 0.0.3 build: scope and limits

The actual test process was restricted to logical processors 0 and 1 (affinity 0x3). The UI was explicitly rendered by Microsoft Basic Render Driver, a CPU adapter using DX12. This bypassed the dedicated GPU. CPU affinity restricts parallel execution; it does not emulate a slower processor, lower memory bandwidth, limited RAM, slower disks, or different display drivers. This is evidence for a constrained execution path, not certification for every low-end PC or a guarantee of 60 FPS.

## Measurements

| Workload | Result |
| --- | --- |
| Scan 2,000 disposable video filenames | 8.82 ms median |
| Preview 10,000 filenames | 36.73 ms median |
| Preview 50,000 filenames | 193.59 ms median |
| 50,000-name preview throughput | 258,278 names/second |
| Rename and undo 500 disposable files | 1070.88 ms median |
| Benchmark peak private allocation | 54.7 MiB |
| Benchmark peak working set | 61.1 MiB |
| CPU-rendered UI mean CPU work per frame | 2.37 ms |
| CPU-rendered UI peak private allocation, including screenshot captures | 166.8 MiB |
| CPU-rendered UI peak working set | 111.6 MiB |
| UI rows built at once from a 502-file list | At most 24 visible rows |
| Static CPU-rendered UI, idle sample | 3004 ms |
| CPU consumed in idle sample | 0 ms |
| Private allocation in idle sample | 151.9 to 151.9 MiB |
| Handles in idle sample | 510 to 510 |

The frame figure measures application UI construction, not total present/GPU-driver time or input-to-display latency. Screenshot capture deliberately adds CPU work to the UI smoke test.

Repeated 10,000-file previews used 17.5 MiB after initial warm-up and 20.1 MiB after thirty additional previews. The last ten samples ranged from 19.9 to 20.6 MiB. Initial growth is consistent with allocator/cache retention; later samples fluctuate within a bounded range. This workload showed no sustained linear growth, but is not a proof that all code paths are leak-free.

## Performance and memory controls

- Scan and large-preview work run in background workers; at most one preview is active, and stale results are discarded.
- File rows are virtualized. The full directory is not rebuilt into UI widgets each frame.
- Idle rendering is event-driven; animation transitions request frames only while active.
- Integrated graphics are preferred when available; a compatible CPU adapter is available as a fallback. `--software-rendering` forces CPU rendering for validation.
- Language dictionaries and Arabic shaping are prepared once. Only the selected script's installed font is loaded; CJK fonts are not retained by default for English users.
- Long translated settings wrap and action buttons can wrap at small sizes.

## Correctness and interface checks

70 workspace tests passed, with Clippy warnings treated as errors. Tests cover cross-format index rejection, font glyph coverage for Cyrillic/CJK/Arabic, joined Arabic text and preserved numeric placeholders, unchanged-file selection without rename execution, rename/undo recovery, bounded input, and integration timeouts. Automated screenshots covered Home, Settings, Configuration, the minimum window, About, filters, and the episode marker dialog. Arabic and Japanese UI screenshots were also inspected.

## Security review

Only filenames are edited; video data is not decoded. Format indexes are parsed from a fixed allowlist. The episode and movie allowlists are separate. Settings, media markers, selection handoffs, and undo inputs have bounded reads. Explorer handoffs are restricted to the per-user request directory, reject symbolic links, validate containers, and are consumed once. Directory scans do not follow symlinks. The batch validates destinations and collisions before changes and keeps a recovery journal. Helper commands use explicit executable paths and argument lists. Menu hosts inherit no parent handles and never run inside Explorer. Repair and uninstall do not force Explorer to restart.

The [OSV API](https://google.github.io/osv.dev/post-v1-querybatch/) checked all 351 registry packages in Cargo.lock. It returned no vulnerability reports, but did return four informational maintenance advisories:

- [ttf-parser: RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html): unmaintained font parser used by the current UI dependency chain. The app reads installed Windows fonts and its bundled fallback fonts, not arbitrary font uploads. Replacing this upstream dependency needs a separate renderer migration.
- [zbus_names: RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html), [zvariant: RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html), and [zvariant_derive: RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html): standalone crates are unmaintained after merging upstream. These are Linux-side dependencies in the cross-platform lockfile, not the Windows runtime.

Advisory checks cannot establish the absence of undiscovered vulnerabilities. The local development certificate remains opt-in; Windows security policy is not weakened. A different user can retain the classic context menu without trusting it.

## Reproduce

`Anemorph.exe --benchmark <isolated-directory>` writes fixture measurements and repeated-preview memory samples.

`Anemorph.exe --smoke-test <isolated-directory> --software-rendering` exercises CPU rendering and records `graphics.json` and `smoke.json`. Restrict only that disposable process to affinity 0x3. Never point these test commands at a real media folder.
