use std::path::PathBuf;

use agentpack::deploy::{DesiredState, insert_desired_file};
use agentpack::user_error::UserError;

#[test]
fn insert_desired_file_merges_module_ids_when_bytes_match() -> anyhow::Result<()> {
    let mut desired = DesiredState::new();

    insert_desired_file(
        &mut desired,
        "codex",
        PathBuf::from("out.txt"),
        b"same".to_vec(),
        vec!["module:a".to_string()],
    )?;

    insert_desired_file(
        &mut desired,
        "codex",
        PathBuf::from("out.txt"),
        b"same".to_vec(),
        vec!["module:b".to_string()],
    )?;

    assert_eq!(desired.len(), 1);
    let file = desired.values().next().expect("one desired file");
    assert_eq!(file.bytes, b"same".to_vec());
    assert_eq!(
        file.module_ids,
        vec!["module:a".to_string(), "module:b".to_string()]
    );

    Ok(())
}

#[test]
fn insert_desired_file_errors_on_conflicting_bytes() {
    let mut desired = DesiredState::new();

    insert_desired_file(
        &mut desired,
        "codex",
        PathBuf::from("out.txt"),
        b"one".to_vec(),
        vec!["module:a".to_string()],
    )
    .expect("first insert ok");

    let err = insert_desired_file(
        &mut desired,
        "codex",
        PathBuf::from("out.txt"),
        b"two".to_vec(),
        vec!["module:b".to_string()],
    )
    .expect_err("second insert should conflict");

    let user = err.downcast_ref::<UserError>().expect("error is UserError");
    assert_eq!(user.code, "E_DESIRED_STATE_CONFLICT");
    assert!(user.message.contains("conflicting desired outputs"));
    assert!(user.details.is_some());
}
