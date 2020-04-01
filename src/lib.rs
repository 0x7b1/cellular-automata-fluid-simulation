#[test]
fn test() {
    let v1 = vec![1, 2, 3];
    let v2 = vec![10, 10, 10];

    let iter: Vec<_> = v1.iter().zip(v2.iter()).collect();
}