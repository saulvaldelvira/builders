use builders::Constructor;

#[derive(Constructor,Debug)]
struct S {
    i: i32,
    s: String,
}


fn main() {
    let s = S::new(12,"as");
    assert_eq!(s.i,12);
    assert_eq!(s.s,"as");
}
