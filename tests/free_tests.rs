use wux::commands::free;

#[test]
fn free_dry_run_no_kill() {
    let result = free::run(65432, true, true);
    assert!(result.is_ok());
}

#[test]
fn free_nothing_on_port() {
    let result = free::run(65433, false, true);
    assert!(result.is_ok());
}
