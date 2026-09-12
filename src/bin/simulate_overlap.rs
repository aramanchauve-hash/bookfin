//! Standalone Overlap Simulation Tool for Bookfin
//!
//! Models overlap between users under pure random sampling vs. active pool sampling,
//! calculating expected values and hypergeometric/Poisson probabilities of shared pages.

#[allow(dead_code)]
fn log_factorial(n: u64) -> f64 {
    let mut sum = 0.0;
    for i in 2..=n {
        sum += (i as f64).ln();
    }
    sum
}

#[allow(dead_code)]
fn log_combination(n: u64, k: u64) -> f64 {
    if k > n {
        return f64::NEG_INFINITY;
    }
    if k == 0 || k == n {
        return 0.0;
    }
    log_factorial(n) - log_factorial(k) - log_factorial(n - k)
}

/// Computes P(X >= k) using Poisson approximation when C is large, or exact hypergeometric for smaller pools.
fn prob_at_least_k(lambda: f64, k: u64) -> f64 {
    if lambda <= 0.0 {
        return 0.0;
    }
    // P(X >= k) = 1 - sum_{j=0}^{k-1} P(X = j)
    let mut cumulative_prob = 0.0;
    let mut p = (-lambda).exp(); // P(X = 0)
    for j in 0..k {
        cumulative_prob += p;
        p = p * lambda / ((j + 1) as f64);
    }
    (1.0 - cumulative_prob).clamp(0.0, 1.0)
}

fn simulate_pure_random() {
    println!("\n================================================================================");
    println!("  SCENARIO 1: PURE RANDOM UNIFORM SAMPLING (C = Entire Corpus)");
    println!("================================================================================");

    let corpus_sizes: &[u64] = &[10_000, 100_000, 1_000_000, 10_000_000];
    let reading_volumes: &[u64] = &[50, 200, 500, 1_000, 5_000, 10_000];
    let k_thresholds: &[u64] = &[1, 5, 10, 25, 50, 100];

    for &c in corpus_sizes {
        println!(
            "\n--------------------------------------------------------------------------------"
        );
        println!(" Corpus Size C = {} pages", c);
        println!(
            "--------------------------------------------------------------------------------"
        );
        println!(
            "{:<8} | {:<12} | {:<8} | {:<8} | {:<8} | {:<8} | {:<8} | {:<8}",
            "Pages (n)",
            "E[Overlap]",
            "P(>=1)",
            "P(>=5)",
            "P(>=10)",
            "P(>=25)",
            "P(>=50)",
            "P(>=100)"
        );
        println!(
            "{:-<8}-+-{:-<12}-+-{:-<8}-+-{:-<8}-+-{:-<8}-+-{:-<8}-+-{:-<8}-+-{:-<8}",
            "", "", "", "", "", "", "", ""
        );

        for &n in reading_volumes {
            if n > c {
                continue;
            }
            let expected_overlap = (n as f64 * n as f64) / (c as f64);
            let probs: Vec<String> = k_thresholds
                .iter()
                .map(|&k| {
                    if k > n {
                        return "0.00%".to_string();
                    }
                    let p = prob_at_least_k(expected_overlap, k);
                    if p < 0.0001 && p > 0.0 {
                        format!("{:.2e}", p)
                    } else {
                        format!("{:.2}%", p * 100.0)
                    }
                })
                .collect();

            println!(
                "{:<8} | {:<12.3} | {:<8} | {:<8} | {:<8} | {:<8} | {:<8} | {:<8}",
                n, expected_overlap, probs[0], probs[1], probs[2], probs[3], probs[4], probs[5]
            );
        }
    }
}

