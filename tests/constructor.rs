use builders::Constructor;

#[derive(Constructor,Debug)]
struct S {
    i: i32,
    s: String,
    #[constructor = false]
    opt: Option<i32>,
}


fn main() {
    let s = S::new(12,"as");
    assert_eq!(s.i,12);
    assert_eq!(s.s,"as");
}
