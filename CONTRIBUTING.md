<div align="center">

# Contributing

**Help make Anemorph better for the people using it.**

[Home](README.md) · [Report a problem](https://github.com/yaboirar/Anemorph/issues) · [Build from source](docs/building.md)

</div>

---

## You don't need to write code

| You can help with | What to share |
| :--- | :--- |
| A bug | What you did, what you expected, and what happened |
| A naming mistake | Invented filenames that reproduce the problem and the names you wanted |
| A translation | The language, current wording, and a clearer alternative |
| An interface improvement | A screenshot or description of what is hard to use |
| Documentation | An unclear step, missing explanation, or broken link |

Start with an [issue](https://github.com/yaboirar/Anemorph/issues). For a larger change, explain the idea first so its purpose and scope are clear.

---

## Reporting a problem

Include your **Anemorph version**, **Windows version**, and the shortest steps that reproduce it. If Explorer is involved, say whether you opened Anemorph first or launched it from a right-click menu.

For filename problems, a before/expected table is useful:

| Original filename | Expected filename |
| :--- | :--- |
| `[Group] Anemorph - 19 [1080p].mkv` | `Anemorph S01E19.mkv` |

**Keep personal data out of reports.** Invented filenames usually suffice. Do not upload videos, private folder paths, settings, or undo history. Crop or blur personal details in screenshots.

---

## Making a change

1. Follow the [build guide](docs/building.md) to set up the project.
2. Keep the change focused on one problem or improvement.
3. Test with disposable files or generated fixtures. Never use your only copy of a video.
4. Run the checks below and open a pull request explaining what changes for the user.

```powershell
cargo fmt --all --check
cargo clippy --workspace --locked --all-targets -- -D warnings
cargo test --workspace --locked
```

Add a regression test when fixing behavior that could break again, such as episode detection, Explorer selections, or undo. In the pull request, describe the problem, the resulting behavior, and how you checked it.

### Interface changes

Include before/after screenshots at normal and minimum window sizes. Check keyboard selection, resizing, window dragging, and translated text. An idle window should stay idle after animations finish.

### Documentation changes

Use everyday language in user-facing pages. Keep the README's existing visual structure, screenshots, and download links; make focused adjustments rather than replacing the page. Put build commands and implementation details in the developer guides.

---

## Licenses and assets

Code contributions use the project's [MIT license](LICENSE). Only add artwork, fonts, or dependencies with clear redistribution terms, and include the required notices.

The supplied Anemo artwork is outside the code license. Read [third-party notices](THIRD_PARTY_NOTICES.md) before reusing or changing project assets.
