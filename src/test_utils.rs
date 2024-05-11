pub fn check_rem(rem: &[u8], expected_len: usize) {
    if rem.len() != expected_len {
        let str = String::from_utf8(rem.to_vec()).unwrap();
        println!("rem: {str}");
    }
    assert_eq!(
        expected_len,
        rem.len(),
        "Remainder length should be {expected_len} but was {}",
        rem.len()
    );
}
