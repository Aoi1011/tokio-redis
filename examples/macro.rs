macro_rules! avec {
    () => {
        Vec::new()
    };
}

fn main() {
    let a: Vec<u32> = avec!();
    assert!(a.is_empty());
}