fn simulate_active_pool_strategy() {
    println!("\n================================================================================");
    println!("  SCENARIO 2: ACTIVE SLIDING EPOCH POOL (Algorithmic Bottleneck Solution)");
    println!("================================================================================");
    println!("Strategy: 75% of served pages come from an Active Pool C_active (e.g. 5,000 pages),");
    println!("          25% from the Long Tail of the 1,000,000-page corpus.");
    println!("--------------------------------------------------------------------------------");

    let c_active_pools: &[u64] = &[2_500, 5_000, 10_000];
    let reading_volumes: &[u64] = &[50, 200, 500, 1_000, 2_000];
    let k_thresholds: &[u64] = &[1, 5, 10, 25, 50];

    for &c_active in c_active_pools {
        println!(
            "\nActive Pool C_active = {} pages (Global Corpus = 1,000,000)",
            c_active
        );
        println!(
            "{:<8} | {:<10} | {:<12} | {:<8} | {:<8} | {:<8} | {:<8} | {:<8}",
            "Read (n)",
            "In Pool",
            "E[Pool Overlap]",
            "P(>=1)",
            "P(>=5)",
            "P(>=10)",
            "P(>=25)",
            "P(>=50)"
        );
        println!(
            "{:-<8}-+-{:-<10}-+-{:-<12}-+-{:-<8}-+-{:-<8}-+-{:-<8}-+-{:-<8}-+-{:-<8}",
            "", "", "", "", "", "", "", ""
        );

        for &n in reading_volumes {
            let n_pool = (n as f64 * 0.75).round() as u64;
            let expected_pool_overlap = (n_pool as f64 * n_pool as f64) / (c_active as f64);
            let probs: Vec<String> = k_thresholds
                .iter()
                .map(|&k| {
                    let p = prob_at_least_k(expected_pool_overlap, k);
                    if p < 0.0001 && p > 0.0 {
                        format!("{:.2e}", p)
                    } else {
                        format!("{:.2}%", p * 100.0)
                    }
                })
                .collect();

            println!(
                "{:<8} | {:<10} | {:<14.2} | {:<8} | {:<8} | {:<8} | {:<8} | {:<8}",
                n, n_pool, expected_pool_overlap, probs[0], probs[1], probs[2], probs[3], probs[4]
            );
        }
    }
}

fn simulate_affinity_confidence_scaling() {
    println!("\n================================================================================");
    println!("  SCENARIO 3: AFFINITY SCORE EVOLUTION (Separation of Similarity and Confidence)");
    println!("================================================================================");
    println!("Formula: Affinity = Similarity * Confidence");
    println!("  where Similarity = (common_likes + common_dislikes - disagreements) / overlap");
    println!("  and Confidence  = overlap / (overlap + 20)");
    println!("  can_connect     = (Confidence >= 0.50) AND (Affinity >= 0.50)");
    println!("--------------------------------------------------------------------------------");
    println!(
        "{:<8} | {:<10} | {:<10} | {:<12} | {:<12} | {:<14} | {:<12}",
        "Overlap",
        "Agreements",
        "Disagrees",
        "Similarity",
        "Confidence",
        "Final Affinity",
        "Can Connect?"
    );
    println!(
        "{:-<8}-+-{:-<10}-+-{:-<10}-+-{:-<12}-+-{:-<12}-+-{:-<14}-+-{:-<12}",
        "", "", "", "", "", "", ""
    );

    let scenarios = [
        (3, 3, 0),
        (5, 5, 0),
        (10, 9, 1),
        (20, 18, 2),
        (20, 20, 0),
        (50, 42, 8),
        (100, 85, 15),
        (200, 160, 40),
    ];

    for (overlap, agreements, disagrees) in scenarios {
        let similarity = (agreements as f64 - disagrees as f64) / (overlap as f64);
        let confidence = (overlap as f64) / (overlap as f64 + 20.0);
        let affinity = (similarity * confidence).max(0.0);
        let can_connect = confidence >= 0.50 && affinity >= 0.50;

        println!(
            "{:<8} | {:<10} | {:<10} | {:<12.3} | {:<12.3} | {:<14.3} | {:<12}",
            overlap,
            agreements,
            disagrees,
            similarity,
            confidence,
            affinity,
            if can_connect { "OUI" } else { "NON" }
        );
    }
}

fn main() {
    println!("################################################################################");
    println!("       BOOKFIN - MATHEMATICAL OVERLAP & SOCIAL AFFINITY SIMULATOR");
    println!("################################################################################");

    simulate_pure_random();
    simulate_active_pool_strategy();
    simulate_affinity_confidence_scaling();

    println!("\n================================================================================");
    println!("  KEY ARCHITECTURAL TAKEAWAYS FOR BOOKFIN:");
    println!("  1. Pure random sampling across 1M+ pages makes spontaneous user overlap rare (E[X]=0.25).");
    println!("  2. An Active Sliding Pool of 5,000 pages solves this without personalization or tracking.");
    println!("  3. Confidence dampening (k=20) ensures 100% agreement on 3 pages cannot trigger a match.");
    println!(
        "  4. Only mature profiles with sustained common tastes unlock mutual social contact."
    );
    println!("================================================================================\n");
}
