# Between Frontiers

[Português](README.pt-BR.md)

Between Frontiers is a political and economic grand strategy simulator running from 1925 to 2025.
It simulates population groups, goods markets and a world map of states, on a deterministic
simulation engine written for this project: **Laplace-Schelling**.

The game is free and open source, and will stay that way.

## Status

Design stage. There is no playable build yet. The architecture proposal is in
[docs/architecture.pt-BR.md](docs/architecture.pt-BR.md) (Portuguese).

## Goals

- An economic and demographic simulation as fine-grained as performance allows.
- Peace time that is worth playing: economy and politics carry the game.
- Brazilian Portuguese and English from the first day.
- Linux and Windows from the first day.
- A nightly benchmark that simulates the full hundred years, checks determinism and tracks the
  cost of every system.

## Roadmap

0. Foundation: engine, map, localization, benchmark
1. Economy and the world market
2. Research and technology maturity
3. National politics
4. Diplomacy and international politics
5. War
6. Strategic AI

## The name of the engine

Laplace, for the demon: given the state and the rules, the future can be computed. Thomas
Schelling, for micromotives that turn into macrobehavior.

## License

Code is licensed under the [GNU Affero General Public License v3.0 or later](LICENSE). Game data
and text are licensed under [CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/).
Exception: the files derived from HYDE are licensed under
[CC BY-NC-SA 4.0](https://creativecommons.org/licenses/by-nc-sa/4.0/) (non-commercial use); the
list is in [`data/scenario/1925/LAND.md`](data/scenario/1925/LAND.md), "Licença do HYDE".
