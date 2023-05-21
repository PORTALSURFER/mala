use crate::renderer::Renderer;
use crate::size::Size;
use crate::vertex::Vertex;

#[test]
fn test_render_initialization() {
    let renderer = Renderer::new_blocking().unwrap();
    assert!(renderer.is_initialized(), "Renderer was not initialized");
}

#[test]
fn test_render_to_texture_on_disk() {
    let renderer = Renderer::new_blocking().unwrap();
    renderer.render_triangle().unwrap();
    renderer.save_to_texture_on_disk("output.png").unwrap();
    assert!(std::path::Path::new("output.png").exists(), "Texture was not saved on disk");
    //std::fs::remove_file("output.png").unwrap();
}

#[test]
fn test_save_to_texture_on_disk_with_bad_path() {
    let renderer = Renderer::new_blocking().unwrap();
    renderer.render_triangle().unwrap();
    assert!(renderer.save_to_texture_on_disk("badfolder/output.png").is_err(), "Texture was saved on disk with bad path");
}

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

#[test]
fn test_create_new_vertex() {
    let vertex = Vertex::new(1.0, 3.0);
    assert_eq!(vertex.position, [1.0, 3.0]);
}

#[test]
fn test_create_default_pipeline() {
    let renderer = Renderer::new_blocking().unwrap();
    let shader = renderer.create_default_shader();
    let pipeline = renderer.create_default_pipeline(&shader);
    assert!(pipeline.is_ok(), "Pipeline was not created");
}
