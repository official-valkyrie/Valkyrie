<div align="center">

<img width="450" alt="Valkyrie Logo" src=".readme/logo/Valkyrie-512-3x2.png">

<h3><b>Valkyrie</b></h3>
<b>GPU MCTS chess engine</b>
<br>
<br>

[![License](https://img.shields.io/github/license/official-valkyrie/Valkyrie?style=for-the-badge)](https://opensource.org/license/gpl-3-0) 
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/official-valkyrie/Valkyrie?style=for-the-badge)](https://github.com/official-valkyrie/Valkyrie/releases/latest)
[![Commits](https://img.shields.io/github/commits-since/official-valkyrie/Valkyrie/latest?style=for-the-badge)](https://github.com/official-valkyrie/Valkyrie/commits/main)
<br>
[![Build](https://img.shields.io/github/actions/workflow/status/official-valkyrie/Valkyrie/build.yml?branch=main&label=build)](https://github.com/official-valkyrie/Valkyrie/actions/workflows/build.yml)
[![Tests](https://img.shields.io/github/actions/workflow/status/official-valkyrie/Valkyrie/test.yml?branch=main&label=tests)](https://github.com/official-valkyrie/Valkyrie/actions/workflows/test.yml)
[![Clippy](https://img.shields.io/github/actions/workflow/status/official-valkyrie/Valkyrie/clippy.yml?branch=main&label=clippy)](https://github.com/official-valkyrie/Valkyrie/actions/workflows/clippy.yml)
[![Format](https://img.shields.io/github/actions/workflow/status/official-valkyrie/Valkyrie/fmt.yml?branch=main&label=format)](https://github.com/official-valkyrie/Valkyrie/actions/workflows/fmt.yml)
<br>

</div>

## Compiling
To compile the engine yourself you need [Rust](https://rustup.rs) and GNU Make. On Linux Make is usually already installed, on Windows you can get it with `winget install ezwinports.make`. Then open the terminal and run:

```
git clone https://github.com/official-valkyrie/Valkyrie
cd Valkyrie
make
```

If you don't have Git, you can also download the source code as ZIP from the green Code button above, unpack it and run `make` inside. Binary called `valkyrie` should appear in the same folder. The first build will take a bit longer, because Rust has to download the right compiler version first.

`make` compiles the engine for the native architecture, which means it is optimized for the exact CPU in your computer and uses every instruction that CPU supports. That makes it the fastest binary for your machine, but it might not start on a different computer with an older CPU.

If you want a binary that runs on other machines too, you can compile for one of the x86-64 microarchitecture levels instead. Each level is a set of CPU instructions, higher levels are faster but need newer CPUs. Check the [Microarchitecture levels table](https://en.wikipedia.org/wiki/X86-64#Microarchitecture_levels) to see which level your CPU supports. If you are still not sure, try `v3`, it should run on most CPUs.

| Command | Level |
| :-- | :-- |
| `make v2` | x86-64-v2 |
| `make v3` | x86-64-v3 |
| `make v3-nopext` | x86-64-v3 without PEXT instruction, for AMD Zen 1 and Zen 2 CPUs, where PEXT is very slow |
| `make v4` | x86-64-v4 |

There is also `make release`, which compiles all four levels from the table one after another and puts them in the `releases` folder, with names like `Valkyrie-v0.1.0-windows-v3.exe`. Those are the same binaries that are uploaded on the releases page. They are called by the version of the engine and have all internal tuning parameters locked in, so the only options they show are the ones that are meant for the user.

## Usage
Valkyrie uses the [Universal Chess Interface (UCI)](https://en.wikipedia.org/wiki/Universal_Chess_Interface) protocol. UCI is the standard language that chess programs use to talk with chess engines. Thanks to that you don't need anything special to play against Valkyrie or use it for analysis, you can just add it as an engine in any chess GUI that supports UCI, like [Cute Chess](https://cutechess.com), [En Croissant](https://encroissant.org) or [Arena](http://www.playwitharena.de).

When you start Valkyrie on its own, by double clicking it or running it from the terminal, it opens in TUI mode, which is an interface inside the terminal window. If you prefer the classic text view, start it with `valkyrie --simple`.

You can also give Valkyrie commands when starting it from the terminal. It will run them in order and close when they are done. For example:

```
valkyrie bench
valkyrie go nodes 100000
valkyrie --option.Hash 256 position startpos moves e2e4 go nodes 100000
```

The first one runs the benchmark, which shows how fast the engine is on your computer. The second one searches the starting position for 100000 nodes and prints the best move. The last one sets the size of the search tree to 256 MB, plays e2e4 and then searches the position after it. If you want the engine to stay open after the commands are done, add `--interactive`. When running from the folder with the binary, type `./valkyrie` on Linux and `.\valkyrie.exe` on Windows instead of just `valkyrie`.

To see every available command and option, run `valkyrie help`.

## License
Valkyrie is released under the [GNU General Public License v3.0](LICENSE) with two additional terms under section 7:

- Linking with ONNX Runtime or NVIDIA TensorRT is permitted.
- Modified or redistributed versions must keep the original author attributions and credit to the Valkyrie Contributors in their documentation.
