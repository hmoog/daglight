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
    #[ignore] honest5: Config::case(5.0) => "288 74 0.00% 0.39 0.00 7.10s 1.18 0 0.91s 0.90 0 0.0% -";
    honest20: Config::case(20.0) => "1221 145 0.00% 0.30 0.00 7.44s 0.85 0 0.91s 0.92 0 0.0% -";
    #[ignore] honest50: Config::case(50.0) => "2990 286 0.00% 0.82 0.00 8.66s 0.96 0 0.94s 0.94 0 0.0% -";
    #[ignore] honest50m100: Config::case(50.0).miners(100) => "2988 90 0.00% 0.66 0.00 7.83s 0.20 0 0.88s 0.88 0 0.0% -";
    #[ignore] m20: Config::case(20.0).pool(0.2) => "1221 169 0.00% 0.00 0.00 7.78s 1.35 0 0.93s 0.93 0 0.0% -";
    m30: Config::case(20.0).pool(0.3) => "1221 197 0.00% 0.70 0.00 7.69s 1.74 0 0.93s 0.93 0 0.0% -";
    #[ignore] m40: Config::case(20.0).pool(0.4) => "1221 353 0.00% 0.65 0.00 7.46s 2.34 0 0.95s 0.95 0 0.0% -";
    #[ignore] m20r50: Config::case(50.0).pool(0.2) => "2990 353 0.00% 0.42 0.00 7.91s 1.61 0 0.95s 0.95 0 0.0% -";
    #[ignore] m30r50: Config::case(50.0).pool(0.3) => "2990 375 0.00% 0.20 0.00 8.17s 1.72 0 0.93s 0.93 0 0.0% -";
    misdelay: Config::case(20.0).delay(2.0).assumed_delay(1.0) => "1223 106 0.00% 0.00 0.00 11.89s 1.54 0 1.09s 0.54 0 0.0% -";
    #[ignore] fixed20: Config::case(20.0).fixed() => "1214 142 0.00% 0.40 0.00 7.68s 0.80 0 1.00s 1.00 0 0.0% -";
    #[ignore] fixed_misdelay: Config::case(20.0).delay(2.0).assumed_delay(1.0).fixed() => "1214 115 0.00% 0.30 0.00 12.59s 1.65 0 1.00s 0.50 0 0.0% -";
    #[ignore] fixed_spine45_rate1: Config::case(1.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0).fixed() => "142 56 0.00% 0.00 0.00 10.71s 0.00 0 1.00s 1.00 58 17.9% 51@12s";
    #[ignore] fixed_gr49e10_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Greedy, 0.49, Reveal::Every(10.0), 0.0).fixed() => "2379 192 0.00% 0.15 0.00 36.16s 0.55 0 1.00s 1.00 1210 0.0% 12@10s";
    #[ignore] guess_low: Config::case(20.0).assumed_delay(0.3) => "1221 154 0.93% 4.33 0.00 5.15s 2.82 0 0.33s 0.33 0 0.0% -";
    #[ignore] guess_high: Config::case(20.0).assumed_delay(3.0) => "1221 141 0.00% 0.08 0.00 12.82s 0.30 0 2.45s 2.43 0 0.0% -";
    #[ignore] spine45_rate1: Config::case(1.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "143 59 0.00% 0.00 0.00 9.72s 0.00 0 0.91s 0.97 59 14.9% 52@12s";
    #[ignore] gr49e20_low: Config::case(20.0).assumed_delay(0.3).duration(120.0).attack(Strategy::Greedy, 0.49, Reveal::Every(20.0), 10.0) => "2529 265 0.00% 0.00 0.00 6.07s 1.98 0 0.30s 0.30 1275 32.3% 6@20s";
    #[ignore] half_pace45_rate1: Config::case(1.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0).stamp_pace(0.5) => "139 54 0.00% 0.00 0.00 10.14s 0.00 0 0.93s 0.99 58 16.3% 51@12s";
    #[ignore] half_pace49: Config::case(20.0).duration(120.0).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0).stamp_pace(0.5) => "2441 275 0.00% 0.00 0.00 21.74s 1.05 0 0.90s 0.90 1160 33.6% 1070@10s";
    #[ignore] double_pace45_rate1: Config::case(1.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0).stamp_pace(2.0) => "153 60 0.00% 0.00 0.00 9.30s 0.00 0 0.92s 0.94 66 11.7% 59@12s";
    #[ignore] double_pace49: Config::case(20.0).duration(120.0).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0).stamp_pace(2.0) => "2802 276 0.00% 0.18 0.00 20.02s 1.05 0 0.85s 0.79 1490 34.3% 1400@10s";
    #[ignore] balance45: Config::case(20.0).attack(Strategy::Balance, 0.45, Reveal::Immediately, 10.0) => "1153 129 0.00% 0.20 0.00 17.09s 1.15 0 0.93s 0.92 494 37.3% -";
    #[ignore] spine30: Config::case(20.0).attack(Strategy::Withhold, 0.3, Reveal::Immediately, 10.0) => "1167 133 0.00% 0.00 0.00 14.47s 1.10 0 0.92s 0.91 344 14.3% 294@10s";
    #[ignore] wh30: Config::case(20.0).attack(Strategy::Withhold, 0.3, Reveal::WhenAhead, 0.0) => "1235 115 0.00% 0.38 0.00 14.74s 0.73 0 0.88s 0.85 343 5.4% 14@0s";
    #[ignore] wh45: Config::case(20.0).attack(Strategy::Withhold, 0.45, Reveal::WhenAhead, 0.0) => "1278 105 0.00% 0.60 0.00 20.11s 0.66 0 0.89s 0.83 579 5.9% 21@0s";
    #[ignore] wh45e10: Config::case(20.0).attack(Strategy::Withhold, 0.45, Reveal::Every(10.0), 0.0) => "1294 103 0.00% 0.25 0.00 19.23s 0.65 0 0.89s 0.90 579 0.0% 6@10s";
    q2: Config::case(1.0).miners(2).duration(300.0) => "302 234 0.00% 0.00 0.00 5.43s 2.04 0 0.63s 0.63 0 0.0% -";
    q1s: Config::case(1.0).miners(1).duration(600.0).attack(Strategy::Withhold, 0.3, Reveal::Every(20.0), 0.0) => "779 466 0.00% 0.00 0.00 6.21s 0.00 0 0.39s 0.33 313 0.0% 30@21s";
    #[ignore] hash_halves_withhold: Config::case(20.0).duration(660.0).rate_switch(60.0, 10.0).attack(Strategy::Withhold, 0.3, Reveal::Every(20.0), 60.0) => "11644 1160 0.00% 0.12 0.00 18.03s 1.15 0 0.50s 0.47 4358 19.7% 31@60s";
    hash_doubles_spine: Config::case(1.0).duration(360.0).rate_switch(300.0, 2.0).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 300.0) => "415 229 9.19% 0.00 0.00 7.02s 6.98 0 0.74s 0.42 178 54.0% 54@302s";
    dag0_49_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2734 213 0.00% 0.13 0.00 35.86s 0.63 0 0.80s 0.69 1353 2.5% 28@0s";
    dag0_49_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2725 202 0.00% 0.00 0.00 37.01s 0.51 0 0.81s 0.68 1420 7.5% 41@0s";
    dag0_49_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2684 193 0.00% 0.00 0.00 35.41s 0.48 0 0.80s 0.69 1304 4.5% 33@0s";
    dag0_49_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2671 196 0.00% 0.08 0.00 35.60s 0.50 0 0.81s 0.69 1313 2.2% 29@0s";
    dag0_49_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Dag, 0.49, Reveal::WhenAhead, 0.0) => "2690 200 0.00% 0.38 0.00 36.56s 0.59 0 0.80s 0.69 1349 1.6% 32@0s";
    #[ignore] spine0_45_s1: Config::case(20.0).seed(1).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1169 134 0.00% 0.00 0.00 16.89s 1.15 0 0.93s 0.92 508 35.9% 430@10s";
    #[ignore] spine0_47_s1: Config::case(20.0).seed(1).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1169 167 0.00% 0.05 0.00 17.87s 0.80 0 0.95s 0.94 537 52.2% 452@10s";
    #[ignore] spine0_49_s1: Config::case(20.0).seed(1).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1169 167 0.00% 0.05 0.00 18.46s 1.05 0 0.95s 0.94 567 55.2% 477@10s";
    #[ignore] spine0_45_s2: Config::case(20.0).seed(2).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1258 126 0.00% 0.00 0.00 18.50s 0.80 0 0.93s 0.93 586 31.0% 497@10s";
    #[ignore] spine0_47_s2: Config::case(20.0).seed(2).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1257 171 0.00% 0.35 0.00 18.67s 0.55 0 0.95s 0.95 615 54.4% 523@10s";
    #[ignore] spine0_49_s2: Config::case(20.0).seed(2).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1257 146 0.00% 0.00 0.00 18.58s 0.85 0 0.94s 0.94 638 42.5% 544@10s";
    #[ignore] spine0_45_s3: Config::case(20.0).seed(3).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1264 172 0.00% 0.00 0.00 18.03s 0.45 0 0.95s 0.96 581 50.5% 490@10s";
    #[ignore] spine0_47_s3: Config::case(20.0).seed(3).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1264 146 0.00% 0.00 0.00 18.16s 1.30 0 0.95s 0.95 606 44.5% 510@10s";
    #[ignore] spine0_49_s3: Config::case(20.0).seed(3).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1264 182 0.00% 0.10 0.00 18.80s 0.40 0 0.95s 0.96 633 54.3% 532@10s";
    #[ignore] spine0_45_s4: Config::case(20.0).seed(4).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1247 139 0.00% 0.60 0.00 17.81s 1.40 0 0.93s 0.93 571 39.6% 481@10s";
    #[ignore] spine0_47_s4: Config::case(20.0).seed(4).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1248 131 0.00% 0.60 0.00 17.76s 0.75 0 0.94s 0.94 587 31.3% 495@10s";
    #[ignore] spine0_49_s4: Config::case(20.0).seed(4).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1247 143 0.00% 0.10 0.00 18.95s 1.55 0 0.94s 0.94 621 42.7% 523@10s";
    #[ignore] spine0_45_s5: Config::case(20.0).seed(5).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1228 175 0.00% 0.20 0.00 17.46s 0.50 0 0.95s 0.95 553 48.6% 469@10s";
    spine0_47_s5: Config::case(20.0).seed(5).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1229 166 0.00% 0.05 0.00 17.98s 0.90 0 0.95s 0.95 575 53.0% 488@10s";
    #[ignore] spine0_49_s5: Config::case(20.0).seed(5).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1229 175 0.00% 0.00 0.00 18.36s 0.55 0 0.95s 0.95 605 52.0% 515@10s";
    #[ignore] spine0_45_s6: Config::case(20.0).seed(6).attack(Strategy::Withhold, 0.45, Reveal::Immediately, 10.0) => "1195 182 0.00% 0.25 0.00 18.35s 0.80 0 0.95s 0.95 557 52.8% 462@10s";
    #[ignore] spine0_47_s6: Config::case(20.0).seed(6).attack(Strategy::Withhold, 0.47, Reveal::Immediately, 10.0) => "1195 175 0.00% 0.10 0.00 18.92s 0.75 0 0.95s 0.94 579 58.9% 480@10s";
    #[ignore] spine0_49_s6: Config::case(20.0).seed(6).attack(Strategy::Withhold, 0.49, Reveal::Immediately, 10.0) => "1195 137 0.00% 0.00 0.00 19.45s 0.65 0 0.95s 0.94 602 32.9% 500@10s";
    #[ignore] wh47_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2700 219 0.00% 0.26 0.00 20.50s 0.50 0 0.80s 0.69 1296 4.0% 25@0s";
    #[ignore] wh47_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2741 220 0.00% 0.13 0.00 19.75s 0.57 0 0.80s 0.68 1357 9.8% 34@0s";
    #[ignore] wh47_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2663 207 0.00% 0.13 0.00 21.17s 0.70 0 0.79s 0.69 1229 0.0% 24@0s";
    #[ignore] wh47_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2630 221 0.00% 0.47 0.00 21.39s 0.73 0 0.80s 0.69 1218 6.3% 19@0s";
    #[ignore] wh47_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2613 203 0.00% 0.46 0.00 21.26s 0.48 0 0.80s 0.68 1268 2.1% 22@0s";
    #[ignore] wh47_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2607 204 0.00% 0.00 0.00 21.75s 0.65 0 0.80s 0.69 1206 0.0% 14@0s";
    #[ignore] wh47_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2753 208 0.00% 0.65 0.00 20.17s 0.64 0 0.80s 0.69 1264 2.1% 26@0s";
    #[ignore] wh47_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2631 206 0.00% 0.25 0.00 20.84s 0.60 0 0.81s 0.69 1243 6.3% 22@0s";
    #[ignore] wh47_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2620 199 0.00% 0.13 0.00 21.04s 0.60 0 0.80s 0.68 1279 7.1% 27@0s";
    #[ignore] wh47_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Withhold, 0.47, Reveal::WhenAhead, 0.0) => "2588 218 0.00% 0.08 0.00 21.63s 0.48 0 0.81s 0.69 1214 2.5% 24@0s";
    #[ignore] wh49_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2701 216 0.00% 0.47 0.00 20.26s 0.64 0 0.80s 0.68 1325 3.5% 25@0s";
    wh49_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2704 206 0.00% 0.17 0.00 19.58s 0.55 0 0.81s 0.69 1391 10.5% 34@0s";
    #[ignore] wh49_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2645 216 0.00% 0.00 0.00 21.13s 0.59 0 0.81s 0.69 1269 6.0% 25@0s";
    #[ignore] wh49_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2610 204 0.00% 0.00 0.00 21.11s 0.50 0 0.81s 0.69 1261 2.7% 21@0s";
    #[ignore] wh49_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2656 215 0.00% 0.00 0.00 20.66s 0.59 0 0.80s 0.68 1323 2.5% 23@0s";
    #[ignore] wh49_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2592 213 0.00% 0.42 0.00 21.36s 0.73 0 0.81s 0.69 1250 0.0% 14@0s";
    #[ignore] wh49_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2706 207 0.00% 0.47 0.00 20.19s 0.65 0 0.81s 0.70 1319 2.1% 27@0s";
    #[ignore] wh49_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2553 200 0.00% 0.17 0.00 21.27s 0.53 0 0.81s 0.69 1286 2.7% 22@0s";
    #[ignore] wh49_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2637 210 0.00% 0.08 0.00 20.86s 0.55 0 0.80s 0.68 1324 6.7% 27@0s";
    #[ignore] wh49_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Withhold, 0.49, Reveal::WhenAhead, 0.0) => "2608 196 0.00% 0.04 0.00 20.92s 0.62 0 0.81s 0.68 1275 2.8% 26@0s";
    #[ignore] hv0_40_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2692 243 0.00% 0.00 0.00 21.18s 0.84 0 0.79s 0.70 1128 11.3% 61@0s";
    #[ignore] hv0_40_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2636 231 0.00% 0.13 0.00 21.66s 1.25 0 0.79s 0.70 1105 6.0% 43@0s";
    #[ignore] hv0_40_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2561 256 0.00% 0.00 0.00 21.58s 0.75 0 0.81s 0.72 1041 23.4% 99@0s";
    #[ignore] hv0_40_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2566 246 0.00% 0.00 0.00 22.86s 0.62 0 0.80s 0.70 1013 11.7% 44@0s";
    #[ignore] hv0_40_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2601 280 0.00% 0.62 0.00 20.41s 1.30 0 0.81s 0.72 1086 33.8% 133@0s";
    #[ignore] hv0_40_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2588 226 0.00% 0.09 0.00 22.20s 1.20 0 0.80s 0.70 1044 1.9% 41@0s";
    #[ignore] hv0_40_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2708 234 0.00% 0.09 0.00 21.25s 1.25 0 0.79s 0.71 1107 15.7% 73@0s";
    #[ignore] hv0_40_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2639 235 0.00% 0.57 0.00 21.96s 0.80 0 0.80s 0.71 1069 17.1% 61@0s";
    #[ignore] hv0_40_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2567 272 0.00% 0.09 0.00 21.05s 1.05 0 0.80s 0.71 1059 32.3% 118@0s";
    #[ignore] hv0_40_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.40, Reveal::WhenAhead, 0.0) => "2472 280 0.00% 0.52 0.00 21.38s 1.14 0 0.80s 0.71 1016 36.1% 129@0s";
    #[ignore] hv0_43_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2616 313 0.00% 0.13 0.00 19.22s 1.00 0 0.81s 0.73 1136 46.1% 172@0s";
    #[ignore] hv0_43_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2618 250 0.00% 0.13 0.00 20.80s 0.67 0 0.80s 0.70 1193 16.2% 81@0s";
    #[ignore] hv0_43_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2589 244 0.00% 0.39 0.00 22.25s 0.90 0 0.81s 0.71 1121 23.6% 115@0s";
    #[ignore] hv0_43_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2515 278 0.00% 0.26 0.00 21.59s 1.10 0 0.81s 0.71 1044 34.9% 130@0s";
    #[ignore] hv0_43_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2470 382 0.00% 0.22 0.00 18.32s 1.84 0 0.83s 0.74 1079 64.2% 282@0s";
    #[ignore] hv0_43_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2582 217 0.00% 0.00 0.00 23.49s 0.75 0 0.79s 0.69 1105 0.0% 29@0s";
    #[ignore] hv0_43_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2641 303 0.00% 0.04 0.00 20.20s 0.71 0 0.81s 0.73 1134 36.4% 139@0s";
    #[ignore] hv0_43_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2552 228 0.00% 0.00 0.00 22.27s 0.70 0 0.80s 0.70 1122 18.7% 62@0s";
    #[ignore] hv0_43_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2486 277 0.00% 0.22 0.00 21.27s 1.00 0 0.81s 0.71 1079 33.7% 115@0s";
    #[ignore] hv0_43_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.43, Reveal::WhenAhead, 0.0) => "2540 283 0.00% 0.26 0.00 21.31s 0.80 0 0.81s 0.71 1117 32.7% 98@0s";
    #[ignore] hv0_45_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2660 236 0.00% 0.35 0.00 20.63s 1.15 0 0.80s 0.70 1203 21.5% 78@0s";
    #[ignore] hv0_45_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2667 244 0.00% 0.00 0.00 20.43s 0.67 0 0.80s 0.69 1263 17.1% 79@0s";
    #[ignore] hv0_45_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2529 556 0.00% 0.00 0.00 14.69s 1.26 0 0.87s 0.81 1123 80.5% 485@0s";
    #[ignore] hv0_45_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2591 249 0.00% 0.00 0.00 20.84s 2.05 0 0.81s 0.70 1149 23.3% 104@0s";
    #[ignore] hv0_45_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2482 467 0.00% 0.58 0.00 16.72s 1.15 0 0.86s 0.77 1143 73.4% 385@0s";
    #[ignore] hv0_45_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2549 371 0.00% 0.27 0.00 19.03s 0.80 0 0.83s 0.74 1112 52.6% 230@0s";
    #[ignore] hv0_45_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2688 251 0.00% 0.00 0.00 20.76s 1.10 0 0.80s 0.70 1173 16.4% 80@0s";
    #[ignore] hv0_45_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2562 236 0.00% 0.26 0.00 21.54s 0.58 0 0.80s 0.69 1162 19.0% 72@0s";
    #[ignore] hv0_45_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2453 340 0.00% 0.35 0.00 19.91s 1.19 0 0.83s 0.73 1138 54.4% 223@0s";
    #[ignore] hv0_45_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.45, Reveal::WhenAhead, 0.0) => "2540 272 0.00% 0.09 0.00 20.70s 0.84 0 0.81s 0.71 1161 34.0% 117@0s";
    #[ignore] hv47_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2620 344 0.00% 0.26 0.00 18.32s 1.35 0 0.82s 0.72 1233 51.7% 206@0s";
    #[ignore] hv47_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2473 693 0.00% 0.37 0.00 12.92s 0.45 0 0.89s 0.83 1190 86.3% 633@0s";
    hv47_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2629 248 0.00% 0.00 0.00 20.70s 0.95 0 0.81s 0.70 1224 22.7% 92@0s";
    #[ignore] hv47_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2613 248 0.00% 0.26 0.00 21.15s 0.75 0 0.81s 0.70 1210 17.2% 81@0s";
    #[ignore] hv47_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2423 868 0.00% 0.48 0.00 9.70s 1.75 0 0.94s 0.91 1143 93.6% 862@0s";
    #[ignore] hv47_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2434 1152 0.00% 0.00 0.00 7.94s 0.80 0 0.99s 0.99 1152 100.0% 1152@0s";
    #[ignore] hv47_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2578 496 0.00% 0.63 0.00 15.91s 1.20 0 0.85s 0.78 1168 72.1% 387@0s";
    #[ignore] hv47_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2525 281 0.00% 0.17 0.00 19.90s 1.34 0 0.81s 0.71 1189 39.5% 149@0s";
    #[ignore] hv47_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2492 342 0.00% 0.39 0.00 19.47s 0.90 0 0.83s 0.74 1217 55.7% 218@0s";
    hv47_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.47, Reveal::WhenAhead, 0.0) => "2570 243 0.00% 0.51 0.00 21.51s 0.68 0 0.80s 0.69 1197 18.0% 73@0s";
    #[ignore] hv49_s1: Config::case(20.0).duration(120.0).seed(1).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2585 443 0.00% 0.40 0.00 17.39s 1.05 0 0.84s 0.74 1255 66.4% 292@0s";
    #[ignore] hv49_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2641 238 0.00% 0.00 0.00 20.06s 0.85 0 0.81s 0.69 1340 20.3% 86@0s";
    #[ignore] hv49_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2468 1214 0.00% 0.00 0.00 7.71s 0.40 0 0.98s 0.99 1214 100.0% 1214@0s";
    #[ignore] hv49_s4: Config::case(20.0).duration(120.0).seed(4).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2574 246 0.00% 0.13 0.00 20.42s 1.50 0 0.81s 0.70 1241 26.3% 117@0s";
    #[ignore] hv49_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2413 1195 0.00% 0.00 0.00 7.98s 1.70 0 0.98s 0.98 1195 100.0% 1190@0s";
    #[ignore] hv49_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2594 191 0.00% 0.51 0.00 21.21s 0.78 0 0.81s 0.69 1255 0.0% 48@0s";
    #[ignore] hv49_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2683 241 0.00% 0.43 0.00 19.99s 1.50 0 0.81s 0.70 1306 24.6% 93@0s";
    #[ignore] hv49_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2363 843 0.00% 0.00 0.00 11.07s 1.48 0 0.91s 0.86 1183 92.4% 818@0s";
    #[ignore] hv49_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2602 252 0.00% 0.51 0.00 20.09s 1.00 0 0.81s 0.70 1313 32.7% 118@0s";
    #[ignore] hv49_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.49, Reveal::WhenAhead, 0.0) => "2459 360 0.00% 0.00 0.00 18.87s 1.09 0 0.83s 0.72 1190 57.1% 250@0s";
    #[ignore] hv47e20_s6: Config::case(20.0).duration(120.0).seed(6).attack(Strategy::Harvest, 0.47, Reveal::Every(20.0), 0.0) => "2540 206 0.00% 0.00 0.00 27.88s 0.64 0 0.81s 0.80 1232 0.0% 6@20s";
    #[ignore] hv47e20_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.47, Reveal::Every(20.0), 0.0) => "2628 200 0.00% 0.25 0.00 24.12s 0.60 0 0.81s 0.81 1274 0.0% 6@20s";
    #[ignore] hv47e20_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Harvest, 0.47, Reveal::Every(20.0), 0.0) => "2484 201 0.00% 0.10 0.00 24.77s 0.69 0 0.81s 0.80 1216 0.0% 6@20s";
    #[ignore] hv49e20_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Harvest, 0.49, Reveal::Every(20.0), 0.0) => "2529 193 0.00% 0.59 0.00 23.88s 0.64 0 0.82s 0.81 1281 0.0% 6@20s";
    #[ignore] hv49e20_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Harvest, 0.49, Reveal::Every(20.0), 0.0) => "2580 197 0.00% 0.05 0.00 23.91s 0.65 0 0.82s 0.81 1304 0.0% 6@20s";
    #[ignore] gr40e10_s2: Config::case(20.0).duration(120.0).seed(2).attack(Strategy::Greedy, 0.40, Reveal::Every(10.0), 0.0) => "2504 206 0.00% 0.00 0.00 32.10s 0.84 0 0.81s 0.80 1074 0.0% 12@10s";
    #[ignore] gr45e20_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Greedy, 0.45, Reveal::Every(20.0), 0.0) => "2458 190 0.00% 0.39 0.00 24.71s 0.59 0 0.81s 0.79 1165 0.0% 6@20s";
    #[ignore] gr47e20_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Greedy, 0.47, Reveal::Every(20.0), 0.0) => "2626 201 0.00% 0.30 0.00 27.32s 0.49 0 0.81s 0.81 1274 0.0% 6@20s";
    #[ignore] gr49e10_s8: Config::case(20.0).duration(120.0).seed(8).attack(Strategy::Greedy, 0.49, Reveal::Every(10.0), 0.0) => "2478 194 0.00% 0.00 0.00 37.11s 0.44 0 0.84s 0.84 1286 0.0% 12@10s";
    #[ignore] gr45a_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Greedy, 0.45, Reveal::WhenAhead, 0.0) => "2512 551 0.00% 0.32 0.00 16.36s 1.26 0 0.88s 0.82 1109 81.2% 487@0s";
    #[ignore] gr45e20_s10: Config::case(20.0).duration(120.0).seed(10).attack(Strategy::Greedy, 0.45, Reveal::Every(20.0), 0.0) => "2461 192 0.00% 0.24 0.00 28.12s 0.74 0 0.81s 0.80 1147 0.0% 6@20s";
    #[ignore] gr47a_s7: Config::case(20.0).duration(120.0).seed(7).attack(Strategy::Greedy, 0.47, Reveal::WhenAhead, 0.0) => "2537 489 0.00% 0.00 0.00 19.22s 1.20 0 0.85s 0.78 1151 73.1% 406@0s";
    #[ignore] gr47e20_s3: Config::case(20.0).duration(120.0).seed(3).attack(Strategy::Greedy, 0.47, Reveal::Every(20.0), 0.0) => "2553 198 0.00% 0.44 0.00 24.17s 0.55 0 0.81s 0.80 1224 0.0% 6@20s";
    #[ignore] gr49e10_s11: Config::case(20.0).duration(120.0).seed(11).attack(Strategy::Greedy, 0.49, Reveal::Every(10.0), 0.0) => "2551 210 0.00% 0.05 0.00 35.48s 0.55 0 0.82s 0.81 1261 0.0% 12@10s";
    #[ignore] gr49a_s5: Config::case(20.0).duration(120.0).seed(5).attack(Strategy::Greedy, 0.49, Reveal::WhenAhead, 0.0) => "2409 1191 0.00% 0.00 0.00 7.96s 1.70 0 0.98s 0.98 1191 100.0% 1190@0s";
    #[ignore] gr49e10_s9: Config::case(20.0).duration(120.0).seed(9).attack(Strategy::Greedy, 0.49, Reveal::Every(10.0), 0.0) => "2486 195 0.00% 0.10 0.00 36.96s 0.54 0 0.83s 0.82 1273 0.0% 12@10s";
}
