use deep_causality_algebra::Real;
use deep_causality_num::{BFloat16, Float106, ToPrimitive};

/// Words consumed if the loop stops when the next contribution falls below the scalar's own
/// resolution — no table, no declaration, using only `Real::epsilon()`.
fn words<T: Real + ToPrimitive>() -> u32 {
    let word_scale = 2f64.powi(-53);
    let eps = T::epsilon().to_f64().unwrap();
    let mut scale = word_scale;
    let mut n = 1;
    while scale > eps {
        n += 1;
        scale *= word_scale;
    }
    n
}

#[test]
fn probe() {
    for (name, eps, w) in [
        ("BFloat16", BFloat16::epsilon().to_f64(), words::<BFloat16>()),
        ("f32", <f32 as Real>::epsilon() as f64, words::<f32>()),
        ("f64", <f64 as Real>::epsilon(), words::<f64>()),
        ("Float106", Float106::epsilon().to_f64(), words::<Float106>()),
    ] {
        println!("PROBE {name:9} epsilon = {eps:.6e}  -> words = {w}");
    }
}
