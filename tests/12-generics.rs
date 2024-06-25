use builders::*;

#[derive(Builder,Getters,Setters)]
pub struct Command<'a,T: Clone> {
    #[builder(each = "arg")]
    args: Vec<&'a str>,
    i: T,
}

fn main() {
    let command: Command<'_,i32> = Command::builder()
        .arg("build")
        .i(12)
        .build()
        .unwrap();
    assert_eq!(command.get_args(), &["build"]);
    assert_eq!(*command.get_i(), 12);
}
