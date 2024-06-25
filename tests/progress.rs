#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse.rs");
    t.pass("tests/02-create-builder.rs");
    t.pass("tests/03-call-setters.rs");
    t.pass("tests/04-call-build.rs");
    t.pass("tests/05-method-chaining.rs");
    t.pass("tests/06-optional-field.rs");
    t.pass("tests/07-repeated-field.rs");
    t.compile_fail("tests/08-unrecognized-attribute.rs");
    t.pass("tests/09-redefined-prelude-types.rs");
    t.pass("tests/10-hash-map.rs");
    t.pass("tests/11-prefix.rs");
    t.pass("tests/12-generics.rs");
    t.pass("tests/constructor.rs");
}
