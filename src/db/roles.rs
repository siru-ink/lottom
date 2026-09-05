// #[derive(Debug)]
// pub struct Role {
//     id: i32,
//     name: String,
// }

#[derive(Debug)]
pub enum Roles {
    owner,
    editor,
    viewer,
}

// impl Role {
//     pub async fn get_id_by_name(pool: &PgPool, name: &str) -> Option<Self> {
//         query_as!(Role, "SELECT * FROM roles WHERE name = $1", name,)
//             .fetch_optional(pool)
//             .await
//             .ok()?
//     }
// }

impl Roles {
    pub fn id(self) -> i32 {
        match self {
            Self::owner => 1,
            Self::editor => 2,
            Self::viewer => 3,
        }
    }
}
