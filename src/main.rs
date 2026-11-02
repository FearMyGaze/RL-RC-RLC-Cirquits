use slint::{ComponentHandle, Timer, TimerMode, VecModel};
use std::{cell::RefCell, f32::consts::PI, rc::Rc, time::Duration};

slint::include_modules!();

const SCOPE_X_START: f32 = 52.0;
const SCOPE_X_SPAN: f32 = 990.0;
const SCOPE_TIME_WINDOW: f32 = 0.05;
const SCOPE_SAMPLES: usize = 360;
const SCOPE_MAX_AMPLITUDE: f32 = 44.0;
const SCOPE_CENTER_Y: f32 = 102.0;
const SCOPE_VOLTAGE_RANGE: f32 = 600.0;
const SCOPE_CURRENT_RANGE: f32 = 15.0;

#[derive(Clone)]
struct State {
    mode: i32,       // 0 = RC, 1 = RL, 2 = RLC
    l_mh: f32,
    c_uf: f32,
    r_ohm: f32,
    frequency: f32,
    voltage: f32,
    speed: f32,
    playing: bool,
    time: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            mode: 1,
            l_mh: 100.0,
            c_uf: 100.0,
            r_ohm: 31.4,
            frequency: 50.0,
            voltage: 230.0,
            speed: 0.20,
            playing: true,
            time: 0.0,
        }
    }
}

#[derive(Clone, Copy)]
struct Results {
    xl: f32,
    xc: f32,
    xnet: f32,
    z: f32,
    current: f32,
    phi_rad: f32,
    phi_deg: f32,
    dt_ms: f32,
    resonance: f32,
    vr: f32,
    vx: f32,
}

fn calculate(s: &State) -> Results {
    let l = (s.l_mh / 1000.0).max(0.000_001);
    let c = (s.c_uf / 1_000_000.0).max(0.000_000_01);
    let f = s.frequency.max(0.1);
    let xl = 2.0 * PI * f * l;
    let xc = 1.0 / (2.0 * PI * f * c);
    let xnet = match s.mode {
        0 => -xc,
        2 => xl - xc,
        _ => xl,
    };
    let z = (s.r_ohm * s.r_ohm + xnet * xnet).sqrt().max(0.000_001);
    let current = s.voltage / z;
    let phi_rad = xnet.atan2(s.r_ohm);
    let phi_deg = phi_rad.to_degrees();
    let dt_ms = (phi_deg / (360.0 * f)) * 1000.0;
    let resonance = 1.0 / (2.0 * PI * (l * c).sqrt());
    Results {
        xl, xc, xnet, z, current, phi_rad, phi_deg, dt_ms, resonance,
        vr: current * s.r_ohm,
        vx: current * xnet,
    }
}

