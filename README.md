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
- Voltage and impedance triangles with the correct reactive sign
- Phasor diagram with voltage as the fixed 0-degree reference and current leading/lagging according to the circuit
- Oscilloscope traces drawn as connected line segments from sine-wave samples (not text/bar characters)
- Play/pause, simulation-speed controls and a reset that restores the default parameter values for the current circuit mode and redraws every diagram

The project uses Slint 1.18.1. Build it locally with `cargo run` to compile the UI and launch the simulator.

## Oscilloscope scales
The oscilloscope displays the source voltage and series current as two overlaid traces on a shared baseline, grid and time axis so phase and amplitude differences are directly comparable. Each channel still uses its own calibrated vertical range (V and A). Each trace amplitude is derived from the calculated peak value (`Vpeak = Vrms·√2`, `Ipeak = Irms·√2`) and the vertical range shown next to each channel. The voltage channel uses a fixed ±600 V range; the current channel uses ±15 A and expands only when a larger current would otherwise be clipped. The horizontal window is a real 50 ms (5 ms/div), so the number of visible cycles follows the selected frequency. The plotted traces preserve the calculated phase relation: `v(t) = Vpeak·sin(ωt)` and `i(t) = Ipeak·sin(ωt − φ)`.
