use std::{collections::HashMap, fs, path::Path};

pub fn read_workspace_deps(path: Option<impl AsRef<Path> + Copy>) -> HashMap<String, toml::Value> {
    let path = path
        .as_ref()
        .map_or(Path::new("./Cargo.toml"), |path| path.as_ref());
    let workspace_data: toml::Table =
        toml::from_str(&fs::read_to_string(path).expect("Error reading file"))
            .expect("Error parsing toml");
    workspace_data
        .get("workspace")
        .expect("Not a workspace")
        .get("dependencies")
        .expect("Missing workspace dependencies")
        .as_table()
        .expect("Expected table")
        .into_iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect::<HashMap<String, toml::Value>>()
}

pub fn list_package_workspace_deps(
    manifest_path: impl AsRef<Path>,
) -> impl Iterator<Item = String> {
    let mut manifest: toml::Table =
        toml::from_str(&fs::read_to_string(manifest_path).expect("Error reading file"))
            .expect("Error parsing Cargo.toml");
    let dependencies = manifest
        .remove("dependencies")
        .unwrap_or(toml::Value::Table(toml::Table::new()));
    let dev_dependencies = manifest
        .remove("dev-dependencies")
        .unwrap_or(toml::Value::Table(toml::Table::new()));
    let build_dependencies = manifest
        .remove("build-dependencies")
        .unwrap_or(toml::Value::Table(toml::Table::new()));
    match dependencies {
        toml::Value::Table(table) => table.into_iter().map(|(k, _)| k),
        _ => panic!("Expected table"),
    }
    .chain(match dev_dependencies {
        toml::Value::Table(table) => table.into_iter().map(|(k, _)| k),
        _ => panic!("Expected table"),
    })
    .chain(match build_dependencies {
        toml::Value::Table(table) => table.into_iter().map(|(k, _)| k),
        _ => panic!("Expected table"),
    })
    .chain(
        // All target-conditional dependencies
        manifest
            .get("target")
            .and_then(|val| val.as_table())
            .into_iter()
            .flat_map(|target_table| {
                target_table
                    .iter()
                    .filter_map(|(_config_condition, config_values)| {
                        config_values
                            .as_table()
                            .and_then(|table| table.get("dependencies"))
                            .and_then(|dependencies| dependencies.as_table())
                    })
                    .flat_map(toml::Table::keys)
            })
            .cloned()
            .collect::<Vec<String>>(),
    )
}

pub fn remove_deps_from_workspace(manifest_path: impl AsRef<Path>, deps: &[String]) {
    let mut manifest = fs::read_to_string(&manifest_path)
        .expect("Failed to read manifest")
        .parse::<toml_edit::DocumentMut>()
        .expect("Failed to parse manifest as TOML");
    let workspace_dependencies_table = manifest
        .get_mut("workspace")
        .expect("Missing workspace table in toml file")
        .as_table_like_mut()
        .expect("`workspace` should be a table")
        .get_mut("dependencies")
        .expect("Missing `workspace.dependencies` table in toml file")
        .as_table_like_mut()
        .expect("`workspace.dependencies` should be a table");
    for dep_to_remove in deps {
        workspace_dependencies_table
            .remove(dep_to_remove)
            .expect("Tried to remove dependency that doesn't exist");
    }
    fs::write(manifest_path, manifest.to_string().as_bytes())
        .expect("Failed to write back the changed dependencies");
}
