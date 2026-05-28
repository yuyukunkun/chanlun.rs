/// build.rs — 编译时版本一致性检查
///
/// 验证 Cargo.toml 和 pyproject.toml 的版本号是否一致。
///
/// Cargo.toml:     version = "YY.MM.patch"    (e.g., "26.5.57")
/// pyproject.toml: version = "YYMM.patch"      (e.g., "2605.57")
/// 规则: YY = major, MM = minor, patch = patch
fn main() {
    let cargo_manifest_dir =
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let cargo_path = std::path::PathBuf::from(&cargo_manifest_dir);

    let cargo_version = read_version(&cargo_path.join("Cargo.toml"), "[package]");
    let cargo_parts: Vec<&str> = cargo_version.split('.').collect();
    assert_eq!(
        cargo_parts.len(),
        3,
        "Cargo.toml version '{}' is not in YY.MM.patch format",
        cargo_version
    );

    let pyproject_version = read_version(&cargo_path.join("pyproject.toml"), "[project]");
    let py_parts: Vec<&str> = pyproject_version.split('.').collect();
    assert_eq!(
        py_parts.len(),
        2,
        "pyproject.toml version '{}' is not in YYMM.patch format",
        pyproject_version
    );

    let expected_yymm = format!("{}{:0>2}", cargo_parts[0], cargo_parts[1]);
    assert_eq!(
        py_parts[0], expected_yymm,
        "pyproject.toml version prefix '{}' != expected '{}' (from Cargo {})",
        py_parts[0], expected_yymm, cargo_version
    );
    assert_eq!(
        py_parts[1], cargo_parts[2],
        "pyproject.toml patch '{}' != Cargo.toml patch '{}'",
        py_parts[1], cargo_parts[2]
    );

    println!("cargo:rerun-if-changed=pyproject.toml");
    println!("cargo:rerun-if-changed=Cargo.toml");
}

fn read_version(path: &std::path::Path, section: &str) -> String {
    let content =
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("Cannot read {:?}: {}", path, e));

    let mut in_section = section.is_empty();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == section;
            continue;
        }
        if !in_section {
            continue;
        }
        if trimmed.starts_with("version") {
            if let Some(v) = trimmed.split('=').nth(1) {
                return v.trim().trim_matches('"').trim().to_string();
            }
        }
    }
    panic!("Cannot parse version from {:?}", path);
}
