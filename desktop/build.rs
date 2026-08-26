use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");

    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/app-icons/icon.ico");
        res.set("ProductName", "Orbien Desktop");
        res.set("FileDescription", "Orbien Desktop");
        res.set("ProductVersion", &version);
        res.set("FileVersion", &version);
        res.compile()
            .expect("failed to compile Windows resources (icon)");
    }

    #[cfg(target_os = "macos")]
    {
        embed_macos_plist(&version);
    }

    generate_slint_version(&version);
    generate_slint_i18n();
    slint_build::compile("ui/app.slint").expect("slint compile failed");
}

fn generate_slint_version(version: &str) {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let out = manifest.join("ui/version.slint");
    let content = format!(
        "// @generated from CARGO_PKG_VERSION — do not edit.\n\
         export global AppMeta {{\n\
             out property <string> version: \"{version}\";\n\
         }}\n"
    );
    fs::write(&out, content).expect("write ui/version.slint");
}

#[cfg(target_os = "macos")]
fn embed_macos_plist(version: &str) {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let src = manifest.join("macos/Info.plist");
    println!("cargo:rerun-if-changed={}", src.display());
    let raw = fs::read_to_string(&src).expect("macos/Info.plist missing");
    let rendered = raw.replace("__ORB_VERSION__", version);
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let out = out_dir.join("Info.plist");
    fs::write(&out, rendered).expect("write OUT_DIR/Info.plist");
    println!(
        "cargo:rustc-link-arg=-Wl,-sectcreate,__TEXT,__info_plist,{}",
        out.display()
    );
}

fn generate_slint_i18n() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let zh_path = manifest.join("i18n/zh_CN.properties");
    let en_path = manifest.join("i18n/en_US.properties");
    println!("cargo:rerun-if-changed={}", zh_path.display());
    println!("cargo:rerun-if-changed={}", en_path.display());

    let zh = parse_properties(&fs::read_to_string(&zh_path).expect("read zh_CN.properties"));
    let en = parse_properties(&fs::read_to_string(&en_path).expect("read en_US.properties"));

    for key in zh.keys() {
        assert!(en.contains_key(key), "en_US.properties missing key: {key}");
    }
    for key in en.keys() {
        assert!(zh.contains_key(key), "zh_CN.properties missing key: {key}");
    }

    let keys: Vec<&String> = zh.keys().filter(|k| !k.starts_with("msg.")).collect();

    let out = manifest.join("ui/i18n.slint");
    fs::write(&out, render_slint(&keys, &zh, &en)).expect("write ui/i18n.slint");
}

fn parse_properties(raw: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            panic!("invalid properties line (expected key=value): {line}");
        };
        let key = key.trim().to_string();
        let value = value.trim().replace("\\n", "\n").replace("\\t", "\t");
        assert!(!key.is_empty(), "empty key in properties");
        assert!(
            map.insert(key.clone(), value).is_none(),
            "duplicate properties key: {key}"
        );
    }
    map
}

fn escape_slint(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn render_slint(
    keys: &[&String],
    zh: &BTreeMap<String, String>,
    en: &BTreeMap<String, String>,
) -> String {
    let mut out = String::from(
        "// @generated from i18n/*.properties — do not edit.\n\
         // locale-index: 0 = zh_CN, 1 = en_US\n\
         export global Tr {\n\
             in-out property <int> locale-index: 0;\n\
             property <bool> zh: locale-index == 0;\n\n",
    );
    for key in keys {
        let z = escape_slint(zh.get(*key).unwrap());
        let e = escape_slint(en.get(*key).unwrap());
        if z == e {
            out.push_str(&format!("    out property <string> {key}: \"{z}\";\n"));
        } else {
            out.push_str(&format!(
                "    out property <string> {key}: zh ? \"{z}\" : \"{e}\";\n"
            ));
        }
    }
    out.push_str("}\n");
    out
}
