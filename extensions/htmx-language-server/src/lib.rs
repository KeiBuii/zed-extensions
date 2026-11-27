use std::fs;

use zed_extension_api::{self as zed, LanguageServerId, Result};

const SERVER_NAME: &str = "htmx-lsp";
const GITHUB_REPO: &str = "ThePrimeagen/htmx-lsp";

struct HtmxExtension {
    cached_binary_path: Option<String>,
}

/// Name of the release asset for a platform, or `None` if upstream ships no
/// prebuilt binary for it (e.g. Linux arm64).
fn asset_name(os: zed::Os, arch: zed::Architecture) -> Option<&'static str> {
    use zed::{Architecture::*, Os::*};
    match (os, arch) {
        (Mac, Aarch64) => Some("htmx-lsp-darwin-arm64"),
        (Mac, X8664) => Some("htmx-lsp-darwin-x64"),
        (Linux, X8664) => Some("htmx-lsp-linux-x64"),
        (Windows, Aarch64) => Some("htmx-lsp-windows-arm64.exe"),
        (Windows, X8664) => Some("htmx-lsp-windows-x64.exe"),
        _ => None,
    }
}

impl HtmxExtension {
    fn binary_path(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<String> {
        // A user-installed binary (e.g. `cargo install htmx-lsp`) wins.
        if let Some(path) = worktree.which(SERVER_NAME) {
            return Ok(path);
        }

        if let Some(path) = &self.cached_binary_path
            && fs::metadata(path).is_ok_and(|m| m.is_file())
        {
            return Ok(path.clone());
        }

        let (os, arch) = zed::current_platform();
        let asset_name = asset_name(os, arch).ok_or_else(|| {
            format!(
                "{GITHUB_REPO} ships no prebuilt binary for this platform; \
                 install it with `cargo install htmx-lsp` and make sure it is on PATH"
            )
        })?;

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );
        let release = zed::latest_github_release(
            GITHUB_REPO,
            zed::GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )?;
        let asset = release
            .assets
            .iter()
            .find(|a| a.name == asset_name)
            .ok_or_else(|| format!("no asset named {asset_name} in release {}", release.version))?;

        let version_dir = format!("{SERVER_NAME}-{}", release.version);
        let binary_path = format!("{version_dir}/{asset_name}");

        if !fs::metadata(&binary_path).is_ok_and(|m| m.is_file()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            fs::create_dir_all(&version_dir)
                .map_err(|e| format!("failed to create {version_dir}: {e}"))?;
            zed::download_file(
                &asset.download_url,
                &binary_path,
                zed::DownloadedFileType::Uncompressed,
            )
            .map_err(|e| format!("failed to download {asset_name}: {e}"))?;
            zed::make_file_executable(&binary_path)?;

            // Drop binaries from older releases.
            if let Ok(entries) = fs::read_dir(".") {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name.starts_with(&format!("{SERVER_NAME}-")) && name != version_dir {
                        fs::remove_dir_all(entry.path()).ok();
                    }
                }
            }
        }

        self.cached_binary_path = Some(binary_path.clone());
        Ok(binary_path)
    }
}

impl zed::Extension for HtmxExtension {
    fn new() -> Self {
        Self {
            cached_binary_path: None,
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: self.binary_path(language_server_id, worktree)?,
            args: Vec::new(),
            env: Vec::new(),
        })
    }
}

zed::register_extension!(HtmxExtension);

#[cfg(test)]
mod tests {
    use super::*;
    use zed::{Architecture, Os};

    #[test]
    fn maps_supported_platforms_to_assets() {
        assert_eq!(
            asset_name(Os::Mac, Architecture::Aarch64),
            Some("htmx-lsp-darwin-arm64")
        );
        assert_eq!(
            asset_name(Os::Linux, Architecture::X8664),
            Some("htmx-lsp-linux-x64")
        );
        assert_eq!(
            asset_name(Os::Windows, Architecture::X8664),
            Some("htmx-lsp-windows-x64.exe")
        );
    }

    const MANIFEST: &str = include_str!("../extension.toml");
    const CANONICAL_SNIPPETS: &str = include_str!("../snippets/html.json");

