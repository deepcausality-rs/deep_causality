use deep_causality_num::Quaternion;

#[test]
fn test_display_format() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let display_str = format!("{}", q);
    assert_eq!(display_str, "1 + 2i + 3j + 4k");

    let q_neg = Quaternion::new(1.0, -2.0, 3.0, -4.0);
    let display_str_neg = format!("{}", q_neg);
    assert_eq!(display_str_neg, "1 - 2i + 3j - 4k");
}
