use regex::Regex;

const REAL_TYPES: [&str; 9] = [
    "string",
    "boolean",
    "integer",
    "numeric",
    "enumeration",
    "set",
    "directory name",
    "file name",
    "byte",
];

/**
 * Clean type using real types
 * @return Option<String> The cleaned type
 */
pub fn clean_type(type_str: String) -> Option<String> {
    if type_str == "bool" {
        return Some("boolean".to_string());
    } else if type_str == "varchar"
        || type_str == "text"
        || type_str == "ip address"
        || type_str == "datetime"
        // ndb_recv_thread_cpu_mask documents itself as "Bitmap" but its on-disk
        // form is a comma-separated CPU-mask string (e.g. "0-3,5").
        || type_str == "bitmap"
    {
        return Some("string".to_string());
    } else if type_str == "filename" {
        return Some("file name".to_string());
    } else if type_str == "double" {
        return Some("numeric".to_string());
    } else if type_str == "bigint unsigned" || type_str == "int unsigned" || type_str == "unsigned long" {
        return Some("integer".to_string());
    }

    if !REAL_TYPES.into_iter().any(|t| t == type_str) {
        if type_str.contains("in bytes")
            || type_str.contains("number of bytes")
            || type_str.contains("size in mb")
            || type_str.contains("bytes read from")
            || type_str.contains("bytes written to")
        {
            return Some("byte".to_string());
        } else if type_str.contains("number of")
            || type_str.contains("size of")
            || type_str.contains("batch size")
            || type_str.contains("in microseconds")
            || type_str.contains("in seconds")
        {
            return Some("integer".to_string());
        } else if type_str.contains("numeric (64-bit unsigned integer)")
            || type_str.contains("numeric (32-bit unsigned integer)")
        {
            return Some("numeric".to_string());
        } else if type_str.contains("enum") {
            //enumerated
            return Some("enumeration".to_string());
        } else if type_str.contains("directory name")
            || type_str.contains("path name")
            || type_str.contains("path to")
            || type_str.ends_with("directory.")
        {
            return Some("directory name".to_string());
        } else if type_str.contains("filename") {
            return Some("file name".to_string());
        } else if type_str.ends_with("unused.") || type_str.contains("unused since") || type_str.ends_with("removed.") {
            return None;
        }

        if type_str.len() < 30 && !type_str.is_empty() {
            eprintln!("not known type: {type_str}");
        }

        return None;
    }
    Some(type_str)
}

pub fn get_clean_type_from_mixed_string(mixed_string: String) -> Option<String> {
    REAL_TYPES
        .into_iter()
        .find(|real_type_to_test| mixed_string.contains(real_type_to_test))
        .map(std::string::ToString::to_string)
}

const REGEX_CLI: &str = r"(?i)([-]{2})([0-9a-z-_]+)";

pub fn transform_cli_into_name(cli: String) -> Option<String> {
    let regex_cli = Regex::new(REGEX_CLI).expect("regex should compile");

    let matches = regex_cli.captures(&cli);
    matches.map(|cap| cap.get(2).unwrap().as_str().replace('-', "_"))
}

/**
 * Clean cli argument
 * @param String cli The command line string
 * @param bool skipRegex Skip regex check
 * @return Option<String> The cleaned cli
 */
pub fn clean_cli(mut cli: String, skip_regex: bool) -> Option<String> {
    if cli.contains("<code>") || cli.contains("</code>") {
        cli = cli.replace("<code>", "");
        cli = cli.replace("</code>", "");
        cli = cli.replace('>', "");
        cli = cli.replace('<', "");
    }

    let regex_cli = Regex::new(REGEX_CLI).expect("regex should compile");
    if !skip_regex && !regex_cli.is_match(&cli) {
        return None;
    }

    Some(cli)
}

/**
 * Clean the range object
 * @param {Object} range The range object
 * @return {Object} The cleaned range object
 */
pub const fn clean_range(range: Option<String>) -> Option<String> {
    if range.is_some() {
        // clean range
        // TODO: re-implement
        /*if (typeof range.from != "number" || isNaN(range.from)) {
            delete range.from;
        }
        if (typeof range.to == "string" && range.to.is_match(/upwards/i)) {
            range.to = "upwards";
        } else if (typeof range.to != "number" || isNaN(range.to)) {
            delete range.to;
        }*/
    }
    range
}

