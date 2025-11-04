use deep_causality_num::Quaternion;

#[test]
fn test_display_format() {
    let q = Quaternion::new(1.0, 2.0, 3.0, 4.0);
    let display_str = format!("{}", q);
    assert_eq!(display_str, "1 + 2i + 3j + 4k");
}
