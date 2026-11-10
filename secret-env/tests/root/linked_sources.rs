// SPDX-License-Identifier: Apache-2.0
use std::time::{Duration, Instant};

#[test]
fn the_linked_secret_sources_are_plugins_on_the_secret_table() {
    const CHILD: &str = "BUSBAR_CENTRAL2_LINKED_CHILD";
    const WITNESS: &str = "actual linked product assertions completed";
    if std::env::var_os(CHILD).is_none() {
        struct Child(std::process::Child);
        impl Drop for Child {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }
        let name = std::thread::current()
            .name()
            .expect("the test runner names its test thread")
            .to_string();
        let mut child = Child(
            std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", &name, "--nocapture"])
                .env(CHILD, "1")
                .stdout(std::process::Stdio::piped())
                .spawn()
                .expect("start the isolated actual-root test"),
        );
        let until = Instant::now() + Duration::from_secs(60);
        loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                use std::io::Read;
                let mut output = String::new();
                child
                    .0
                    .stdout
                    .take()
                    .unwrap()
                    .read_to_string(&mut output)
                    .unwrap();
                assert!(
                    status.success(),
                    "the actual-root usage witness failed: {status}: {output}"
                );
                assert!(
                    output.contains(WITNESS),
                    "the exact child did not complete its assertions: {output}"
                );
                return;
            }
            assert!(
                Instant::now() < until,
                "the actual-root usage witness timed out"
            );
            std::thread::park_timeout(Duration::from_millis(5));
        }
    }
    crate::dist_root_install::install();
    use busbar_kernel::config::secret::SecretResolver;
    let reg = busbar_kernel::preflight::plugins_preflight(
        None,
        None,
        &Default::default(),
        &Default::default(),
        &busbar_kernel::config::PluginsCfg::default(),
        &Default::default(),
    )
    .expect("the default boot registers its linked rows");
    for name in ["env", "file"] {
        assert!(reg.resolve(name).is_none(), "{name} is no cold-kind row");
        assert!(
            busbar_kernel::preflight::root_rows()
                .secret_axis
                .expect("actual secret axis")
                .linked(name),
            "{name} is a linked secret plugin"
        );
    }
    let var = "BUSBAR_K5B_LINKED_SECRET_ROW";
    std::env::set_var(var, "hunter2");
    let resolver = SecretResolver::builtins_only();
    assert_eq!(
        busbar_contract::secret::SecretResolve::resolve(
            &resolver,
            &crate::distribution::config_validation_application::source_fill(var)
        )
        .unwrap(),
        busbar_contract::redacted::Secret::new(b"hunter2".to_vec())
    );
    let missing = busbar_contract::secret::SecretResolve::resolve(
        &resolver,
        &crate::distribution::config_validation_application::source_fill(
            "BUSBAR_K5B_NO_SUCH_VARIABLE",
        ),
    )
    .expect_err("an unset variable refuses");
    assert_eq!(
        missing,
        "secret env:BUSBAR_K5B_NO_SUCH_VARIABLE cannot resolve: environment variable \
         'BUSBAR_K5B_NO_SUCH_VARIABLE' is unset"
    );

    let cold = SecretResolver::with_plugin(Box::new(|_, _| {
        panic!("a linked actual secret source must never delegate to the cold fallback")
    }));
    assert_eq!(
        busbar_contract::secret::SecretResolve::resolve(
            &cold,
            &crate::distribution::config_validation_application::source_fill(var)
        )
        .unwrap(),
        busbar_contract::redacted::Secret::new(b"hunter2".to_vec())
    );
    std::env::remove_var(var);
    println!("{WITNESS}");
}
