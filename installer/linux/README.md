# Anemorph for Linux

Extract the archive and open `anemorph`. Your video contents stay unchanged; review the new filenames before clicking Rename.

To add Anemorph to your app menu, open a terminal in this folder and run:

```sh
sh install.sh
```

Choose a different location with `sh install.sh /absolute/install/path`. No administrator permission is needed. A custom location may need its `share` folder added to `XDG_DATA_DIRS` for the app menu to find it.

Compatible file managers can open selected videos using **Open With > Anemorph**. You can also run `./anemorph --files /path/to/video.mkv`.

This x86-64 build requires glibc 2.39 or newer. On Ubuntu 24.04 with X11, install missing runtime libraries with `sudo apt install libxkbcommon-x11-0 libwayland-client0 libegl1 libgl1`. A graphical desktop is required. The default drawing mode does not require a graphics card.

Run `sh uninstall.sh` with the same install location to remove the installed files. Settings and undo history are kept in `${XDG_DATA_HOME:-$HOME/.local/share}/anemorph`.

Source, help, and updates: https://github.com/yaboirar/Anemorph

Code licenses are included under `licenses`. The supplied logo has separate terms in `third-party-notices.md`.