/**
 * Clean a default value
 * @param String defaultValue The default value
 * @return String The same or an alternative formated text
 */
pub fn clean_default(default_value: String) -> String {
    let values: Vec<String> = default_value
        .split('\n')
        .map(|el| clean_text_default(el.to_string().trim().to_string()))
        .collect();
    values.join(", ")
}

/**
 * Clean text of a default value
 * @param String default_text_value The default text value
 * @return String The same or an alternative text
 */
pub fn clean_text_default(default_text_value: String) -> String {
    // Some pages on the new docs site contain a literal "`` " token where
    // the author meant to render an empty Markdown code span — strip it so
    // we don't carry the stray backticks into the JSON.
    let default_text_value = default_text_value
        .replace("`` ", "")
        .replace(" ``", "")
        .replace("``", "");
    if default_text_value == "Autosized (see description)" {
        return "(autosized)".to_string();
    }
    if default_text_value.contains("Based on the number of processors") {
        return "(based on the number of processors)".to_string();
    }
    if default_text_value == "The MariaDB data directory" {
        return "(the MariaDB data directory)".to_string();
    }
    if default_text_value.contains("-1 (signifies (autoscaling); do not assign this literal value)") {
        return "(-1 signifies autoscaling; do not use -1)".to_string();
    }
    if default_text_value.contains("-1 (signifies (autosizing); do not assign this literal value)") {
        return "(-1 signifies autosizing; do not use -1)".to_string();
    }
    default_text_value
}

/**
 * Clean range to from values
 */
pub fn clean_range_from_to(default_text_value: String) -> String {
    if default_text_value.contains(" (log file block size)") {
        return default_text_value
            .replace(" (log file block size)", "")
            .trim()
            .to_string();
    }
    if default_text_value.contains(" (MIN_ACTIVATION_THRESHOLD)") {
        return default_text_value
            .replace(" (MIN_ACTIVATION_THRESHOLD)", "")
            .trim()
            .to_string();
    }
    if default_text_value.contains(" (MAX_ACTIVATION_THRESHOLD)") {
        return default_text_value
            .replace(" (MAX_ACTIVATION_THRESHOLD)", "")
            .trim()
            .to_string();
    }
    if default_text_value.contains('(') && default_text_value.contains(')') {
        eprintln!("dtv: {default_text_value}");
    }
    default_text_value.trim().to_string()
}

/**
 * Determine if the default value should be extracted from code
 */
pub fn is_valid_default(text_value: &str) -> bool {
    let regex_with_comment = Regex::new(r".^* \([a-z0-9A-Z -]+\)$").expect("regex should compile");
    let regex_space = Regex::new(r": [0-9]+$").expect("regex should compile");
    regex_with_comment.is_match(text_value) || regex_space.is_match(text_value)
}

/**
 * Clean text of a valid values list
 */
