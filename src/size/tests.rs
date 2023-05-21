use crate::size::Size;

#[test]
fn test_create_new_size() {
    let size = Size::new(100, 200);
    assert_eq!(size.width, 100);
    assert_eq!(size.height, 200);
}

#[test]
fn test_get_size_area() {
    let size_area = Size::new(100, 200).get_area();
    assert_eq!(size_area, 20000);
}