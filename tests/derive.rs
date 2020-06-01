use slock::*;

#[test]
fn derive() {
    #[derive(Slockable)]
    struct User {
        name: String,
        age: u32,
    }
}
