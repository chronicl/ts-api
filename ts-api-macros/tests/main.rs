#[test]
fn tests() {
    let t = trybuild::TestCases::new();

    t.pass("tests/test1.rs");
}
