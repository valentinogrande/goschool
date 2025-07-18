use crate::structs::{MySelf, Role};
use crate::traits::New;

impl New for MySelf {
    fn new(id: u64, role: Role) -> Self{
        Self { role, id }
    }
}