fn nice_scale(peak: f32) -> f32 {
    let peak = peak.max(0.001);
    let magnitude = 10.0_f32.powf(peak.log10().floor());
    let normalized = peak / magnitude;
    let step = if normalized <= 1.0 {
        1.0
    } else if normalized <= 1.5 {
        1.5
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 3.0 {
        3.0
    } else if normalized <= 4.0 {
        4.0
    } else if normalized <= 5.0 {
        5.0
    } else if normalized <= 6.0 {
        6.0
    } else if normalized <= 8.0 {
        8.0
    } else {
        10.0
    };
    step * magnitude
}

fn channel_scale(peak: f32, base_range: f32) -> f32 {
    if peak <= base_range {
        base_range
    } else {
        nice_scale(peak)
    }
}

fn format_scale(value: f32) -> String {
    if value >= 100.0 {
        format!("{:.0}", value)
    } else if value >= 10.0 {
        format!("{:.1}", value)
    } else {
        format!("{:.2}", value)
    }
}

fn waveform(phase: f32, cycles: f32, amplitude: f32, center_y: f32) -> Vec<WaveSegment> {
    // Draw a continuous sine trace as short rotated line segments, not text glyphs.
    let mut segments = Vec::with_capacity(SCOPE_SAMPLES);
    for i in 0..SCOPE_SAMPLES {
        let u0 = i as f32 / SCOPE_SAMPLES as f32;
        let u1 = (i + 1) as f32 / SCOPE_SAMPLES as f32;
        let x0 = SCOPE_X_START + SCOPE_X_SPAN * u0;
        let x1 = SCOPE_X_START + SCOPE_X_SPAN * u1;
        let y0 = center_y - amplitude * (phase + 2.0 * PI * cycles * u0).sin();
        let y1 = center_y - amplitude * (phase + 2.0 * PI * cycles * u1).sin();
        let dx = x1 - x0;
        let dy = y1 - y0;
        segments.push(WaveSegment {
            x: x0,
            y: y0 - 1.0,
            length: (dx * dx + dy * dy).sqrt(),
            rotation: dy.atan2(dx).to_degrees(),
        });
    }
    segments
}

struct Conclusion {
    title: &'static str,
    text: &'static str,
    formula: &'static str,
}

fn conclusions_for_mode(mode: i32) -> [Conclusion; 3] {
    match mode {
        0 => [
            Conclusion {
                title: "Φάση ρεύματος",
                text: "Το ρεύμα προηγείται της τάσης κατά φ (χωρητική συμπεριφορά).",
                formula: "φ = −arctan(XC / R)",
            },
            Conclusion {
                title: "Επίδραση f και C",
                text: "Αύξηση της f ή της C μειώνει το XC, άρα αυξάνει το ρεύμα και μειώνει το |φ|.",
                formula: "XC = 1 / (2π·f·C)",
            },
            Conclusion {
                title: "Τάσεις στοιχείων",
                text: "Η τάση VR είναι σε φάση με το ρεύμα, ενώ η VC καθυστερεί κατά 90°.",
                formula: "VR = I·R   VC = I·XC",
            },
        ],
        2 => [
            Conclusion {
                title: "Συντονισμός",
                text: "Σε συντονισμό f = f₀ ισχύει XL = XC, άρα Z = R, το ρεύμα είναι μέγιστο και φ = 0°.",
                formula: "XL = XC   f₀ = 1 / (2π√(LC))",
            },
            Conclusion {
                title: "Κάτω από τον συντονισμό",
                text: "Για f < f₀ επικρατεί η χωρητική συμπεριφορά· το ρεύμα προηγείται της τάσης.",
                formula: "f < f₀ ⇒ XC > XL ⇒ φ < 0",
            },
            Conclusion {
                title: "Πάνω από τον συντονισμό",
                text: "Για f > f₀ επικρατεί η επαγωγική συμπεριφορά· το ρεύμα καθυστερεί της τάσης.",
                formula: "f > f₀ ⇒ XL > XC ⇒ φ > 0",
            },
        ],
        _ => [
            Conclusion {
                title: "Φάση ρεύματος",
                text: "Το ρεύμα καθυστερεί της τάσης κατά φ (επαγωγική συμπεριφορά).",
                formula: "φ = +arctan(XL / R)",
            },
            Conclusion {
                title: "Επίδραση f και L",
                text: "Αύξηση της f ή της L αυξάνει το XL, άρα μειώνει το ρεύμα και αυξάνει το |φ|.",
                formula: "XL = 2π·f·L",
            },
            Conclusion {
                title: "Τάσεις στοιχείων",
                text: "Η τάση VR είναι σε φάση με το ρεύμα, ενώ η VL προηγείται κατά 90°.",
                formula: "VR = I·R   VL = I·XL",
            },
        ],
    }
}

fn apply(ui: &MainWindow, s: &State) {
    let p = calculate(s);
    let mode_name = match s.mode { 0 => "RC", 2 => "RLC", _ => "RL" };
    let hint = if p.phi_deg > 0.05 {
        "Το ρεύμα καθυστερεί ως προς την τάση (επαγωγική συμπεριφορά)"
    } else if p.phi_deg < -0.05 {
        "Το ρεύμα προηγείται της τάσης (χωρητική συμπεριφορά)"
    } else {
        "Τάση και ρεύμα βρίσκονται σε φάση"
    };
    let phi_state = if p.phi_deg > 0.05 { "ΚΑΘΥΣΤΕΡΗΣΗ" } else if p.phi_deg < -0.05 { "ΠΡΟΗΓΗΣΗ" } else { "ΣΕ ΦΑΣΗ" };
    let x_label = match s.mode {
        0 => format!("Xc = {:.2} Ω", p.xc),
        2 => format!("XL − XC = {:.2} Ω", p.xnet),
        _ => format!("XL = {:.2} Ω", p.xl),
    };
    let v_reactive_label = match s.mode {
        0 => format!("VC = {:.2} V", p.vx.abs()),
        2 => format!("VL − VC = {:.2} V", p.vx),
        _ => format!("VL = {:.2} V", p.vx.abs()),
    };
    let z_reactive_label = match s.mode {
        0 => format!("XC = {:.2} Ω", p.xc),
        2 => format!("XL − XC = {:.2} Ω", p.xnet),
        _ => format!("XL = {:.2} Ω", p.xl),
    };
    let resonance_text = if p.resonance >= 1000.0 {
        format!("f₀ = {:.2} kHz", p.resonance / 1000.0)
    } else {
        format!("f₀ = {:.2} Hz", p.resonance)
    };

    ui.set_circuit_mode(s.mode);
    ui.set_l_value(s.l_mh);
    ui.set_c_value(s.c_uf);
    ui.set_r_value(s.r_ohm);
    ui.set_frequency_value(s.frequency);
    ui.set_voltage_value(s.voltage);
    ui.set_speed_value(s.speed);
    ui.set_l_text(format!("{:.0} mH", s.l_mh).into());
    ui.set_c_text(format!("{:.0} μF", s.c_uf).into());
    ui.set_r_text(format!("{:.1} Ω", s.r_ohm).into());
    ui.set_frequency_text(format!("{:.0} Hz", s.frequency).into());
    ui.set_voltage_text(format!("{:.0} V", s.voltage).into());
    ui.set_speed_text(format!("{:.2}x", s.speed).into());
    ui.set_play_text(if s.playing { "Παύση" } else { "Συνέχεια" }.into());
    ui.set_mode_title(format!("Κύκλωμα σειράς {mode_name}").into());
    ui.set_mode_hint(hint.into());
    ui.set_phase_legend(format!("{} · φ = {:.1}°", phi_state, p.phi_deg.abs()).into());

    ui.set_calc_xl(format!("XL = {:.2} Ω", p.xl).into());
    ui.set_calc_xc(format!("XC = {:.2} Ω", p.xc).into());
    ui.set_calc_xnet(x_label.into());
    ui.set_calc_z(format!("Z = {:.2} Ω", p.z).into());
    ui.set_calc_i(format!("Irms = {:.2} A", p.current).into());
    ui.set_calc_phi(format!("φ = {:.1}°", p.phi_deg.abs()).into());
    ui.set_calc_phi_state(phi_state.into());
    ui.set_calc_dt(format!("Δt = {:.2} ms", p.dt_ms.abs()).into());
    ui.set_calc_resonance(resonance_text.into());
    ui.set_calc_vr(format!("VR = {:.2} V", p.vr).into());
    ui.set_calc_vx(v_reactive_label.into());
    ui.set_calc_v(format!("V = {:.2} V", s.voltage).into());
    ui.set_calc_r(format!("R = {:.2} Ω", s.r_ohm).into());
    ui.set_calc_x(z_reactive_label.into());
    ui.set_calc_z_triangle(format!("Z = {:.2} Ω", p.z).into());

    let [conclusion_1, conclusion_2, conclusion_3] = conclusions_for_mode(s.mode);
    ui.set_conclusion_title(format!("ΒΑΣΙΚΑ ΣΥΜΠΕΡΑΣΜΑΤΑ · {mode_name}").into());
    ui.set_conclusion_1_title(conclusion_1.title.into());
    ui.set_conclusion_1(conclusion_1.text.into());
    ui.set_conclusion_1_formula(conclusion_1.formula.into());
    ui.set_conclusion_2_title(conclusion_2.title.into());
    ui.set_conclusion_2(conclusion_2.text.into());
    ui.set_conclusion_2_formula(conclusion_2.formula.into());
    ui.set_conclusion_3_title(conclusion_3.title.into());
    ui.set_conclusion_3(conclusion_3.text.into());
    ui.set_conclusion_3_formula(conclusion_3.formula.into());

    // Keep the two right triangles proportional to their hypotenuse.
    let base = 118.0 * s.r_ohm / p.z;
    let reactive = 118.0 * p.xnet.abs() / p.z;
    let sign = if p.xnet < 0.0 { -1.0 } else { 1.0 };
    let angle = -sign * reactive.atan2(base.max(0.001)).to_degrees();
    ui.set_triangle_base(base);
    ui.set_triangle_height(reactive.max(1.0));
    ui.set_triangle_angle(angle);
    ui.set_triangle_sign(sign);
    ui.set_voltage_triangle_base(format!("VR {:.1} V", p.vr).into());
    ui.set_voltage_triangle_side(match s.mode {
        0 => format!("VC {:.1} V", p.vx.abs()).into(),
        2 => format!("VL−VC {:.1} V", p.vx).into(),
        _ => format!("VL {:.1} V", p.vx.abs()).into(),
    });
    ui.set_voltage_triangle_hyp(format!("V {:.1} V", s.voltage).into());
    ui.set_impedance_triangle_base(format!("R {:.1} Ω", s.r_ohm).into());
    ui.set_impedance_triangle_side(format!("X {:.1} Ω", p.xnet).into());
    ui.set_impedance_triangle_hyp(format!("Z {:.1} Ω", p.z).into());

    // The yellow angle badge stays inside the wedge between the blue and red vectors.
    // Voltage is the fixed 0° reference. Current lags in RL and leads in RC.
    let v_angle = 0.0;
    let i_angle = p.phi_deg;
    let mid = (v_angle + i_angle) * 0.5;
    let radius = 34.0;
    let bubble_x = 175.0 + radius * mid.to_radians().cos();
    let bubble_y = 153.0 + radius * mid.to_radians().sin();
    ui.set_voltage_vector_angle(v_angle);
    ui.set_current_vector_angle(i_angle);
    ui.set_voltage_tip_x(175.0 + 128.0 * v_angle.to_radians().cos());
    ui.set_voltage_tip_y(153.0 + 128.0 * v_angle.to_radians().sin());
    ui.set_current_tip_x(175.0 + 106.0 * i_angle.to_radians().cos());
    ui.set_current_tip_y(153.0 + 106.0 * i_angle.to_radians().sin());
    ui.set_angle_bubble_x(bubble_x);
    ui.set_angle_bubble_y(bubble_y);
    ui.set_angle_badge(format!("{:.0}°", p.phi_deg.abs()).into());

    // The scope has two physically calibrated channels: each trace amplitude is derived
    // from the real peak value and the selected vertical range for that channel.
    let voltage_peak = s.voltage * 2.0_f32.sqrt();
    let current_peak = p.current * 2.0_f32.sqrt();
    let voltage_scale = channel_scale(voltage_peak, SCOPE_VOLTAGE_RANGE);
    let current_scale = channel_scale(current_peak, SCOPE_CURRENT_RANGE);
    let voltage_amplitude = SCOPE_MAX_AMPLITUDE * voltage_peak / voltage_scale;
    let current_amplitude = SCOPE_MAX_AMPLITUDE * current_peak / current_scale;
    ui.set_scope_voltage_scale(
        format!(
            "Vp = {:.2} V  ·  Vrms = {:.2} V  ·  κλίμακα ±{} V",
            voltage_peak,
            s.voltage,
            format_scale(voltage_scale)
        )
        .into(),
    );
    ui.set_scope_current_scale(
        format!(
            "Ip = {:.3} A  ·  Irms = {:.3} A  ·  κλίμακα ±{} A",
            current_peak,
            p.current,
            format_scale(current_scale)
        )
        .into(),
    );
    ui.set_scope_time_base(
        format!(
            "Βάση χρόνου: {:.1} ms/div  ·  παράθυρο {:.0} ms",
            SCOPE_TIME_WINDOW * 1000.0 / 10.0,
            SCOPE_TIME_WINDOW * 1000.0
        )
        .into(),
    );

    // v(t) = Vpeak·sin(ωt); i(t) = Ipeak·sin(ωt − φ).
    // The horizontal window is a real 50 ms so the displayed number of cycles and the
    // phase displacement follow the selected frequency and calculated phase angle.
    let cycles_in_window = s.frequency * SCOPE_TIME_WINDOW;
    let electrical_phase = 2.0 * PI * (s.time * s.frequency).fract();
    let voltage_segments = Rc::new(VecModel::from(waveform(
        electrical_phase,
        cycles_in_window,
        voltage_amplitude,
        SCOPE_CENTER_Y,
    )));
    let current_segments = Rc::new(VecModel::from(waveform(
        electrical_phase - p.phi_rad,
        cycles_in_window,
        current_amplitude,
        SCOPE_CENTER_Y,
    )));
    ui.set_voltage_wave(voltage_segments.into());
    ui.set_current_wave(current_segments.into());
    ui.set_animation_time(s.time);
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    let state = Rc::new(RefCell::new(State::default()));
    apply(&ui, &state.borrow());

    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_mode_selected(move |mode| {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                s.mode = mode.clamp(0, 2);
                apply(&ui, &s);
            }
        });
    }

    macro_rules! bind_slider {
        ($callback:ident, $field:ident) => {{
            let weak = ui.as_weak();
            let state = state.clone();
            ui.$callback(move |value| {
                if let Some(ui) = weak.upgrade() {
                    let mut s = state.borrow_mut();
                    s.$field = value;
                    apply(&ui, &s);
                }
            });
        }};
    }
    bind_slider!(on_l_changed, l_mh);
    bind_slider!(on_c_changed, c_uf);
    bind_slider!(on_r_changed, r_ohm);
    bind_slider!(on_frequency_changed, frequency);
    bind_slider!(on_voltage_changed, voltage);
    bind_slider!(on_speed_changed, speed);

    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_play_clicked(move || {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                s.playing = !s.playing;
                apply(&ui, &s);
            }
        });
    }
    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_reset_clicked(move || {
            if let Some(ui) = weak.upgrade() {
                let mode = state.borrow().mode;
                *state.borrow_mut() = State {
                    mode,
                    ..State::default()
                };
                apply(&ui, &state.borrow());
            }
        });
    }
    {
        let weak = ui.as_weak();
        let state = state.clone();
        ui.on_quick_speed(move |speed| {
            if let Some(ui) = weak.upgrade() {
                let mut s = state.borrow_mut();
                s.speed = speed;
                apply(&ui, &s);
            }
        });
    }

    let timer = Timer::default();
    let weak = ui.as_weak();
    let state_for_timer = state.clone();
    timer.start(TimerMode::Repeated, Duration::from_millis(33), move || {
        if let Some(ui) = weak.upgrade() {
            let mut s = state_for_timer.borrow_mut();
            if s.playing {
                s.time += 0.033 * s.speed;
                if s.time > 100.0 { s.time = 0.0; }
                apply(&ui, &s);
            }
        }
    });

    ui.run()
}
