//! Isolated, repeatable measurements. Never reads the user's media library.
use crate::engine::{self, Settings};
use std::{
    path::{Path, PathBuf},
    time::Instant,
};
fn median(mut run: impl FnMut(), count: usize) -> f64 {
    run();
    let mut times = Vec::new();
    for _ in 0..count {
        let start = Instant::now();
        run();
        times.push(start.elapsed().as_secs_f64() * 1000.);
    }
    times.sort_by(f64::total_cmp);
    times[count / 2]
}
pub fn run(dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(dir)?;
    let dir = &std::path::absolute(dir)?;
    let settings = Settings::default();
    let disk = dir.join("fixtures");
    for season in 1..=4 {
        let folder = disk.join(format!("S{season:02}"));
        std::fs::create_dir_all(&folder)?;
        for episode in 1..=500 {
            let p = folder.join(format!("Original S{season:02}E{episode:03}.mkv"));
            if !p.exists() {
                std::fs::write(p, b"fixture")?;
            }
        }
    }
    let scan_ms = median(
        || {
            std::hint::black_box(engine::scan(&disk, &settings).unwrap());
        },
        5,
    );
    let synthetic = dir.join("synthetic");
    let files: Vec<PathBuf> = (0..50_000)
        .map(|i| {
            synthetic.join(format!(
                "S{:02}/Original S{:02}E{:03}.mkv",
                i / 500 + 1,
                i / 500 + 1,
                i % 500 + 1
            ))
        })
        .collect();
    let preview_10k_ms = median(
        || {
            std::hint::black_box(
                engine::plan(&synthetic, "Benchmark", &files[..10_000], &settings).unwrap(),
            );
        },
        5,
    );
    let preview_50k_ms = median(
        || {
            std::hint::black_box(engine::plan(&synthetic, "Benchmark", &files, &settings).unwrap());
        },
        3,
    );
    let rename_dir = dir.join("rename");
    std::fs::create_dir_all(&rename_dir)?;
    let changes: Vec<_> = (0..500)
        .map(|i| engine::Change {
            source: rename_dir.join(format!("old{i:03}.mkv")),
            target: rename_dir.join(format!("new{i:03}.mkv")),
        })
        .collect();
    for c in &changes {
        if !c.source.exists() {
            std::fs::write(&c.source, b"unchanged fixture")?;
        }
    }
    let history = dir.join("undo.json");
    let rename_undo_500_ms = median(
        || {
            engine::rename_batch(&changes, &history).unwrap();
            engine::undo(&history).unwrap();
        },
        3,
    );
    // Warm the allocator before checking repeated-preview memory stability.
    for _ in 0..5 {
        std::hint::black_box(
            engine::plan(&synthetic, "Benchmark", &files[..10_000], &settings).unwrap(),
        );
    }
    let memory_after_warmup = private_bytes();
    let mut memory_samples = Vec::new();
    for _ in 0..30 {
        std::hint::black_box(
            engine::plan(&synthetic, "Benchmark", &files[..10_000], &settings).unwrap(),
        );
        memory_samples.push(private_bytes());
    }
    let memory_after_repeats = private_bytes();
    let report = serde_json::json!({"private_bytes_after_warmup":memory_after_warmup,"private_bytes_after_30_repeats":memory_after_repeats,"private_bytes_samples":memory_samples,"scan_2000_ms":scan_ms,"preview_10000_ms":preview_10k_ms,"preview_50000_ms":preview_50k_ms,"rename_undo_500_ms":rename_undo_500_ms});
    std::fs::write(
        dir.join("benchmark.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(())
}

fn private_bytes() -> usize {
    use windows_sys::Win32::System::{
        ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX},
        Threading::GetCurrentProcess,
    };
    let mut counters: PROCESS_MEMORY_COUNTERS_EX = unsafe { std::mem::zeroed() };
    counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
    // Process pseudo-handle is borrowed; the OS fills only the provided structure.
    unsafe {
        if GetProcessMemoryInfo(
            GetCurrentProcess(),
            (&mut counters as *mut PROCESS_MEMORY_COUNTERS_EX).cast(),
            counters.cb,
        ) == 0
        {
            return 0;
        }
    }
    counters.PrivateUsage
}
