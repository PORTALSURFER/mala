use crate::vertex::Vertex;

#[test]
fn test_create_new_vertex() {
    let vertex = Vertex::new(1.0, 3.0);
    assert_eq!(vertex.position, [1.0, 3.0]);
}

