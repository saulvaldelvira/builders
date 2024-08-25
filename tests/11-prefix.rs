use builders::*;

#[derive(Builder,Getters,Setters)]
#[getters(prefix = "pre_")]
#[setters(prefix = "intoo_")]
pub struct Command {
    executable: String,
    args: Vec<String>,
    env: Vec<String>,
    #[builder(optional = true)]
    current_dir: Option<String>,
}

fn main() {
    let command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .build()
        .unwrap();
    assert!(command.pre_current_dir().is_none());

    let mut command = Command::builder()
        .executable("cargo".to_owned())
        .args(vec!["build".to_owned(), "--release".to_owned()])
        .env(vec![])
        .current_dir("..".to_owned())
        .build()
        .unwrap();
    command.intoo_executable("EXEC");
    assert!(command.pre_current_dir().is_some());
    assert_eq!(command.pre_executable(), "EXEC");
}