pub fn clean_text_valid_values(valid_values_text: String) -> String {
    if Regex::new(r"^See .* for the full list\.$")
        .expect("regex should compile")
        .is_match(&valid_values_text)
    {
        return String::new();
    }
    if Regex::new(r"^.* or .*$")
        .expect("regex should compile")
        .is_match(&valid_values_text)
    {
        return valid_values_text.replace(" or ", ",");
    }
    if valid_values_text == "See description" {
        return String::new();
    }
    valid_values_text
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_transform_cli_into_name() {
        let cli = transform_cli_into_name("--test-argument".to_string());
        assert_eq!(cli, Some("test_argument".to_string()));
    }

    #[test]
    fn test_transform_cli_into_name_invalid_1() {
        let cli = transform_cli_into_name("test-argument".to_string());
        assert_eq!(cli, None);
    }

    #[test]
    fn test_transform_cli_into_name_invalid_2() {
        let cli = transform_cli_into_name("".to_string());
        assert_eq!(cli, None);
    }

    #[test]
    fn clean_cli_html_code() {
        let cli = clean_cli("<code>--test-argument</code>".to_string(), false);
        assert_eq!(cli, Some("--test-argument".to_string()));
    }

    #[test]
    fn clean_cli_html_code_not_closed() {
        let cli = clean_cli("<code>--test-argument".to_string(), false);
        assert_eq!(cli, Some("--test-argument".to_string()));
    }

    #[test]
    fn clean_cli_nothing_to_clean() {
        let cli = clean_cli("--test-argument".to_string(), false);
        assert_eq!(cli, Some("--test-argument".to_string()));
    }

    #[test]
    fn clean_cli_undefined() {
        let cli = clean_cli("".to_string(), false);
        assert_eq!(cli, None);
    }
    /*

    #[test]
    fn clean_range_undefined() {
        const range = cleaner.cleanRange(undefined);
        expect(range).to.deep.equal(undefined);
    }

    #[test]
    fn clean_range_from_typeof object (dataset-1)() {
        const range = cleaner.cleanRange({
            from: null,
            to: null,
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.from typeof object (dataset-2)() {
        const range = cleaner.cleanRange({
            to: null,
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.from typeof object (dataset-3)() {
        const range = cleaner.cleanRange({
            from: null,
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.from typeof object (dataset-4)() {
        const range = cleaner.cleanRange({
            from: undefined,
            to: undefined,
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.from typeof object (dataset-5)() {
        const range = cleaner.cleanRange({
            to: undefined,
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.from typeof object (dataset-6)() {
        const range = cleaner.cleanRange({
            from: undefined,
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.from typeof object (dataset-7)() {
        const range = cleaner.cleanRange({
            from: NaN,
            to: NaN,
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.from typeof int() {
        const range = cleaner.cleanRange({
            from: 1024,
        });
        expect(range).to.deep.equal({
            from: 1024,
        });
    }

    #[test]
    fn clean range.from typeof string() {
        const range = cleaner.cleanRange({
            from: "1024",
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.to typeof int() {
        const range = cleaner.cleanRange({
            to: 1024,
        });
        expect(range).to.deep.equal({
            to: 1024,
        });
    }

    #[test]
    fn clean range.to typeof string() {
        const range = cleaner.cleanRange({
            to: "1024",
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range.to typeof object() {
        const range = cleaner.cleanRange({
            to: {},
        });
        expect(range).to.deep.equal({});
    }

    #[test]
    fn clean range to upwards() {
        const range = cleaner.cleanRange({
            to: "upwards",
        });
        expect(range).to.deep.equal({
            to: "upwards",
        });
    }

    #[test]
    fn clean range to upwards match() {
        const range = cleaner.cleanRange({
            to: "(128KB) upwards",
        });
        expect(range).to.deep.equal({
            to: "upwards",
        });
    }*/

    #[test]
    fn clean_binary_types_in_bytes() {
        let type_str = clean_type("in bytes".to_string());
        assert_eq!(type_str, Some("byte".to_string()));
    }

    #[test]
    fn clean_binary_types_size_in_mb() {
        let type_str = clean_type("size in mb".to_string());
        assert_eq!(type_str, Some("byte".to_string()));
    }

    #[test]
    fn clean_binary_types_number_of_bytes() {
        let type_str = clean_type("number of bytes".to_string());
        assert_eq!(type_str, Some("byte".to_string()));
    }

    #[test]
    fn clean_binary_types_number_of() {
        let type_str = clean_type("number of".to_string());
        assert_eq!(type_str, Some("integer".to_string()));
    }

    #[test]
    fn clean_binary_types_size_of() {
        let type_str = clean_type("size of".to_string());
        assert_eq!(type_str, Some("integer".to_string()));
    }

    #[test]
    fn clean_binary_types_in_microseconds() {
        let type_str = clean_type("in microseconds".to_string());
        assert_eq!(type_str, Some("integer".to_string()));
    }

    #[test]
    fn clean_binary_types_in_seconds() {
        let type_str = clean_type("in seconds".to_string());
        assert_eq!(type_str, Some("integer".to_string()));
    }

    #[test]
    fn clean_wtf_type() {
        let type_str = clean_type("wtf".to_string());
        assert_eq!(type_str, None);
    }

    #[test]
    fn clean_enumeration_type() {
        let type_str = clean_type("enumeration".to_string());
        assert_eq!(type_str, Some("enumeration".to_string()));
    }

    #[test]
    fn clean_undefined_type() {
        let type_str = clean_type("undefined".to_string());
        assert_eq!(type_str, None);
    }

    #[test]
    fn clean_type_bool() {
        let type_str = clean_type("bool".to_string());
        assert_eq!(type_str, Some("boolean".to_string()));
    }

    #[test]
    fn clean_type_varchar() {
        let type_str = clean_type("varchar".to_string());
        assert_eq!(type_str, Some("string".to_string()));
    }

    #[test]
    fn clean_type_text() {
        let type_str = clean_type("text".to_string());
        assert_eq!(type_str, Some("string".to_string()));
    }

    #[test]
    fn clean_type_filename() {
        let type_str = clean_type("filename".to_string());
        assert_eq!(type_str, Some("file name".to_string()));
        let type_str = clean_type("wsrep status output filename".to_string());
        assert_eq!(type_str, Some("file name".to_string()));
    }

    #[test]
    fn clean_type_directory_name_s() {
        let type_str = clean_type("directory name/s".to_string());
        assert_eq!(type_str, Some("directory name".to_string()));
    }

    #[test]
    fn clean_type_path_name() {
        let type_str = clean_type("path name".to_string());
        assert_eq!(type_str, Some("directory name".to_string()));
    }

    #[test]
    fn clean_type_batch_size() {
        let type_str = clean_type("insert batch size.".to_string());
        assert_eq!(type_str, Some("integer".to_string()));
    }

    #[test]
    fn clean_type_datetime() {
        let type_str = clean_type("datetime".to_string());
        assert_eq!(type_str, Some("string".to_string()));
    }

    #[test]
    fn clean_type_double() {
        let type_str = clean_type("double".to_string());
        assert_eq!(type_str, Some("numeric".to_string()));
    }

    #[test]
    fn clean_type_unsigned_long() {
        // MySQL audit-log-reference exposes some vars as "Type: Unsigned Long".
        assert_eq!(clean_type("unsigned long".to_string()), Some("integer".to_string()),);
    }

    #[test]
    fn clean_type_bigint_unsigned() {
        assert_eq!(clean_type("bigint unsigned".to_string()), Some("integer".to_string()),);
    }

    #[test]
    fn clean_type_int_unsigned() {
        assert_eq!(clean_type("int unsigned".to_string()), Some("integer".to_string()),);
    }

    #[test]
    fn clean_type_bitmap() {
        // MySQL ndb_recv_thread_cpu_mask is documented as "Type: Bitmap" but
        // its value is a comma-separated CPU-mask string like "0-3,5".
        assert_eq!(clean_type("bitmap".to_string()), Some("string".to_string()),);
    }

    #[test]
    fn clean_type_ip_address() {
        let type_str = clean_type("ip address".to_string());
        assert_eq!(type_str, Some("string".to_string()));
    }

    #[test]
    fn clean_type_bytes_from_to() {
        let type_str = clean_type("bytes read from block cache.".to_string());
        assert_eq!(type_str, Some("byte".to_string()));
        let type_str = clean_type("bytes written to block cache.".to_string());
        assert_eq!(type_str, Some("byte".to_string()));
    }

    #[test]
    fn clean_datetime_type() {
        let type_str = clean_type("datetime".to_string());
        assert_eq!(type_str, Some("string".to_string()));
    }

    #[test]
    fn clean_removed_type() {
        let type_str = clean_type("removed.".to_string());
        assert_eq!(type_str, None);
    }

    #[test]
    fn clean_unused_type() {
        let type_str = clean_type("unused.".to_string());
        assert_eq!(type_str, None);
        let type_str = clean_type("unused since 10.1.4".to_string());
        assert_eq!(type_str, None);
    }

    #[test]
    fn clean_enumerated_type() {
        let type_str = clean_type("enumerated".to_string());
        assert_eq!(type_str, Some("enumeration".to_string()));
    }

    #[test]
    fn get_clean_type_from_a_mixed_string_dataset_1() {
        let found_type = get_clean_type_from_mixed_string("boolean: ON (Version: 5.7)".to_string());
        assert_eq!(found_type, Some("boolean".to_string()));
    }

    #[test]
    fn get_clean_type_from_a_mixed_string_dataset_2() {
        let found_type = get_clean_type_from_mixed_string("numeric: 15".to_string());
        assert_eq!(found_type, Some("numeric".to_string()));
    }

    #[test]
    fn get_clean_text_vie_valid_values_non_valid_value_dataset_1() {
        let cleaned_value = clean_text_valid_values("See description".to_string());
        assert_eq!(cleaned_value, "");
    }

    #[test]
    fn get_clean_text_vie_valid_values_non_valid_value_dataset_2() {
        let cleaned_value = clean_text_valid_values("See alter_algorithm for the full list.".to_string());
        assert_eq!(cleaned_value, "");
    }

    #[test]
    fn get_clean_text_vie_valid_values_non_valid_value_dataset_3() {
        let cleaned_value = clean_text_valid_values("See OLD Mode for the full list.".to_string());
        assert_eq!(cleaned_value, "");
    }

    #[test]
    fn get_clean_text_vie_valid_values_non_valid_value_dataset_4() {
        let cleaned_value = clean_text_valid_values("0 or 1".to_string());
        assert_eq!(cleaned_value, "0,1");
    }

    #[test]
    fn get_clean_text_vie_valid_values_non_valid_value_dataset_5() {
        let cleaned_value = clean_text_valid_values("\"\" or \"uncompressed\"".to_string());
        assert_eq!(cleaned_value, "\"\",\"uncompressed\"");
    }

    #[test]
    fn clean_range_from_to_dataset_1() {
        let cleaned_value = clean_range_from_to("512 (log file block size)".to_string());
        assert_eq!(cleaned_value, "512");
    }

    #[test]
    fn clean_range_from_to_trim_dataset_2() {
        let cleaned_value = clean_range_from_to(" 512 (log file block size)".to_string());
        assert_eq!(cleaned_value, "512");
    }

    #[test]
    fn clean_range_from_to_trim_dataset_3() {
        let cleaned_value = clean_range_from_to("0 (MIN_ACTIVATION_THRESHOLD)".to_string());
        assert_eq!(cleaned_value, "0");
    }

    #[test]
    fn clean_range_from_to_trim_dataset_4() {
        let cleaned_value = clean_range_from_to("16 (MAX_ACTIVATION_THRESHOLD)".to_string());
        assert_eq!(cleaned_value, "16");
    }

    #[test]
    fn is_valid_default_dataset_1() {
        let is_valid = is_valid_default("512 (log file block size)");
        assert_eq!(is_valid, true);
    }

    #[test]
    fn is_valid_default_dataset_2() {
        let is_valid = is_valid_default(": 100");
        assert_eq!(is_valid, true);
    }

    #[test]
    fn is_valid_default_dataset_3() {
        let is_valid = is_valid_default("Depends on the system. Often /usr/share/cracklib/pw_dict");
        assert_eq!(is_valid, false);
    }

    #[test]
    fn is_valid_default_dataset_4() {
        let is_valid = is_valid_default("Empty, previously 0.0.0.0");
        assert_eq!(is_valid, false);
    }

    #[test]
    fn is_valid_default_dataset_5() {
        let is_valid = is_valid_default("NULL (>= MariaDB 10.2.2), . (<= MariaDB 10.2.1)");
        assert_eq!(is_valid, false);
    }

    #[test]
    fn is_valid_default_dataset_6() {
        let is_valid = is_valid_default("134217728 (128M)");
        assert_eq!(is_valid, true);
    }

    #[test]
    fn is_valid_default_dataset_7() {
        let is_valid = is_valid_default("The lower of 900 and (50 + max_connections/5)");
        assert_eq!(is_valid, false);
    }

    #[test]
    fn is_valid_default_dataset_8() {
        let is_valid = is_valid_default("0 (non-segmented)");
        assert_eq!(is_valid, true);
    }

    /// Walk every committed `data/variables/*.json` and flag entries that
    /// carry a `cli` or `default` (so they're not just a name-only stub)
    /// but no `type`. That's the shape of an entry whose `Data Type:` line
    /// went through `clean_type` and got `None` back — i.e. a new unknown
    /// type label is in play and needs to be added here.
    #[test]
    fn data_files_have_typed_variables() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("variables");

        let mut missing: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(&dir).expect("data/variables exists") {
            let path = entry.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) != Some("json") {
                continue;
            }
            let raw = std::fs::read_to_string(&path).unwrap();
            let v: serde_json::Value =
                serde_json::from_str(&raw).unwrap_or_else(|e| panic!("invalid JSON in {}: {}", path.display(), e));
            let Some(arr) = v.get("data").and_then(|d| d.as_array()) else {
                continue;
            };
            for item in arr {
                let has_cli_or_default = item.get("cli").is_some() || item.get("default").is_some();
                let has_type = item.get("type").is_some();
                let is_removed = item.get("isRemoved").and_then(|b| b.as_bool()).unwrap_or(false);
                if has_cli_or_default && !has_type && !is_removed {
                    let name = item.get("name").and_then(|n| n.as_str()).unwrap_or("<unnamed>");
                    missing.push(format!("{}: {}", path.file_name().unwrap().to_string_lossy(), name));
                }
            }
        }

        // Snapshot of currently-allowed gaps. These are variables whose
        // upstream docs page genuinely lacks a "Data Type:" line, so
        // clean_type has nothing to consume. Anything *not* in this list is
        // a regression — most likely a new "Type: …" label that clean_type
        // doesn't yet map.
        let allowed_known_gaps: std::collections::HashSet<&str> = [
            // MySQL command-line switches (boolean flags) — server-options
            // pages don't carry a Data Type.
            "mysql-server-options.json: ansi",
            "mysql-server-options.json: console",
            "mysql-server-options.json: core_file",
            "mysql-server-options.json: help",
            "mysql-server-options.json: install",
            "mysql-server-options.json: install_manual",
            "mysql-server-options.json: local_service",
            "mysql-server-options.json: remove",
            "mysql-server-options.json: skip_host_cache",
            "mysql-server-options.json: skip_new",
            "mysql-server-options.json: skip_stack_trace",
            "mysql-server-options.json: standalone",
            "mysql-mysql-cluster-options-variables.json: skip_ndbcluster",
            // MySQL 5.7 thread_pool variables — upstream omits Data Type.
            "mysql-server-system-variables_5.7.json: thread_pool_algorithm",
            "mysql-server-system-variables_5.7.json: thread_pool_max_unused_threads",
            "mysql-server-system-variables_5.7.json: thread_pool_prio_kickup_timer",
            // MariaDB legacy / deprecated / removed-engine pages — preserved
            // from origin/main because the upstream page is either gone
            // (PBXT) or never had a Data Type field.
            "mariadb-cassandra-system-variables.json: cassandra_read_consistency",
            "mariadb-cassandra-system-variables.json: cassandra_write_consistency",
            "mariadb-mariadb-audit-plugin-options-and-system-variables.json: server_audit_sync_log_file",
            "mariadb-pbxt-system-variables.json: pbxt_auto_increment_mode",
            "mariadb-pbxt-system-variables.json: pbxt_checkpoint_frequency",
            "mariadb-pbxt-system-variables.json: pbxt_flush_log_at_trx_commit",
            "mariadb-pbxt-system-variables.json: pbxt_garbage_threshold",
            "mariadb-pbxt-system-variables.json: pbxt_index_cache_size",
            "mariadb-pbxt-system-variables.json: pbxt_offline_log_function",
            "mariadb-pbxt-system-variables.json: pbxt_record_cache_size",
            "mariadb-pbxt-system-variables.json: pbxt_support_xa",
            "mariadb-pbxt-system-variables.json: pbxt_sweeper_priority",
            "mariadb-replication-and-binary-log-server-system-variables.json: init_rpl_role",
            "mariadb-replication-and-binary-log-server-system-variables.json: slave_parallel_workers",
            "mariadb-server-system-variables.json: default_table_type",
            "mariadb-server-system-variables.json: multi_range_count",
            "mariadb-server-system-variables.json: tmp_memory_table_size",
            "mariadb-spider-server-system-variables.json: spider_table_sts_thread_count",
            "mariadb-xtradbinnodb-server-system-variables.json: innodb_auto_lru_dump",
            "mariadb-xtradbinnodb-server-system-variables.json: innodb_lock_wait_timeout",
        ]
        .into_iter()
        .collect();

        let unexpected: Vec<&String> = missing
            .iter()
            .filter(|m| !allowed_known_gaps.contains(m.as_str()))
            .collect();

        assert!(
            unexpected.is_empty(),
            "Entries with cli/default but no type — likely a new unknown 'Type: …' label that clean_type doesn't handle:\n  {}",
            unexpected
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join("\n  ")
        );
    }
}
