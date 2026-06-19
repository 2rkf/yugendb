#![cfg(feature = "memory")]

use yugendb::prelude::Driver;

#[test]
fn facade_exposes_memory_driver() {
    let driver = yugendb::drivers::memory();
    assert_eq!(driver.name(), "memory");

    let typed_driver = yugendb::drivers::MemoryDriver::new();
    assert_eq!(typed_driver.name(), "memory");
}
