use deep_causality_num::Num;
use deep_causality_num::Quaternion;

#[test]
fn test_num_trait() {
    // Num is a marker trait, so we just test that it can be used.
    fn takes_num<T: Num>(_: T) {}
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    takes_num(q);
}
