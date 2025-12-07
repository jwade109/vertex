use crate::secret_project::*;

#[derive(Deserialize, Serialize, Debug)]
pub struct ReferenceImage {
    pub path: String,
    pub pos: Vec2,
}
