//! The measured table: every simulated case with the exact metrics it must reproduce.
//!
//! Runs are deterministic per seed, so any changed decision changes a row. A representative
//! subset runs by default; `cargo test -- --include-ignored` runs all of them.

use daglight_simulation_config::{Config, Reveal, Strategy};
use daglight_simulation_metrics::Metrics;

/// A row of the table.
trait Case {
    /// Creates a row at `rate` with ten equal miners, 1 s delay, 30% jitter, 60 s and seed 1.
    fn case(rate: f64) -> Self;

    /// Runs the row and returns its printed metrics.
    fn row(self) -> String;
}

impl Case for Config {
    fn case(rate: f64) -> Self {
        Config::new(rate, 1.0, 10).jitter(0.3)
    }

    fn row(self) -> String {
        let row = Metrics::outcome(self).row();
        row.split_whitespace().collect::<Vec<_>>().join(" ")
    }
}

/// Declares one test per row, asserting the row's metrics.
macro_rules! table {
    ($( $(#[$attr:meta])* $name:ident: $case:expr => $expected:literal; )*) => {
        $(
            $(#[$attr])*
            #[test]
            fn $name() {
                assert_eq!($case.row(), $expected);
            }
        )*
    };
}

table! {
    #[ignore] honest5: Config::case(5.0) => "288 72 0.00% 0.39 0.00 11.40s 1.18 0 0.92s 0.90 0 0.0% -";
    honest20: Config::case(20.0) => "1221 155 0.00% 0.00 0.00 7.49s 0.90 0 0.92s 0.92 0 0.0% -";
    #[ignore] honest50: Config::case(50.0) => "2990 280 0.00% 0.00 0.00 8.21s 0.82 0 0.94s 0.93 0 0.0% -";
    #[ignore] honest50m100: Config::case(50.0).miners(100) => "2988 91 0.00% 0.24 0.00 8.04s 0.20 0 0.88s 0.88 0 0.0% -";
    #[ignore] m20: Config::case(20.0).pool(0.2) => "1221 196 0.00% 0.40 0.00 7.36s 0.95 0 0.94s 0.94 0 0.0% -";
    m30: Config::case(20.0).pool(0.3) => "1221 341 0.00% 0.00 0.00 7.52s 1.55 0 0.99s 0.99 0 0.0% -";
    #[ignore] m40: Config::case(20.0).pool(0.4) => "1221 453 0.00% 0.00 0.00 7.50s 1.94 0 0.99s 0.99 0 0.0% -";
    #[ignore] m20r50: Config::case(50.0).pool(0.2) => "2990 389 0.00% 0.20 0.00 8.13s 1.30 0 0.95s 0.95 0 0.0% -";
    #[ignore] m30r50: Config::case(50.0).pool(0.3) => "2990 783 0.00% 0.00 0.00 7.48s 2.33 0 1.01s 1.01 0 0.0% -";
    misdelay: Config::case(20.0).delay(2.0).assumed_delay(1.0) => "1223 125 0.00% 0.00 0.00 12.77s 1.88 0 1.09s 0.54 0 0.0% -";
    #[ignore] fixed20: Config::case(20.0).fixed() => "1214 147 0.00% 0.95 0.00 8.12s 1.10 0 1.00s 1.00 0 0.0% -";
    #[ignore] fixed_misdelay: Config::case(20.0).delay(2.0).assumed_delay(1.0).fixed() => "1214 123 0.00% 0.00 0.00 12.37s 1.80 0 1.00s 0.50 0 0.0% -";
    #[ignore] fixed_spine45_rate1: Config::case(1.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0).fixed() => "142 56 0.00% 0.00 0.00 32.46s 0.00 0 1.00s 1.00 58 17.9% 51@12s";
    #[ignore] fixed_gr49e10_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Greedy, 0.49, Reveal::Every(10.0), 0.0).fixed() => "2379 1210 0.00% 2.70 0.00 18.53s 0.95 0 1.00s 1.00 1210 100.0% 12@10s";
    #[ignore] guess_low: Config::case(20.0).assumed_delay(0.3) => "1221 153 0.34% 0.00 0.00 5.37s 3.66 0 0.33s 0.33 0 0.0% -";
    #[ignore] guess_high: Config::case(20.0).assumed_delay(3.0) => "1221 152 0.00% 0.22 0.00 13.43s 0.28 0 2.45s 2.44 0 0.0% -";
    #[ignore] spine45_rate1: Config::case(1.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "143 59 0.00% 0.00 0.00 33.38s 0.00 0 0.91s 0.99 59 14.9% 52@12s";
    #[ignore] gr49e20_low: Config::case(20.0).assumed_delay(0.3).duration(120.0).attack(Strategy::Greedy, 0.49, Reveal::Every(20.0), 10.0) => "2448 1207 0.00% 12.49 4.23 22.69s 6.58 22 0.25s 0.24 1207 100.0% 6@20s";
    #[ignore] half_pace45_rate1: Config::case(1.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0).stamp_pace(0.5) => "139 54 0.00% 0.00 0.00 25.26s 0.00 0 0.93s 1.00 58 16.3% 51@12s";
    #[ignore] half_pace49: Config::case(20.0).duration(120.0).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0).stamp_pace(0.5) => "2441 279 0.00% 0.00 0.00 21.47s 2.89 0 0.90s 0.90 1160 33.4% 1070@10s";
    #[ignore] double_pace45_rate1: Config::case(1.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0).stamp_pace(2.0) => "153 60 0.00% 0.00 0.00 41.96s 0.00 0 0.92s 0.96 66 11.7% 59@12s";
    #[ignore] double_pace49: Config::case(20.0).duration(120.0).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0).stamp_pace(2.0) => "2802 293 0.00% 0.28 0.00 40.10s 0.57 0 0.85s 0.80 1490 32.7% 1400@10s";
    #[ignore] balance45: Config::case(20.0).attack(Strategy::Balance, 0.45, Reveal::Immediately, 10.0) => "1153 163 0.00% 0.15 0.00 15.82s 0.70 0 0.96s 0.95 494 50.4% -";
    #[ignore] spine30: Config::case(20.0).attack(Strategy::Withhold, 0.3, Reveal::Immediately, 10.0) => "1167 140 0.00% 0.40 0.00 14.44s 1.00 0 0.93s 0.92 344 30.8% 294@10s";
    #[ignore] wh30: Config::case(20.0).attack(Strategy::Withhold, 0.3, Reveal::WhenAhead, 0.0) => "1221 126 0.00% 0.00 0.00 17.81s 0.60 0 0.89s 0.85 333 4.9% 19@0s";
    #[ignore] wh45: Config::case(20.0).attack(Strategy::Withhold, 0.45, Reveal::WhenAhead, 0.0) => "1286 133 0.00% 0.00 0.00 31.76s 0.60 0 0.90s 0.83 574 17.1% 34@0s";
    #[ignore] wh45e10: Config::case(20.0).attack(Strategy::Withhold, 0.45, Reveal::Every(10.0), 0.0) => "1294 116 0.00% 0.40 0.00 19.10s 0.45 0 0.89s 0.90 579 0.0% 6@10s";
    q2: Config::case(1.0).miners(2).duration(300.0) => "302 234 0.00% 0.00 0.00 27.70s 2.04 0 0.63s 0.65 0 0.0% -";
    q1s: Config::case(1.0).miners(1).duration(600.0).attack(Strategy::Withhold, 0.3, Reveal::Every(20.0), 0.0) => "779 466 0.00% 0.00 0.00 135.97s 0.00 0 0.37s 0.30 313 0.0% 30@21s";
    #[ignore] hash_halves_withhold: Config::case(20.0).duration(660.0).rate_switch(60.0, 10.0).attack(Strategy::Withhold, 0.3, Reveal::Every(20.0), 60.0) => "11599 1319 0.00% 0.07 0.00 193.24s 0.55 0 0.52s 0.68 4375 30.0% 31@60s";
    hash_doubles_spine: Config::case(1.0).duration(360.0).rate_switch(300.0, 2.0).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 300.0) => "415 216 0.00% 0.00 0.00 27.18s 0.00 0 0.77s 0.44 178 43.5% 54@302s";
    dag0_49_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2726 236 0.00% 0.00 0.00 60.33s 0.55 0 0.81s 0.68 1363 4.6% 51@0s";
    dag0_49_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2668 232 0.00% 0.00 0.00 47.34s 0.54 0 0.82s 0.74 1411 8.0% 60@0s";
    dag0_49_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2697 224 0.00% 0.09 0.00 61.94s 0.45 0 0.81s 0.68 1314 4.3% 48@0s";
    dag0_49_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2685 205 0.00% 0.38 0.00 61.79s 0.65 0 0.81s 0.68 1324 2.1% 40@0s";
    dag0_49_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2683 219 0.00% 0.00 0.00 61.66s 0.68 0 0.81s 0.68 1357 4.5% 50@0s";
    #[ignore] spine0_45_s1: Config::case(20.0).seed(1).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1169 173 0.00% 0.05 0.00 17.89s 0.90 0 0.94s 0.94 508 50.4% 430@10s";
    #[ignore] spine0_47_s1: Config::case(20.0).seed(1).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1169 180 0.00% 0.00 0.00 18.08s 0.50 0 0.95s 0.94 537 52.3% 452@10s";
    #[ignore] spine0_49_s1: Config::case(20.0).seed(1).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1169 174 0.00% 0.05 0.00 18.62s 3.27 0 0.95s 0.95 567 53.6% 477@10s";
    #[ignore] spine0_45_s2: Config::case(20.0).seed(2).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1258 186 0.00% 0.15 0.00 18.72s 0.45 0 0.95s 0.95 586 56.4% 497@10s";
    #[ignore] spine0_47_s2: Config::case(20.0).seed(2).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1257 203 0.00% 0.00 0.00 19.43s 0.80 0 0.95s 0.95 615 56.6% 523@10s";
    #[ignore] spine0_49_s2: Config::case(20.0).seed(2).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1257 195 0.00% 0.25 0.00 20.15s 0.60 0 0.96s 0.96 638 55.9% 544@10s";
    #[ignore] spine0_45_s3: Config::case(20.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1264 182 0.00% 0.40 0.00 18.35s 0.55 0 0.95s 0.96 581 53.8% 490@10s";
    #[ignore] spine0_47_s3: Config::case(20.0).seed(3).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1264 186 0.00% 0.00 0.00 18.70s 0.55 0 0.95s 0.96 606 55.3% 510@10s";
    #[ignore] spine0_49_s3: Config::case(20.0).seed(3).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1264 191 0.00% 0.40 0.00 19.22s 0.40 0 0.95s 0.96 633 56.0% 532@10s";
    #[ignore] spine0_45_s4: Config::case(20.0).seed(4).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1247 187 0.00% 0.80 0.00 18.11s 0.50 0 0.95s 0.95 571 51.4% 481@10s";
    #[ignore] spine0_47_s4: Config::case(20.0).seed(4).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1248 185 0.00% 0.90 0.00 18.79s 0.65 0 0.94s 0.95 587 53.0% 495@10s";
    #[ignore] spine0_49_s4: Config::case(20.0).seed(4).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1247 188 0.00% 0.10 0.00 19.37s 0.60 0 0.95s 0.95 621 55.3% 523@10s";
    #[ignore] spine0_45_s5: Config::case(20.0).seed(5).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1228 182 0.00% 0.35 0.00 17.91s 0.65 0 0.95s 0.95 553 49.5% 469@10s";
    spine0_47_s5: Config::case(20.0).seed(5).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1229 184 0.00% 0.15 0.00 18.49s 0.65 0 0.95s 0.94 575 54.9% 488@10s";
    #[ignore] spine0_49_s5: Config::case(20.0).seed(5).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1229 190 0.00% 0.25 0.00 19.15s 0.75 0 0.95s 0.95 605 55.3% 515@10s";
    #[ignore] spine0_45_s6: Config::case(20.0).seed(6).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1195 196 0.00% 0.00 0.00 18.69s 0.75 0 0.96s 0.96 557 57.7% 462@10s";
    #[ignore] spine0_47_s6: Config::case(20.0).seed(6).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1195 194 0.00% 0.10 0.00 19.02s 0.85 0 0.95s 0.95 579 62.0% 480@10s";
    #[ignore] spine0_49_s6: Config::case(20.0).seed(6).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1195 205 0.00% 0.00 0.00 19.14s 0.55 0 0.96s 0.96 602 59.6% 500@10s";
    #[ignore] wh47_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2698 255 0.00% 0.00 0.00 60.85s 0.43 0 0.81s 0.68 1299 9.7% 44@0s";
    #[ignore] wh47_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2720 262 0.00% 0.13 0.00 62.05s 0.61 0 0.80s 0.67 1329 14.4% 44@0s";
    #[ignore] wh47_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2662 254 0.00% 0.09 0.00 60.17s 0.76 0 0.81s 0.68 1238 12.2% 47@0s";
    #[ignore] wh47_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2614 241 0.00% 0.00 0.00 60.64s 0.62 0 0.81s 0.69 1230 5.8% 39@0s";
    #[ignore] wh47_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2604 235 0.00% 0.17 0.00 61.84s 0.55 0 0.80s 0.66 1261 8.3% 35@0s";
    #[ignore] wh47_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2612 241 0.00% 0.17 0.00 59.12s 0.75 0 0.82s 0.69 1225 6.2% 51@0s";
    #[ignore] wh47_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2743 243 0.00% 0.22 0.00 52.13s 0.60 0 0.81s 0.70 1268 13.1% 59@0s";
    #[ignore] wh47_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2620 238 0.00% 0.38 0.00 61.10s 0.59 0 0.81s 0.68 1244 15.8% 42@0s";
    #[ignore] wh47_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2606 226 0.00% 0.63 0.00 61.81s 0.68 0 0.81s 0.67 1277 13.3% 38@0s";
    #[ignore] wh47_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2586 239 0.00% 0.08 0.00 62.05s 0.58 0 0.81s 0.68 1219 10.4% 38@0s";
    #[ignore] wh49_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2693 236 0.00% 0.00 0.00 59.44s 0.60 0 0.81s 0.69 1325 10.9% 62@0s";
    wh49_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2584 233 0.00% 0.19 0.00 47.27s 0.70 0 0.82s 0.79 1332 15.5% 53@0s";
    #[ignore] wh49_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2664 234 0.00% 0.00 0.00 60.20s 0.54 0 0.81s 0.68 1288 7.8% 60@0s";
    #[ignore] wh49_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2604 225 0.00% 0.00 0.00 61.38s 0.50 0 0.82s 0.68 1260 5.8% 47@0s";
    #[ignore] wh49_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2653 232 0.00% 0.00 0.00 61.52s 0.58 0 0.81s 0.67 1318 8.9% 36@0s";
    #[ignore] wh49_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2601 219 0.00% 0.38 0.00 61.29s 0.65 0 0.81s 0.68 1259 0.0% 48@0s";
    #[ignore] wh49_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2712 236 0.00% 0.00 0.00 59.80s 0.65 0 0.81s 0.70 1327 13.6% 59@0s";
    #[ignore] wh49_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2565 220 0.00% 0.00 0.00 48.18s 0.59 0 0.82s 0.73 1322 9.0% 64@0s";
    #[ignore] wh49_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2615 236 0.00% 0.13 0.00 61.55s 0.49 0 0.81s 0.67 1325 16.4% 56@0s";
    #[ignore] wh49_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2589 239 0.00% 0.34 0.00 56.17s 0.59 0 0.81s 0.69 1270 9.0% 38@0s";
    #[ignore] hv0_40_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2439 958 0.00% 0.00 0.00 7.46s 0.54 0 1.01s 1.02 958 100.0% 958@0s";
    #[ignore] hv0_40_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2429 991 0.00% 0.00 0.00 7.49s 0.35 0 1.00s 1.00 991 100.0% 991@0s";
    #[ignore] hv0_40_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2469 1007 0.00% 0.00 0.00 7.32s 0.45 0 1.00s 1.01 1007 100.0% 1007@0s";
    #[ignore] hv0_40_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2358 957 0.00% 0.00 0.00 7.66s 0.40 0 0.99s 0.99 957 100.0% 957@0s";
    #[ignore] hv0_40_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2410 970 0.00% 0.00 0.00 7.44s 0.50 0 1.01s 1.01 970 100.0% 970@0s";
    #[ignore] hv0_40_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2434 994 0.00% 0.00 0.00 7.44s 0.35 0 1.00s 1.01 994 100.0% 994@0s";
    #[ignore] hv0_40_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2355 975 0.00% 0.00 0.00 7.79s 0.55 0 0.99s 0.98 975 100.0% 975@0s";
    #[ignore] hv0_40_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2366 944 0.00% 0.00 0.00 7.60s 0.40 0 1.00s 0.99 944 100.0% 944@0s";
    #[ignore] hv0_40_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2437 1028 0.00% 0.00 0.00 7.47s 0.35 0 1.00s 1.01 1028 100.0% 1028@0s";
    #[ignore] hv0_40_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2369 956 0.00% 0.00 0.00 7.70s 0.45 0 0.99s 0.99 956 100.0% 956@0s";
    #[ignore] hv0_43_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2439 1017 0.00% 0.00 0.00 7.56s 0.45 0 1.00s 1.01 1017 100.0% 1017@0s";
    #[ignore] hv0_43_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2428 1068 0.00% 0.00 0.00 7.58s 0.45 0 1.00s 1.00 1068 100.0% 1068@0s";
    #[ignore] hv0_43_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2468 1078 0.00% 0.00 0.00 7.41s 0.30 0 1.00s 1.01 1078 100.0% 1078@0s";
    #[ignore] hv0_43_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2358 1027 0.00% 0.00 0.00 7.71s 0.35 0 0.98s 0.98 1027 100.0% 1027@0s";
    #[ignore] hv0_43_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2410 1036 0.00% 0.00 0.00 7.50s 0.40 0 1.00s 1.01 1036 100.0% 1036@0s";
    #[ignore] hv0_43_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2433 1069 0.00% 0.00 0.00 7.55s 0.60 0 1.00s 1.00 1069 100.0% 1069@0s";
    #[ignore] hv0_43_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2561 471 0.00% 0.00 0.00 28.55s 1.20 0 0.84s 0.77 1094 66.8% 355@0s";
    #[ignore] hv0_43_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2366 1010 0.00% 0.00 0.00 7.67s 0.40 0 0.99s 0.98 1010 100.0% 1010@0s";
    #[ignore] hv0_43_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2437 1109 0.00% 0.00 0.00 7.51s 0.30 0 1.00s 1.00 1109 100.0% 1109@0s";
    #[ignore] hv0_43_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2369 1014 0.00% 0.00 0.00 7.75s 0.40 0 1.00s 0.99 1014 100.0% 1014@0s";
    #[ignore] hv0_45_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2439 1059 0.00% 0.00 0.00 7.54s 0.45 0 1.00s 1.01 1059 100.0% 1059@0s";
    #[ignore] hv0_45_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2428 1115 0.00% 0.00 0.00 7.64s 0.70 0 0.99s 0.99 1115 100.0% 1115@0s";
    #[ignore] hv0_45_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2468 1124 0.00% 0.00 0.00 7.45s 0.25 0 1.00s 1.01 1124 100.0% 1124@0s";
    #[ignore] hv0_45_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2358 1071 0.00% 0.00 0.00 7.75s 0.50 0 0.97s 0.97 1071 100.0% 1071@0s";
    #[ignore] hv0_45_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2410 1085 0.00% 0.00 0.00 7.56s 0.30 0 1.00s 1.00 1085 100.0% 1085@0s";
    #[ignore] hv0_45_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2433 1105 0.00% 0.00 0.00 7.55s 0.35 0 0.99s 1.00 1105 100.0% 1105@0s";
    #[ignore] hv0_45_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2355 1083 0.00% 0.00 0.00 7.88s 0.35 0 0.98s 0.97 1083 100.0% 1083@0s";
    #[ignore] hv0_45_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2366 1066 0.00% 0.00 0.00 7.74s 0.35 0 0.99s 0.98 1066 100.0% 1066@0s";
    #[ignore] hv0_45_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2437 1163 0.00% 0.00 0.00 7.58s 0.30 0 0.99s 0.99 1163 100.0% 1163@0s";
    #[ignore] hv0_45_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2369 1052 0.00% 0.00 0.00 7.80s 0.54 0 0.99s 0.98 1052 100.0% 1052@0s";
    #[ignore] hv47_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2439 1098 0.00% 0.00 0.00 7.57s 0.30 0 0.99s 1.00 1098 100.0% 1098@0s";
    #[ignore] hv47_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2428 1160 0.00% 0.00 0.00 7.68s 0.40 0 0.98s 0.98 1160 100.0% 1160@0s";
    hv47_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2467 1164 0.00% 0.00 0.00 7.51s 0.30 0 0.99s 1.00 1164 100.0% 1164@0s";
    #[ignore] hv47_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2358 1117 0.00% 0.00 0.00 7.82s 0.30 0 0.97s 0.97 1117 100.0% 1117@0s";
    #[ignore] hv47_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2410 1138 0.00% 0.00 0.00 7.60s 0.40 0 0.99s 1.00 1138 100.0% 1138@0s";
    #[ignore] hv47_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2433 1152 0.00% 0.00 0.00 7.59s 0.40 0 0.99s 0.99 1152 100.0% 1152@0s";
    #[ignore] hv47_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2355 1126 0.00% 0.00 0.00 7.91s 0.40 0 0.97s 0.96 1126 100.0% 1126@0s";
    #[ignore] hv47_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2365 1113 0.00% 0.00 0.00 7.80s 0.30 0 0.98s 0.97 1113 100.0% 1113@0s";
    #[ignore] hv47_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2437 1204 0.00% 0.00 0.00 7.62s 0.30 0 0.98s 0.98 1204 100.0% 1204@0s";
    hv47_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2369 1100 0.00% 0.00 0.00 7.80s 0.35 0 0.99s 0.98 1100 100.0% 1100@0s";
    #[ignore] hv49_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2439 1156 0.00% 0.00 0.00 7.64s 0.40 0 0.99s 0.99 1156 100.0% 1156@0s";
    #[ignore] hv49_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2428 1214 0.00% 0.00 0.00 7.72s 0.35 0 0.97s 0.98 1214 100.0% 1214@0s";
    #[ignore] hv49_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2467 1214 0.00% 0.00 0.00 7.56s 0.25 0 0.98s 0.99 1214 100.0% 1214@0s";
    #[ignore] hv49_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2358 1159 0.00% 0.00 0.00 7.86s 0.30 0 0.96s 0.96 1159 100.0% 1159@0s";
    #[ignore] hv49_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2410 1192 0.00% 0.00 0.00 7.64s 0.45 0 0.98s 0.98 1192 100.0% 1192@0s";
    #[ignore] hv49_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2433 1194 0.00% 0.00 0.00 7.66s 0.35 0 0.98s 0.98 1194 100.0% 1194@0s";
    #[ignore] hv49_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2355 1195 0.00% 0.00 0.00 7.97s 0.35 0 0.96s 0.95 1195 100.0% 1195@0s";
    #[ignore] hv49_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2365 1172 0.00% 0.00 0.00 7.87s 0.30 0 0.97s 0.96 1172 100.0% 1172@0s";
    #[ignore] hv49_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2437 1248 0.00% 0.00 0.00 7.65s 0.30 0 0.97s 0.97 1248 100.0% 1248@0s";
    #[ignore] hv49_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2369 1152 0.00% 0.00 0.00 7.82s 0.35 0 0.98s 0.97 1152 100.0% 1152@0s";
    #[ignore] hv47e20_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.47, Reveal::Every(20.0), 0.0) => "2472 373 0.40% 0.00 0.00 31.57s 9.89 0 0.83s 0.82 1185 51.6% 6@20s";
    #[ignore] hv47e20_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.47, Reveal::Every(20.0), 0.0) => "2613 229 0.00% 0.00 0.00 36.32s 0.83 0 0.82s 0.82 1247 0.0% 6@20s";
    #[ignore] hv47e20_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.47, Reveal::Every(20.0), 0.0) => "2414 200 0.42% 0.38 0.00 36.92s 9.18 0 0.83s 0.81 1170 0.0% 6@20s";
    #[ignore] hv49e20_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.49, Reveal::Every(20.0), 0.0) => "2407 879 0.33% 0.15 0.00 25.57s 8.44 0 0.85s 0.84 1173 93.0% 6@20s";
    #[ignore] hv49e20_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.49, Reveal::Every(20.0), 0.0) => "2541 530 0.00% 0.40 0.00 27.72s 1.82 0 0.84s 0.84 1241 74.3% 6@20s";
    #[ignore] gr40e10_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Greedy, 0.40, Reveal::Every(10.0), 0.0) => "2487 230 0.51% 0.25 0.00 31.76s 4.82 0 0.82s 0.81 1076 0.0% 12@10s";
    #[ignore] gr45e20_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Greedy, 0.45, Reveal::Every(20.0), 0.0) => "2458 223 0.00% 0.24 0.00 35.85s 0.44 0 0.82s 0.80 1164 0.0% 6@20s";
    #[ignore] gr47e20_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Greedy, 0.47, Reveal::Every(20.0), 0.0) => "2631 220 0.00% 0.00 0.00 36.85s 0.59 0 0.81s 0.82 1279 0.0% 6@20s";
    #[ignore] gr49e10_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Greedy, 0.49, Reveal::Every(10.0), 0.0) => "2432 1227 0.00% 2.14 38.97 47.43s 7.99 4 0.83s 1.00 1227 100.0% 12@10s";
    #[ignore] gr45a_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Greedy, 0.45, Reveal::WhenAhead, 0.0) => "2468 1124 0.00% 0.00 0.00 7.45s 0.25 0 1.00s 1.01 1124 100.0% 1124@0s";
    #[ignore] gr45e20_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Greedy, 0.45, Reveal::Every(20.0), 0.0) => "2417 358 0.00% 0.14 0.00 37.67s 1.82 0 0.82s 0.80 1115 50.9% 6@20s";
    #[ignore] gr47a_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Greedy, 0.47, Reveal::WhenAhead, 0.0) => "2355 1126 0.00% 0.00 0.00 7.91s 0.40 0 0.97s 0.96 1126 100.0% 1126@0s";
    #[ignore] gr47e20_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Greedy, 0.47, Reveal::Every(20.0), 0.0) => "2553 214 0.00% 0.00 0.00 36.57s 0.64 0 0.81s 0.80 1223 0.0% 6@20s";
    #[ignore] gr49e10_s11: Config::case(20.0).duration(120.0).seed(11).attack(Strategy::Greedy, 0.49, Reveal::Every(10.0), 0.0) => "2482 1184 0.00% 1.25 0.00 17.06s 1.19 0 0.89s 0.91 1184 100.0% 12@10s";
    #[ignore] gr49a_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Greedy, 0.49, Reveal::WhenAhead, 0.0) => "2410 1192 0.00% 0.00 0.00 7.64s 0.45 0 0.98s 0.98 1192 100.0% 1192@0s";
    #[ignore] gr49e10_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Greedy, 0.49, Reveal::Every(10.0), 0.0) => "2377 1192 0.00% 2.11 0.00 16.92s 1.09 0 0.87s 0.87 1192 100.0% 12@10s";
}
