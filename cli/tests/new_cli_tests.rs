use unifmu_macros::for_each_fmu;

#[for_each_fmu]
#[test]
fn test_some_name() {
    println!("LOUDLY ________________!")
}