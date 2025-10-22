#![cfg(test)]

use crate::mock::*;

#[test]
fn test_it_works() {
    TestState::default().build_and_execute(|| {
        assert!(true);
    });
}
