pub struct Renderer {
    is_initialized: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            is_initialized: true,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_initialization() {
        let renderer = Renderer::new();
        assert!(renderer.is_initialized(), "Renderer was not initialized");
    }
}