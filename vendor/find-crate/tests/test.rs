#![warn(rust_2018_idioms, single_use_lifetimes)]

use find_crate::{Dependencies, Manifest};
use semver::{Version, VersionReq};

#[test]
fn dependencies() {
    const MANIFEST: &str = r#"
        [dependencies]
        foo = "0.1"

        [dev-dependencies.foo]
        version = "0.1.1"

        [build-dependencies]
        bar = "0.2"
    "#;

    const NAME1: &str = "foo";
    const NAME2: &str = "bar";
    const NAME3: &str = "baz";

    let mut manifest = Manifest::from_toml(toml::from_str(MANIFEST).unwrap());

    assert_eq!(Dependencies::Default, manifest.dependencies);

    assert_eq!(NAME1, manifest.find(|s| s == NAME1).unwrap().name);
    assert_eq!("0.1", manifest.find(|s| s == NAME1).unwrap().version);

    manifest.dependencies = Dependencies::Dev;
    assert_eq!(NAME1, manifest.find(|s| s == NAME1).unwrap().name);
    assert_eq!("0.1.1", manifest.find(|s| s == NAME1).unwrap().version);

    manifest.dependencies = Dependencies::Build;
    assert_eq!(None, manifest.find(|s| s == NAME1));

    assert_eq!(NAME2, manifest.find(|s| s == NAME2).unwrap().name);
    assert_eq!("0.2", manifest.find(|s| s == NAME2).unwrap().version);

    manifest.dependencies = Dependencies::Default;
    assert_eq!(None, manifest.find(|s| s == NAME2));

    manifest.dependencies = Dependencies::All;
    assert_eq!(NAME2, manifest.find(|s| s == NAME2).unwrap().name);
    assert_eq!("0.2", manifest.find(|s| s == NAME2).unwrap().version);

    assert_eq!(None, manifest.find(|s| s == NAME3));
}

#[test]
fn renamed() {
    const MANIFEST: &str = r#"
        [dependencies]
        foo-renamed = { package = "foo", version = "0.1" }

        [dependencies.bar_renamed]
        package = "bar"
        version = "0.2"
    "#;

    const NAME1: &str = "foo";
    const NAME2: &str = "bar";

    let manifest = Manifest::from_toml(toml::from_str(MANIFEST).unwrap());

    assert_eq!("foo_renamed", manifest.find(|s| s == NAME1).unwrap().name);
    assert_eq!("0.1", manifest.find(|s| s == NAME1).unwrap().version);

    assert_eq!("bar_renamed", manifest.find(|s| s == NAME2).unwrap().name);
    assert_eq!("0.2", manifest.find(|s| s == NAME2).unwrap().version);
}

#[test]
fn target() {
    const MANIFEST: &str = r#"
        [target.'cfg(target_os = "linux")'.dependencies]
        foo = "0.1"

        [target.'cfg(target_os = "macos")'.dependencies]
        bar = { version = "0.2" }

        [target.x86_64-unknown-linux-gnu.dependencies.baz]
        version = "0.3"
    "#;

    const NAME1: &str = "foo";
    const NAME2: &str = "bar";
    const NAME3: &str = "baz";

    let manifest = Manifest::from_toml(toml::from_str(MANIFEST).unwrap());

    assert_eq!(NAME1, manifest.find(|s| s == NAME1).unwrap().name);
    assert_eq!("0.1", manifest.find(|s| s == NAME1).unwrap().version);

    assert_eq!(NAME2, manifest.find(|s| s == NAME2).unwrap().name);
    assert_eq!("0.2", manifest.find(|s| s == NAME2).unwrap().version);

    assert_eq!(NAME3, manifest.find(|s| s == NAME3).unwrap().name);
    assert_eq!("0.3", manifest.find(|s| s == NAME3).unwrap().version);
}

#[test]
fn find2() {
    fn check(req: &str, version: &Version) -> bool {
        VersionReq::parse(req).unwrap().matches(version)
    }

    const MANIFEST: &str = r#"
        [dependencies]
        foo = "0.1"
        bar = "0.2"
        baz = { path = ".." }
    "#;

    const NAME1: &str = "foo";
    const NAME2: &str = "bar";
    const NAME3: &str = "baz";

    let manifest = Manifest::from_toml(toml::from_str(MANIFEST).unwrap());

    let version = Version::parse("0.2.0").unwrap();

    assert_eq!(None, manifest.find2(|s, v| s == NAME1 && check(v, &version)));

    assert_eq!(NAME2, manifest.find2(|s, v| s == NAME2 && check(v, &version)).unwrap().name);
    assert_eq!("0.2", manifest.find2(|s, v| s == NAME2 && check(v, &version)).unwrap().version);

    assert_eq!(NAME3, manifest.find2(|s, v| s == NAME3 && check(v, &version)).unwrap().name);
    assert_eq!("*", manifest.find2(|s, v| s == NAME3 && check(v, &version)).unwrap().version);
}

#[test]
fn crate_name() {
    const MANIFEST: &str = r#"
    [package]
    name = "crate-name"
    version = "0.1.0"
    "#;

    let manifest = Manifest::from_toml(toml::from_str(MANIFEST).unwrap());
    let package = manifest.crate_package().unwrap();
    assert_eq!("crate_name", package.name);
    assert_eq!("0.1.0", package.version);
}
