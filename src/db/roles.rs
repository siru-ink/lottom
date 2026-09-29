#[derive(Debug)]
pub enum Roles {
    Owner,
    // Editor,
    // Viewer,
}

impl Roles {
    pub fn id(self) -> i32 {
        match self {
            Self::Owner => 1,
            // Self::Editor => 2,
            // Self::Viewer => 3,
        }
    }
}