    /// Quoted strings inside the `key = [ ... ]` array of the manifest.
    fn manifest_array(key: &str) -> Vec<String> {
        let start = MANIFEST
            .find(&format!("\n{key} = ["))
            .unwrap_or_else(|| panic!("no `{key}` array in extension.toml"));
        let body = &MANIFEST[start..];
        let body = &body[body.find('[').unwrap() + 1..body.find(']').unwrap()];
        body.split('"')
            .skip(1)
            .step_by(2)
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn snippets_are_valid_and_well_formed() {
        let json: zed::serde_json::Value =
            zed::serde_json::from_str(CANONICAL_SNIPPETS).expect("snippets/html.json is not JSON");
        let snippets = json.as_object().expect("top level must be an object");
        assert!(!snippets.is_empty());
        for (name, snippet) in snippets {
            let prefix = snippet["prefix"].as_str().unwrap_or_default();
            assert!(prefix.starts_with("htmx_"), "{name}: prefix {prefix:?}");
            // Zed's completion filter is fuzzy: a prefix containing h…x…- would
            // be offered (and picked) while typing an `hx-` attribute.
            assert!(
                !fuzzy_matches("hx-", prefix),
                "{name}: prefix {prefix:?} shadows `hx-` attribute completion"
            );
            assert!(snippet["description"].is_string(), "{name}: no description");
            let body = match &snippet["body"] {
                zed::serde_json::Value::String(s) => s.clone(),
                zed::serde_json::Value::Array(lines) => lines
                    .iter()
                    .map(|l| l.as_str().expect("body lines must be strings"))
                    .collect::<Vec<_>>()
                    .join("\n"),
                _ => panic!("{name}: body must be a string or array of lines"),
            };
            assert_no_bare_mirrored_tabstops(name, &body);
        }
    }

    fn fuzzy_matches(query: &str, candidate: &str) -> bool {
        let mut chars = candidate.chars();
        query.chars().all(|q| chars.any(|c| c == q))
    }

    /// Zed does not copy a placeholder's default text to later bare `$N`
    /// references (`<${1:div}></$1>` yields `</>`), so every occurrence of a
    /// tabstop that has a default must repeat it.
    fn assert_no_bare_mirrored_tabstops(name: &str, body: &str) {
        let bytes = body.as_bytes();
        for (i, _) in body.match_indices('$') {
            let digits: String = body[i + 1..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            if digits.is_empty() || digits == "0" || bytes.get(i + 1) == Some(&b'{') {
                continue;
            }
            assert!(
                !body.contains(&format!("${{{digits}:")),
                "{name}: bare ${digits} mirrors a placeholder; write ${{{digits}:default}} again"
            );
        }
    }

    #[test]
    fn mirrored_tabstop_check_catches_bare_reference() {
        let result = std::panic::catch_unwind(|| {
            assert_no_bare_mirrored_tabstops("t", "<${1:div}></$1>");
        });
        assert!(result.is_err());
        assert_no_bare_mirrored_tabstops("t", "<${1:div}>$0</${1:div}>");
        assert!(fuzzy_matches("hx-", "htmx-get"));
        assert!(!fuzzy_matches("hx-", "htmx_get"));
    }

    #[test]
    fn every_language_has_an_identical_snippet_file() {
        let languages = manifest_array("languages");
        let mut snippet_files = manifest_array("snippets");
        let mut expected: Vec<String> = languages
            .iter()
            .map(|l| format!("./snippets/{}.json", l.to_lowercase()))
            .collect();
        snippet_files.sort();
        expected.sort();
        assert_eq!(snippet_files, expected);

        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/");
        for file in &snippet_files {
            let contents = fs::read_to_string(format!("{dir}{file}"))
                .unwrap_or_else(|e| panic!("{file}: {e}"));
            assert!(
                contents == CANONICAL_SNIPPETS,
                "{file} differs from snippets/html.json; re-copy it"
            );
        }
    }

    #[test]
    fn unsupported_platform_has_no_asset() {
        assert_eq!(asset_name(Os::Linux, Architecture::Aarch64), None);
        assert_eq!(asset_name(Os::Mac, Architecture::X86), None);
    }
}
