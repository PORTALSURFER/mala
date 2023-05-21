use crate::renderer::Renderer;

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
fn test_create_default_pipeline() {
    let renderer = Renderer::new_blocking().unwrap();
    let shader = renderer.create_default_shader();
    let pipeline = renderer.create_default_pipeline(&shader);
    assert!(pipeline.is_ok(), "Pipeline was not created");
}
