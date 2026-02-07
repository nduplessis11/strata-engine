// Window management and event handling

use crate::Result;

pub struct WindowManager {
    title: String,
    width: u32,
    height: u32,
}

impl WindowManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            title: "Strata Engine".to_string(),
            width: 800,
            height: 600,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_manager_creation() {
        let wm = WindowManager::new();
        assert!(wm.is_ok(), "WindowManager creation should succeed");
    }

    #[test]
    fn test_window_config() {
        let wm = WindowManager::new().expect("Failed to create WindowManager");

        assert_eq!(wm.title(), "Strata Engine");
        assert_eq!(wm.size(), (800, 600));
    }
}
