# RC / RL / RLC Circuit Lab (Rust + Slint)

Educational series-circuit simulator for RC, RL and RLC circuits.

## Requirements
- Rust toolchain (stable)
- Cargo

## Run
```bash
cargo run
```

## Included
- RC, RL and RLC series circuit schematics with the AC source centered on the loop and live component values shown above each element
- Live calculations for reactance, impedance, RMS current, phase angle and voltage components, with measurement cards that reflow when a value is not applicable
- Mode-specific key conclusions shown as three numbered cards in a horizontal row beneath the measurements, each with its own title, formula and accent color
- Voltage and impedance triangles that stay inside their cards for both capacitive and inductive directions
- Phasor diagram with voltage as the fixed 0-degree reference and current leading/lagging according to the circuit
- Oscilloscope traces drawn as connected line segments from sine-wave samples (not text/bar characters)
- Orange Δt marker on the oscilloscope bridging corresponding voltage/current zero crossings
- Play/pause, simulation-speed controls and a reset that restores the default parameter values for the current circuit mode and redraws every diagram

The project uses Slint 1.18.1. Build it locally with `cargo run` to compile the UI and launch the simulator.

## Fonts
Liberation Sans Regular and Bold are bundled in `ui/fonts/` (SIL Open Font License 1.1, see `ui/fonts/LICENSE.txt`) and embedded in the executable. This keeps Greek text, subscripts, roots, arrows and other formula symbols consistent across macOS, Windows and Linux release builds.

## Oscilloscope scales
The oscilloscope displays the source voltage and series current as two overlaid traces on a shared baseline, grid and time axis so phase differences are directly comparable. Each channel is independently auto-scaled to a nearby round full-scale value so both traces fill most of the grid instead of appearing small; the active range is shown next to each channel. Each trace amplitude is derived from the calculated peak value (`Vpeak = Vrms·√2`, `Ipeak = Irms·√2`). The horizontal window is a real 50 ms (5 ms/div), so the number of visible cycles follows the selected frequency. The plotted traces preserve the calculated phase relation: `v(t) = Vpeak·sin(ωt)` and `i(t) = Ipeak·sin(ωt − φ)`. The orange marker spans the horizontal distance between corresponding zero crossings and is labelled with the calculated `Δt`.
