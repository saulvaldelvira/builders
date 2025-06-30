use builders::Builder;

#[derive(Builder)]
#[builder(infallible)]
pub struct Command {
    #[builder(def = { String::new() })]
    executable: String,
    #[builder(vec = "lol")]
    env: Vec<String>,
    #[builder(optional = true)]
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder().build();
    assert!(command.current_dir.is_none());
}
