<div align="center">

# Test results

**Measured behavior, with the limits left visible.**

[Home](../README.md) · [Build and reproduce](building.md) · [Changelog](../CHANGELOG.md)

</div>

---

Measured on 28 October 2026. The updated 0.0.3 downloads use the lower-memory drawing path described first. The earlier release measurements are retained below for comparison.

## Updated 0.0.3: lower memory and correct selection

| Workload | Memory used by the app |
| :--- | :--- |
| Fresh window, idle | 32.21 MiB |
| Scrolling, renaming 300 files, then undoing | 42.72 MiB peak |
| Interface test with 10,002 files | 88.34 MiB peak |
| Earlier DX12 window, idle | 101.62 MiB |

These are working-set measurements from one development PC. MiB is a memory unit; other computers and workloads can differ. **Good idle results do not prove that active work is fast or uses little memory.** The rename/undo test checked all 300 restored names and file contents.

The earlier GPU-based results below do not describe the new default drawing path. A fresh-profile comparison on the same Windows PC measured about 32.21 MiB working set and 18.53 MiB private memory for the CPU pixel-buffer path, with 0% late idle CPU across the measured samples. The previous DX12 path measured about 101.62 MiB working set; an OpenGL trial measured 139.75 MiB and was rejected as the default.

The new default does not create a GPU device. Full frame drawing averaged 2.76 ms, with a 4.99 ms 95th percentile in the two-logical-processor smoke test. The cold first frame peaked at 185.56 ms. There was one idle repaint in the 1.8-second check. These are local measurements, not measurements from the user's Ryzen laptop. The laptop still needs its own verification.

Follow-up active measurements: the 10,002-file UI workload peaked at 88.34 MiB working set and 73.28 MiB sampled private allocation, with 5.05 ms 95th-percentile full drawing frames. The cold worst frame was 230.91 ms. Interactive scrolling and a real 300-file Rename/Undo button workflow peaked at 42.72 MiB working set and 23.87 MiB sampled private allocation. All original fixture names and contents were restored. Active CPU had short bursts; it must not be described as zero based on the idle sample. These tests used separate disposable files and history.

Explorer-imported files stay checked and highlighted even when a file cannot be identified as an episode or movie. Checkmarks survive refreshes and keep manual edits. Rename eligibility still prevents unchanged or invalid targets from becoming operations. Task Manager's executable description is exactly Anemorph.

The latest patch passes **74 workspace tests**, with Clippy warnings treated as errors. A separate-process test verifies that later launches forward selections instead of acquiring another primary lock. This patch's handoff checks were automated without desktop control; they are not a fresh end-to-end Explorer UI test on every Windows setup.

---

<details>
<summary><strong>Earlier 0.0.3 measurements and test conditions</strong></summary>

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

</details>

---

## How the app keeps work bounded

- Scan and large-preview work run in background workers; at most one preview is active, and stale results are discarded.
- File rows are virtualized. The full directory is not rebuilt into UI widgets each frame.
- Idle rendering is event-driven; animation transitions request frames only while active.
- The current default draws into a CPU pixel buffer without creating a GPU device. Earlier graphics paths remain available for diagnosis; `--software-rendering` selects the older DX12 WARP path.
- Language dictionaries and Arabic shaping are prepared once. Only the selected script's installed font is loaded; CJK fonts are not retained by default for English users.
- Long translated settings wrap and action buttons can wrap at small sizes.

## Correctness and interface checks

The earlier release passed 70 workspace tests; subsequent patches raised the total to 74. Coverage includes separate-process activation, cross-format index rejection, font glyph coverage for Cyrillic/CJK/Arabic, joined Arabic text and preserved numeric placeholders, selection without unnecessary renames, rename/undo recovery, bounded input, and integration timeouts. Earlier automated screenshots covered Home, Settings, Configuration, the minimum window, About, filters, and the episode marker dialog. Arabic and Japanese UI screenshots were also inspected.

## Security review

Only filenames are edited; video data is not decoded. Format indexes are parsed from a fixed allowlist. The episode and movie allowlists are separate. Settings, media markers, selection handoffs, and undo inputs have bounded reads. Explorer handoffs are restricted to the per-user request directory, reject symbolic links, validate containers, and are consumed once. Directory scans do not follow symlinks. The batch validates destinations and collisions before changes and keeps a recovery journal. Helper commands use explicit executable paths and argument lists. Menu hosts inherit no parent handles and never run inside Explorer. Repair and uninstall do not force Explorer to restart.

The earlier release's [OSV API](https://google.github.io/osv.dev/post-v1-querybatch/) check covered all 351 registry packages in that lockfile. A follow-up check after adding the pixel-buffer renderer covered 377 registry packages, with the same four informational maintenance advisories and no vulnerability reports returned:

- [ttf-parser: RUSTSEC-2026-0192](https://rustsec.org/advisories/RUSTSEC-2026-0192.html): unmaintained font parser used by the current UI dependency chain. The app reads installed Windows fonts and its bundled fallback fonts, not arbitrary font uploads. Replacing this upstream dependency needs a separate renderer migration.
- [zbus_names: RUSTSEC-2026-0371](https://rustsec.org/advisories/RUSTSEC-2026-0371.html), [zvariant: RUSTSEC-2026-0370](https://rustsec.org/advisories/RUSTSEC-2026-0370.html), and [zvariant_derive: RUSTSEC-2026-0372](https://rustsec.org/advisories/RUSTSEC-2026-0372.html): standalone crates are unmaintained after merging upstream. These are Linux-side dependencies in the cross-platform lockfile, not the Windows runtime.

Advisory checks cannot establish the absence of undiscovered vulnerabilities. The local development certificate remains opt-in; Windows security policy is not weakened. A different user can retain the classic context menu without trusting it.

## Reproduce

`Anemorph.exe --benchmark <isolated-directory>` writes fixture measurements and repeated-preview memory samples.

`Anemorph.exe --smoke-test <isolated-directory>` exercises the current default drawing path and records `graphics.json` and `smoke.json`. Add `--software-rendering` only to compare the earlier DX12 WARP path. Restrict only the disposable test process to affinity 0x3 if checking two-thread behavior. Never point these commands at a real media folder.

---

[Build guide](building.md) · [Report a problem](https://github.com/yaboirar/Anemorph/issues) · [Back to Anemorph](../README.md)
