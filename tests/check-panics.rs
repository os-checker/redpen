#[test]
fn check() {
    // Pass `BLESS=1` to update stderr.
    let bless = env::var("BLESS").is_ok_and(|x| !x.trim().is_empty());
    // let bless = true;
    run_ui_tests(bless);
}

use std::{env, path::PathBuf, sync::LazyLock};

static PROFILE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let current_exe_path = env::current_exe().unwrap();
    let deps_path = current_exe_path.parent().unwrap();
    let profile_path = deps_path.parent().unwrap();
    profile_path.into()
});

fn run_ui_tests(bless: bool) {
    let mut config = compiletest::Config {
        bless,
        edition: Some("2015".into()),
        mode: compiletest::common::Mode::Ui,
        ..Default::default()
    };

    config.target_rustcflags = Some(
        "--crate-type=lib -Zcrate-attr=feature(register_tool) -Zcrate-attr=register_tool(redpen)"
            .into(),
    );

    config.src_base = "tests/ui".into();
    config.build_base = PROFILE_PATH.join("test/ui");
    config.rustc_path = PROFILE_PATH.join("redpen");
    config.link_deps(); // Populate config.target_rustcflags with dependencies on the path

    compiletest::run_tests(&config);
}
