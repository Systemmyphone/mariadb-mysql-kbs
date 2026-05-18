use select::{
    document::Document,
    node::Node,
    predicate::{Class, Name, Predicate},
};

use crate::{
    cleaner,
    data::{KbParsedEntry, Page, PageProcess, QueryResponse},
};

// MariaDB documentation moved to a GitBook-backed site under
// https://mariadb.com/docs/server/. The old /kb/en/ URLs only redirect
// reliably for a subset of pages — others land on a search results page.
// We therefore list each page with its concrete new docs URL.
//
// The `name` field is preserved as it was historically so the JSON file
// names on disk stay stable (e.g. mariadb-aria-server-status-variables.json).
//
// Pages that have no equivalent on the new docs site at all are intentionally
// omitted (e.g. pbxt-system-variables, galera-cluster-{system,status}-variables,
// mariadb-audit-plugin-system-variables). Their JSON files are preserved
// untouched from the previous extraction.
const DOCS_BASE: &str = "https://mariadb.com/docs/server/";

const PAGES: &[Page] = &[
    // --- storage engine system variables ---
    Page {
        url: "server-usage/storage-engines/aria/aria-system-variables",
        name: "aria-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/connect/connect-system-variables",
        name: "connect-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/myisam-storage-engine/myisam-system-variables",
        name: "myisam-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/myrocks/myrocks-system-variables",
        name: "myrocks-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/mroonga/mroonga-system-variables",
        name: "mroonga-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/s3-storage-engine/s3-storage-engine-system-variables",
        name: "s3-storage-engine-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/spider/spider-system-variables",
        name: "spider-server-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/legacy-storage-engines/cassandra/cassandra-system-variables",
        name: "cassandra-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/legacy-storage-engines/tokudb/tokudb-system-variables",
        name: "tokudb-system-variables",
    },
    Page {
        url: "server-usage/storage-engines/innodb/innodb-system-variables",
        name: "xtradbinnodb-server-system-variables",
    },

    // --- storage engine status variables ---
    Page {
        url: "server-usage/storage-engines/aria/aria-status-variables",
        name: "aria-server-status-variables",
    },
    Page {
        url: "server-usage/storage-engines/myrocks/myrocks-status-variables",
        name: "myrocks-status-variables",
    },
    Page {
        url: "server-usage/storage-engines/mroonga/mroonga-status-variables",
        name: "mroonga-status-variables",
    },
    Page {
        url: "server-usage/storage-engines/legacy-storage-engines/cassandra/cassandra-status-variables",
        name: "cassandra-status-variables",
    },
    Page {
        url: "server-usage/storage-engines/legacy-storage-engines/tokudb/tokudb-status-variables",
        name: "tokudb-status-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/system-variables/innodb-status-variables",
        name: "xtradbinnodb-server-status-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/system-variables/spider-status-variables",
        name: "spider-server-status-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/system-variables/sphinx-status-variables",
        name: "sphinx-status-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/system-variables/ssltls-status-variables",
        name: "ssltls-status-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/system-variables/oqgraph-system-and-status-variables",
        name: "oqgraph-system-and-status-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/system-variables/semisynchronous-replication-plugin-status-variables",
        name: "semisynchronous-replication-plugin-status-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/buffers-caches-and-threads/thread-pool/thread-pool-system-status-variables",
        name: "thread-pool-system-and-status-variables",
    },
    Page {
        url: "ha-and-performance/standard-replication/replication-and-binary-log-status-variables",
        name: "replication-and-binary-log-status-variables",
    },
    Page {
        url: "reference/system-tables/performance-schema/performance-schema-status-variables",
        name: "performance-schema-status-variables",
    },
    Page {
        url: "reference/plugins/mariadb-audit-plugin/mariadb-audit-plugin-status-variables",
        name: "mariadb-audit-plugin-status-variables",
    },
    Page {
        url: "server-management/variables-and-modes/server-status-variables",
        name: "server-status-variables",
    },

    // --- server / cross-cutting system variables ---
    Page {
        url: "server-management/variables-and-modes/server-system-variables",
        name: "server-system-variables",
    },
    Page {
        url: "security/encryption/data-in-transit-encryption/ssltls-system-variables",
        name: "ssltls-system-variables",
    },
    Page {
        url: "reference/system-tables/performance-schema/performance-schema-system-variables",
        name: "performance-schema-system-variables",
    },
    Page {
        url: "reference/plugins/mariadb-audit-plugin/mariadb-audit-plugin-options-and-system-variables",
        name: "mariadb-audit-plugin-options-and-system-variables",
    },
    Page {
        url: "ha-and-performance/standard-replication/replication-and-binary-log-system-variables",
        name: "replication-and-binary-log-server-system-variables",
    },
    Page {
        url: "ha-and-performance/standard-replication/semisynchronous-replication",
        name: "semisynchronous-replication-system-variables",
    },
    Page {
        url: "ha-and-performance/standard-replication/gtid",
        name: "gtid-system-variables",
    },
    Page {
        url: "reference/sql-structure/temporal-tables/system-versioned-tables",
        name: "versioned-tables-system-variables",
    },
    Page {
        url: "reference/sql-structure/nosql/handlersocket/handlersocket-configuration-options",
        name: "handlersocket-configuration-options-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/optimization-and-tuning-compression/storage-engine-independent-column-compression",
        name: "storage-engine-independent-column-compression-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/query-optimizations/statistics-for-optimizing-queries/user-statistics",
        name: "user-statistics-variables",
    },
    Page {
        url: "ha-and-performance/optimization-and-tuning/system-variables/sql-error-log-system-variables-and-options",
        name: "sql-error-log-plugin-variables",
    },

    // --- galera (hosted under a separate docs subsite) ---
    Page {
        url: "https://mariadb.com/docs/galera-cluster/reference/galera-cluster-system-variables",
        name: "galera-cluster-system-variables",
    },
    Page {
        url: "https://mariadb.com/docs/galera-cluster/reference/galera-cluster-status-variables",
        name: "galera-cluster-status-variables",
    },

    // --- plugins ---
    Page {
        url: "reference/plugins/authentication-plugins/authentication-plugin-gssapi",
        name: "authentication-plugin-gssapi-variables",
    },
    Page {
        url: "reference/plugins/authentication-plugins/authentication-with-pluggable-authentication-modules-pam/authentication-plugin-pam",
        name: "authentication-plugin-pam-variables",
    },
    Page {
        url: "security/encryption/data-at-rest-encryption/key-management-and-encryption-plugins/aws-key-management-encryption-plugin",
        name: "aws-key-management-encryption-plugin-variables",
    },
    Page {
        url: "security/encryption/data-at-rest-encryption/key-management-and-encryption-plugins/file-key-management-encryption-plugin",
        name: "file-key-management-encryption-plugin-variables",
    },
    Page {
        url: "reference/plugins/password-validation-plugins/cracklib-password-check-plugin",
        name: "cracklib-password-check-plugin-variables",
    },
    Page {
        url: "reference/plugins/password-validation-plugins/simple-password-check-plugin",
        name: "simple-password-check-plugin-variables",
    },
    Page {
        url: "reference/plugins/other-plugins/disks-plugin",
        name: "disks-plugin-variables",
    },
    Page {
        url: "reference/plugins/other-plugins/feedback-plugin",
        name: "feedback-plugin-variables",
    },
    Page {
        url: "reference/plugins/other-plugins/query-cache-information-plugin",
        name: "query-cache-information-plugin-variables",
    },
    Page {
        url: "reference/plugins/other-plugins/query-response-time-plugin",
        name: "query-response-time-plugin-variables",
    },
];

pub fn get_pages() -> Vec<PageProcess<'static>> {
    PAGES
        .iter()
        .map(|p| PageProcess {
            url: if p.url.starts_with("http") {
                p.url.to_string()
            } else {
                DOCS_BASE.to_owned() + p.url
            },
            name: p.name.to_string(),
            data_type: "variables",
        })
        .collect()
}

// Known field names in the order we try to match them. Longer/more specific
// labels come first so e.g. "Default Value" wins over "Default".
const KNOWN_FIELD_KEYS: &[&str] = &[
    "Default Value - 64 bit",
    "Default Value - 32 bit",
    "Default Table Value",
    "Default Session Value",
    "Default Value",
    "Default",
    "Range - 64-bit",
    "Range - 64 bit",
    "Range - 64bit",
    "Range - 32-bit",
    "Range - 32 bit",
    "Range - 32bit",
    "Range (Windows)",
    "Range",
    "Valid Values",
    "Valid values",
    "Valid Vales",
    "Data Type",
    "Type",
    "Command line",
    "Commandline",
    "Command-line",
    "Access type",
    "DSN parameter name",
    "Related Variables",
    "Read-Only",
    "Read Only",
    "Size limit",
    "Minimum Value",
    "Re-introduced",
    "See Also",
    "See also",
    "Description",
    "Scope",
    "Dynamic",
    "Introduced",
    "Removed",
    "Deprecated",
    "Alias",
    "Notes",
    "Note",
    "Documentation",
    "Unix",
    "Windows",
];

/// Best-effort split of an `<li>` text into (key, value) for the new docs
/// format. The site is mostly "Key: value", but a few entries lack the colon
/// (e.g. "Range <code>0</code> to <code>4294967295</code>").
fn split_key_value_no_strong(li_text: &str) -> Option<(String, String)> {
    if let Some((k, v)) = li_text.split_once(':') {
        return Some((k.trim().to_string(), v.trim().to_string()));
    }
    for key in KNOWN_FIELD_KEYS {
        if li_text.starts_with(key) {
            let rest = &li_text[key.len()..];
            // The key must be followed by whitespace or end-of-text — otherwise
            // we'd e.g. match "Range" against "RangeFoo".
            if rest.is_empty() || rest.starts_with(|c: char| c.is_whitespace()) {
                return Some((key.to_string(), rest.trim().to_string()));
            }
        }
    }
    None
}

fn process_li(mut entry: KbParsedEntry, li_node: Node) -> KbParsedEntry {
    let li_text: String = li_node.text();
    let (mut key_name, row_value): (String, String) =
        match li_node.find(Name("strong")).next() {
            // Legacy KB format: <strong>Key:</strong> value
            Some(strong) => {
                let k = strong.text();
                let v = li_text
                    .split_once(k.as_str())
                    .map(|x| x.1.trim().to_string())
                    .unwrap_or_default();
                (k, v)
            }
            // New docs (GitBook) format: <p>Key: value</p>
            None => match split_key_value_no_strong(li_text.trim()) {
                Some(kv) => kv,
                None => return entry,
            },
        };

    key_name = key_name.to_lowercase().replace(":", "").trim().to_string();

    match key_name.as_str() {
        "dynamic" | "access type" => {
            entry.dynamic = Some(
                row_value.to_lowercase() == "yes"
                    || row_value.to_lowercase() == "can be changed dynamically",
            );
        }
        "data type" | "type" => {
            if li_node.find(Name("code")).count() == 1 {
                entry.r#type = Some(
                    li_node
                        .find(Name("code"))
                        .next()
                        .unwrap()
                        .text()
                        .to_lowercase()
                        .trim()
                        .to_string(),
                );
            } else {
                entry.r#type = Some(row_value.to_lowercase().trim().to_string());
            }

            if entry.r#type == Some("number".to_string()) {
                entry.r#type = Some("integer".to_string());
            }

            if entry.r#type != Some("".to_string()) {
                entry.r#type = cleaner::clean_type(entry.r#type.unwrap());
            }
            if entry.r#type == Some("".to_string()) {
                entry.r#type = None;
            }
            if entry.r#type == Some("numeric".to_string()) {
                entry.r#type = Some("integer".to_string());
            }
        }
        "default value" | "default" | "default value - 64 bit" => {
            if li_node.find(Name("code")).count() == 1
                && cleaner::is_valid_default(row_value.as_ref())
            {
                entry.default = Some(cleaner::clean_default(
                    li_node
                        .find(Name("code"))
                        .next()
                        .unwrap()
                        .text()
                        .trim()
                        .to_string(),
                ));
            } else {
                entry.default = Some(cleaner::clean_default(row_value));
            }
        }
        "commandline" | "command-line" | "command line" => {
            if li_node.find(Name("code")).count() >= 1 {
                entry.cli = Some(
                    li_node
                        .find(Name("code"))
                        .map(|code_node| code_node.text().trim().to_string())
                        .map(|code| cleaner::clean_cli(code, true))
                        .filter(|code| code.is_some())
                        .map(|code| code.unwrap())
                        .filter(|row_value| {
                            row_value.to_lowercase() != "no"
                                && row_value.to_lowercase() != "none"
                                && row_value.to_lowercase() != "n/a"
                                && row_value.to_lowercase() != "no commandline option"
                        })
                        .collect::<Vec<String>>()
                        .join(", "),
                );
                if entry.cli == Some("".to_string()) {
                    entry.cli = None;
                }
            } else {
                if row_value.to_lowercase() != "no"
                    && row_value.to_lowercase() != "none"
                    && row_value.to_lowercase() != "n/a"
                    && row_value.to_lowercase() != "no commandline option"
                {
                    entry.cli = cleaner::clean_cli(row_value, true);
                }
            }
        }

        "scope" => {
            let scope = row_value.to_lowercase().trim().to_string();
            if scope != "" {
                let values: Vec<String> = scope
                    .split(",")
                    .map(|item| item.to_lowercase())
                    .filter(|item| item.contains("session") || item.contains("global"))
                    .map(|item| {
                        if item.contains("session") {
                            return "session".to_string();
                        } else if item.contains("global") {
                            return "global".to_string();
                        }

                        return "".to_string();
                    })
                    .collect();
                entry.scope = Some(values);
            }
            if entry.scope.is_some() {
                // TODO: cleanup scope
                //entry.scope = entry.scope.filter(|e| e == "0" || e.is_some());
            }
        }
        "valid values" | "valid vales" => {
            // Handle typo on log_slow_disabled_statements
            // The new docs site sometimes nests subsequent fields
            // (Default Value, Range, …) inside "Valid Values:" by mistake.
            // Detect that case so we don't claim those nested <code> values
            // as the actual valid values — process_ul will recurse separately.
            let misnested_fields = li_node.find(Name("ul")).any(|ul| {
                ul.find(Name("li")).any(|nested_li| {
                    let t = nested_li.text();
                    let t = t.trim();
                    KNOWN_FIELD_KEYS
                        .iter()
                        .any(|k| t.starts_with(k) && t[k.len()..].starts_with(|c: char| c == ':'))
                })
            });
            if li_node.find(Name("code")).next().is_some() && !misnested_fields {
                let mut values = vec![];
                for code_node in li_node.find(Name("code")) {
                    values.push(code_node.text());
                }
                // The new docs often packs the whole list into a single
                // <code> ("A, B, C") — flatten that into N entries to match
                // the historical extraction shape.
                if values.len() == 1 && values[0].contains(',') {
                    values = values[0]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
                entry.valid_values = Some(values);
            } else if !misnested_fields {
                let clean_value = cleaner::clean_text_valid_values(row_value.trim().to_string());
                if clean_value != "" {
                    entry.valid_values = Some(
                        clean_value
                            .split(',')
                            .map(|el| el.trim().to_string())
                            .collect(),
                    );
                }
            }
        }
        "minimum value" => {
            entry.init_range();
            match entry.range {
                Some(ref mut r) => {
                    r.try_fill_from(row_value);
                }
                None => {}
            }
        }
        "range" | "range - 64 bit" | "range - 64-bit" | "range - 64bit" | "range (windows)" => {
            if li_node.find(Name("code")).next().is_some() {
                let mut values = vec![];
                for code_node in li_node.find(Name("code")).filter(|e| e.text().trim() != "") {
                    values.push(code_node.text());
                }
                if values.len() == 1 {
                    let first_value = values.first().expect("Should have a first value");
                    if first_value.contains('-') {
                        // try x-y
                        entry.init_range();
                        match entry.range {
                            Some(ref mut r) => {
                                let range = first_value.split_once('-').unwrap();

                                r.try_fill_from(range.0.to_string());
                                r.try_fill_to(range.1.to_string());
                            }
                            None => {}
                        }
                    }
                    if first_value.contains("to") {
                        // try x to y
                        entry.init_range();
                        match entry.range {
                            Some(ref mut r) => {
                                let range = first_value.split_once("to").unwrap();

                                r.try_fill_from(range.0.to_string());
                                r.try_fill_to(range.1.to_string());
                            }
                            None => {}
                        }
                    }
                    if li_node.text().contains("upwards") {
                        // try x upwards
                        entry.init_range();
                        match entry.range {
                            Some(ref mut r) => {
                                r.try_fill_from(first_value.to_string());
                                r.to_upwards = Some("upwards".to_string());
                            }
                            None => {}
                        }
                    }
                } else if values.len() == 2 {
                    entry.init_range();
                    match entry.range {
                        Some(ref mut r) => {
                            r.try_fill_from(values.first().unwrap().to_string());
                            r.try_fill_to(values.last().unwrap().to_string());
                        }
                        None => {}
                    }
                } else if values.len() == 4 {
                    // from <code>0</code> to <code>16</code> (version x.y.z)
                    // from <code>0</code> to <code>10</code> (version a.b.c)

                    // "from" values are equal
                    if values.first() == values.get(2) {
                        entry.init_range();
                        match entry.range {
                            Some(ref mut r) => {
                                r.try_fill_from(values.first().unwrap().to_string());
                            }
                            None => {}
                        }
                    }

                    // "to" values are equal
                    if values.last() == values.get(1) {
                        entry.init_range();
                        match entry.range {
                            Some(ref mut r) => {
                                r.try_fill_to(values.last().unwrap().to_string());
                            }
                            None => {}
                        }
                    }
                } else {
                    println!("range: {}", values.len());
                }
            }
        }
        "description" => {
            entry.has_description = true;

            if entry.r#type.is_none() {
                entry.r#type = cleaner::clean_type(row_value.to_lowercase());
            }
        }
        "removed" => {
            entry.is_removed = true;
        }
        "introduced"
        | "range - 32 bit"
        | "range - 32-bit"
        | "range - 32bit"
        | "size limit"
        | "see also"
        | "deprecated"
        | "re-introduced"
        | "default value - 32 bit"
        | "default table value"
        | "default session value"
        | "dsn parameter name"
        | "related variables"
        | "documentation"
        | "read only"
        | "read-only"
        | "alias"
        | "unix"
        | "windows"
        | "notes" => {}
        _key => {
            println!("missing: {} -> {}", key_name, row_value);
        }
    }

    entry
}

fn process_ul(mut entry: KbParsedEntry, ul_node: Node) -> KbParsedEntry {
    let ul_index = ul_node.index();
    let li_nodes = ul_node
        .find(Name("li"))
        // Only direct <li> children of this <ul> — exclude nested lists,
        // which will be recursed into below.
        .filter(|li| li.parent().map(|p| p.index() == ul_index).unwrap_or(false));

    for li in li_nodes {
        entry = process_li(entry, li);

        // The new docs site sometimes nests subsequent fields (e.g. Default
        // Value, Range) inside the <ul> that visually follows "Valid Values:".
        // Recurse so we still pick those fields up at this entry's level.
        for nested in li
            .find(Name("ul"))
            .filter(|n| n.find(Name("li")).next().is_some())
        {
            entry = process_ul(entry, nested);
        }
    }

    entry
}

fn process_block(header_node: Node) -> KbParsedEntry {
    // Prefer the <code> child for the name (new docs format wraps it in <code>,
    // and the surrounding wrapper may contribute hidden glyphs to .text()).
    let name = header_node
        .find(Name("code"))
        .next()
        .map(|c| c.text().trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| header_node.text().trim().to_string());

    let mut entry = KbParsedEntry {
        is_removed: false,
        has_description: false,
        cli: None,
        default: None,
        dynamic: None,
        id: Some(header_node.attr("id").unwrap().to_string()),
        name: Some(name),
        scope: None,
        r#type: None,
        valid_values: None,
        range: None,
    };

    let mut node_count = 30;
    let mut node_cur: Option<Node> = Some(header_node);

    loop {
        // Current node is None exit
        if node_cur.is_none() {
            break;
        }
        // Move cursor to previous and bump count
        node_cur = node_cur.unwrap().next();
        node_count = node_count - 1;
        // If still is None or count too low exit
        if node_cur.is_none() || node_count < 1 {
            break;
        }

        let n = node_cur.unwrap();

        // We hit the next header (legacy KB or new docs format)
        if n.is(Class("anchored_heading")) || n.is(Class("heading")) {
            break;
        }

        if n.is(Name("ul")) && n.find(Name("li")).next().is_some() {
            entry = process_ul(entry, n);
        } else if n.is(Name("div")) {
            // The new docs occasionally wraps a variable's field list in a
            // tabbed widget (e.g. innodb_doublewrite has a "Current" tab and
            // a "< MariaDB 11.0.6" tab). Walk into the div and process any
            // <ul> that looks like a field list.
            for inner_ul in n.find(Name("ul")).filter(|u| {
                u.find(Name("li"))
                    .next()
                    .map(|li| !li.text().trim().is_empty())
                    .unwrap_or(false)
            }) {
                entry = process_ul(entry, inner_ul);
            }
        }
    }

    /*
    const ulElementList = $(element).nextUntil('.anchored_heading');
    if (ulElementList.find('li > strong').length === 0) {
        return { id: null };
    }*/

    entry
}

pub fn extract_mariadb_from_text(qr: QueryResponse) -> Vec<KbParsedEntry> {
    let document = Document::from(qr.body.as_str());

    document
        .find(Class("anchored_heading").or(Class("heading")))
        .filter(|elem| elem.is(Name("h3")) || elem.is(Name("h4")))
        .filter(|elem| elem.attr("id").is_some())
        // Handle an edge case for https://mariadb.com/kb/en/temporal-data-tables/
        .filter(|elem| elem.text().trim() != "SELECT" && elem.attr("id").unwrap() != "select")
        .filter(|elem| {
            elem.text().trim() != "system-variables"
                && elem.attr("id").unwrap() != "system-variables"
        })
        .map(|header_node| process_block(header_node))
        .filter(|entry| {
            entry.r#type.is_some()
                || entry.default.is_some()
                || entry.dynamic.is_some()
                || entry.has_description
                || entry.is_removed
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::data::Range;

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;
    use pretty_assertions::assert_eq;
    use std::env;
    use std::fs;

    fn get_test_data(file_name: &str) -> String {
        let test_dir = env::current_dir().unwrap();
        fs::read_to_string(test_dir.to_str().unwrap().to_owned() + "/src/rust/data/" + file_name)
            .expect("Should have been able to read the test data file")
    }

    #[test]
    fn test_case_1() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_1.html"),
            url: "https://example.com".to_string(),
        });
        assert_eq!(
            vec![
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--query-cache-size=#".to_string()),
                    default: Some("1M (>= MariaDB, 10.1.7), 0 (<= MariaDB 10.1.6), (although frequently given a default value in some setups)".to_string()),
                    dynamic: Some(true),
                    id: Some("query_cache_size".to_string()),
                    name: Some("query_cache_size".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: Some(vec!["0".to_string()]),
                    range: None,
                },
            ],
            entries
        );
    }

    #[test]
    fn test_case_2() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_2.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("query-cache-strip-comments".to_string()),
                default: Some("OFF".to_string()),
                dynamic: Some(true),
                id: Some("query_cache_strip_comments".to_string()),
                name: Some("query_cache_strip_comments".to_string()),
                scope: Some(vec!["session".to_string(), "global".to_string()]),
                r#type: Some("boolean".to_string()),
                valid_values: None,
                range: None,
            },],
            entries
        );
    }

    #[test]
    fn test_case_3() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_3.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: None,
                default: None,
                dynamic: None,
                id: Some("ssl_accept_renegotiates".to_string()),
                name: Some("Ssl_accept_renegotiates".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: None,
            },],
            entries
        );
    }

    #[test]
    fn test_case_4() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_4.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--server-audit-events=value".to_string()),
                    default: Some("Empty string".to_string()),
                    dynamic: Some(true),
                    id: Some("server_audit_events".to_string()),
                    name: Some("server_audit_events".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("string".to_string()),
                    valid_values: Some(vec![
                        "CONNECT".to_string(),
                        "QUERY".to_string(),
                        "TABLE".to_string(),
                        "CONNECT".to_string(),
                        "QUERY".to_string(),
                        "TABLE".to_string(),
                        "QUERY_DDL".to_string(),
                        "QUERY_DML".to_string(),
                        "CONNECT".to_string(),
                        "QUERY".to_string(),
                        "TABLE".to_string(),
                        "QUERY_DDL".to_string(),
                        "QUERY_DML".to_string(),
                        "QUERY_DCL".to_string(),
                        "CONNECT".to_string(),
                        "QUERY".to_string(),
                        "TABLE".to_string(),
                        "QUERY_DDL".to_string(),
                        "QUERY_DML".to_string(),
                        "QUERY_DCL".to_string(),
                        "QUERY_DML_NO_SELECT".to_string(),
                    ]),
                    range: None,
                },
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--server-audit-excl-users=value".to_string()),
                    default: Some("Empty string".to_string()),
                    dynamic: Some(true),
                    id: Some("server_audit_excl_users".to_string()),
                    name: Some("server_audit_excl_users".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("string".to_string()),
                    valid_values: None,
                    range: None,
                },
            ],
            entries
        );
    }

    #[test]
    fn test_case_5() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_5.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    dynamic: Some(false),
                    id: Some("tokudb_version".to_string()),
                    name: Some("tokudb_version".to_string()),
                    r#type: Some("string".to_string()),
                    cli: None,
                    default: None,
                    range: None,
                    scope: None,
                    valid_values: None,
                },
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    default: Some("1000".to_string()),
                    dynamic: Some(true),
                    id: Some("tokudb_write_status_frequency".to_string()),
                    name: Some("tokudb_write_status_frequency".to_string()),
                    range: Some(Range {
                        to_upwards: None,
                        from: Some(0),
                        to: Some(4294967295),
                        from_f: None,
                        to_f: None,
                    }),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    cli: None,
                    valid_values: None,
                },
            ],
            entries
        );
    }

    #[test]
    fn test_case_6() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_6.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--rpl-semi-sync-slave-trace_level[=#]".to_string()),
                    default: Some("32".to_string()),
                    dynamic: Some(true),
                    id: Some("rpl_semi_sync_slave_trace_level".to_string()),
                    name: Some("rpl_semi_sync_slave_trace_level".to_string()),
                    range: Some(Range {
                        to_upwards: None,
                        from: Some(0),
                        to: Some(18446744073709551615),
                        from_f: None,
                        to_f: None,
                    }),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: None,
                },
                KbParsedEntry {
                    has_description: true,
                    is_removed: true,
                    cli: Some("--rpl-semi-sync-master=value".to_string()),
                    default: Some("ON".to_string()),
                    id: Some("rpl_semi_sync_master".to_string()),
                    name: Some("rpl_semi_sync_master".to_string()),
                    r#type: Some("enumeration".to_string()),
                    valid_values: Some(vec![
                        "OFF".to_string(),
                        "ON".to_string(),
                        "FORCE".to_string(),
                        "FORCE_PLUS_PERMANENT".to_string()
                    ]),
                    range: None,
                    scope: None,
                    dynamic: None,
                },
            ],
            entries
        );
    }

    #[test]
    fn test_case_7() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_7.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                dynamic: None,
                cli: Some("--wsrep-provider=value".to_string()),
                default: Some("None".to_string()),
                id: Some("wsrep_provider".to_string()),
                name: Some("wsrep_provider".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("string".to_string()),
                valid_values: None,
                range: None,
            },],
            entries
        );
    }

    #[test]
    fn test_case_8() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_8.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--tls-version=value".to_string()),
                default: Some("TLSv1.1,TLSv1.2,TLSv1.3".to_string()),
                dynamic: Some(false),
                id: Some("tls_version".to_string()),
                name: Some("tls_version".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("enumeration".to_string()),
                valid_values: Some(vec![
                    "TLSv1.0".to_string(),
                    "TLSv1.1".to_string(),
                    "TLSv1.2".to_string(),
                    "TLSv1.3".to_string()
                ]),
                range: None,
            },],
            entries
        );
    }

    #[test]
    fn test_case_9() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_9.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--connect-work-size=#".to_string()),
                default: Some("67108864".to_string()),
                dynamic: Some(true),
                id: Some("connect_work_size".to_string()),
                name: Some("connect_work_size".to_string()),
                scope: Some(vec!["global".to_string(), "session".to_string()]),
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: Some(Range {
                    to_upwards: Some("upwards".to_string()),
                    from: Some(4194304),
                    from_f: None,
                    to: None,
                    to_f: None,
                }),
            },],
            entries
        );
    }

    #[test]
    fn test_case_10() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_10.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--wsrep-sync-wait=".to_string()),
                default: Some("0".to_string()),
                dynamic: Some(true),
                id: Some("wsrep_sync_wait".to_string()),
                name: Some("wsrep_sync_wait".to_string()),
                scope: Some(vec!["global".to_string(), "session".to_string()]),
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: Some(Range {
                    to_upwards: None,
                    from: Some(0),
                    from_f: None,
                    to: None,
                    to_f: None,
                }),
            },],
            entries
        );
    }

    #[test]
    fn test_case_11() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_11.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--lock-wait-timeout=#".to_string()),
                default: Some(
                    "86400 (1 day) >= MariaDB 10.2.4, , 31536000 (1 year) <= MariaDB 10.2.3"
                        .to_string()
                ),
                dynamic: Some(true),
                id: Some("lock_wait_timeout".to_string()),
                name: Some("lock_wait_timeout".to_string()),
                scope: Some(vec!["global".to_string(), "session".to_string()]),
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: Some(Range {
                    to_upwards: None,
                    from: None,
                    from_f: None,
                    to: Some(31536000),
                    to_f: None,
                }),
            },],
            entries
        );
    }

    #[test]
    fn test_case_12() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_12.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: None,
                default: None,
                dynamic: None,
                id: Some("wsrep_cert_index_size".to_string()),
                name: Some("wsrep_cert_index_size".to_string()),
                scope: None,
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: None,
            },],
            entries
        );
    }

    #[test]
    fn test_case_13() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_13.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--system-versioning-insert-history[={0|1}]".to_string()),
                default: Some("OFF".to_string()),
                dynamic: Some(true),
                id: Some("system_versioning_insert_history".to_string()),
                name: Some("system_versioning_insert_history".to_string()),
                scope: Some(vec!["global".to_string(), "session".to_string()]),
                r#type: Some("boolean".to_string()),
                valid_values: None,
                range: None,
            },],
            entries
        );
    }

    #[test]
    fn test_case_14() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_14.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: false,
                is_removed: false,
                cli: Some("--gtid-pos-auto-engines=value".to_string()),
                default: Some("empty".to_string()),
                dynamic: Some(true),
                id: Some("gtid_pos_auto_engines".to_string()),
                name: Some("gtid_pos_auto_engines".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("string".to_string()),
                valid_values: None,
                range: None,
            },],
            entries
        );
    }

    #[test]
    fn test_case_15() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_15.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--handlersocket-wrlock-timeout=\"value\"".to_string()),
                default: None,
                dynamic: Some(false),
                id: Some("handlersocket_wrlock_timeout".to_string()),
                name: Some("handlersocket_wrlock_timeout".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: Some(Range {
                    to_upwards: None,
                    from: Some(0),
                    to: Some(3600),
                    from_f: None,
                    to_f: None,
                }),
            },],
            entries
        );
    }

    #[test]
    fn test_case_16() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_16.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--innodb-fast-shutdown[=#]".to_string()),
                default: Some("1".to_string()),
                dynamic: Some(true),
                id: Some("innodb_fast_shutdown".to_string()),
                name: Some("innodb_fast_shutdown".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: Some(Range {
                    to_upwards: None,
                    from: Some(0),
                    to: None,
                    from_f: None,
                    to_f: None,
                }),
            },],
            entries
        );
    }

    #[test]
    fn test_case_17() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_17.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--innodb-fill-factor=#".to_string()),
                default: Some("100".to_string()),
                dynamic: Some(true),
                id: Some("innodb_fill_factor".to_string()),
                name: Some("innodb_fill_factor".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: Some(Range {
                    to_upwards: None,
                    from: Some(10),
                    to: Some(100),
                    from_f: None,
                    to_f: None,
                }),
            },],
            entries
        );
    }

    #[test]
    fn test_case_18() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_18.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![
                KbParsedEntry {
                    has_description: true,
                    is_removed: true,
                    cli: Some("innodb-buffer-pool-restore-at-startup".to_string()),
                    default: Some("0".to_string()),
                    dynamic: Some(true),
                    id: Some("innodb_buffer_pool_restore_at_startup".to_string()),
                    name: Some("innodb_buffer_pool_restore_at_startup".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: None,
                    range: Some(Range {
                        to_upwards: None,
                        from: Some(0),
                        to: Some(18446744073709547520),
                        from_f: None,
                        to_f: None,
                    }),
                },
                KbParsedEntry {
                    cli: Some("--myisam-mmap-size=#".to_string()),
                    default: Some("18446744073709547520".to_string()),
                    dynamic: Some(true),
                    id: Some("myisam_mmap_size".to_string()),
                    name: Some("myisam_mmap_size".to_string()),
                    range: Some(Range {
                        from: Some(7,),
                        from_f: None,
                        to: Some(18446744073709547520),
                        to_f: None,
                        to_upwards: None,
                    },),
                    scope: Some(vec!["global".to_string(), "session".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: None,
                    has_description: true,
                    is_removed: false,
                },
            ],
            entries
        );
    }

    #[test]
    fn test_case_19() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_19.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--spider-max-connections".to_string()),
                    default: None,
                    dynamic: Some(true),
                    id: Some("spider_max_connections".to_string()),
                    name: Some("spider_max_connections".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: None,
                    range: Some(Range {
                        to_upwards: None,
                        from: Some(0),
                        to: Some(99999),
                        from_f: None,
                        to_f: None,
                    }),
                },
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--master-verify-checksum=[0|1]".to_string()),
                    default: Some("OFF (0)".to_string()),
                    dynamic: Some(true),
                    id: Some("master_verify_checksum".to_string()),
                    name: Some("master_verify_checksum".to_string()),
                    range: None,
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("boolean".to_string()),
                    valid_values: None,
                },
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--ft-min-word-len=#".to_string()),
                    default: Some("4".to_string()),
                    dynamic: Some(false),
                    id: Some("ft_min_word_len".to_string()),
                    name: Some("ft_min_word_len".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: None,
                    range: Some(Range {
                        to_upwards: None,
                        from: Some(1),
                        to: None,
                        from_f: None,
                        to_f: None,
                    }),
                },
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--handlersocket-epoll=\"value\"".to_string()),
                    default: Some("1".to_string()),
                    dynamic: Some(false),
                    id: Some("handlersocket_epoll".to_string()),
                    name: Some("handlersocket_epoll".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: Some(vec!["0".to_string(), "1".to_string()]),
                    range: None,
                },
            ],
            entries
        );
    }

    #[test]
    fn test_case_20() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_20.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--wsrep-debug[={NONE|SERVER|TRANSACTION|STREAMING|CLIENT}], --wsrep-debug[={0|1}]".to_string()),
                default: Some("NONE (>= MariaDB 10.4.3),  OFF (<= MariaDB 10.4.2)".to_string()),
                dynamic: Some(true),
                id: Some("wsrep_debug".to_string()),
                name: Some("wsrep_debug".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("enumeration".to_string()),
                valid_values: Some(vec![
                    "NONE".to_string(),
                    "SERVER".to_string(),
                    "TRANSACTION".to_string(),
                    "STREAMING".to_string(),
                    "CLIENT".to_string()
                ]),
                range: None,
            },],
            entries
        );
    }

    #[test]
    fn test_case_23_nested_misplaced_fields() {
        // The new docs sometimes nests "Default Value" and "Range" under
        // "Valid Values:" as if they were sub-items. We should still extract
        // them at the entry level and not treat the inner codes as valid
        // values.
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_23.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--aria_group_commit_interval=#".to_string()),
                default: Some("0".to_string()),
                dynamic: Some(false),
                id: Some("aria_group_commit_interval".to_string()),
                name: Some("aria_group_commit_interval".to_string()),
                scope: Some(vec!["global".to_string()]),
                r#type: Some("integer".to_string()),
                valid_values: None,
                range: Some(Range {
                    to_upwards: None,
                    from: Some(0),
                    to: Some(4294967295),
                    from_f: None,
                    to_f: None,
                }),
            }],
            entries
        );
    }

    #[test]
    fn test_case_22_new_docs_format() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_22.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--aria-block-size=#".to_string()),
                    default: Some("8192".to_string()),
                    dynamic: Some(false),
                    id: Some("aria_block_size".to_string()),
                    name: Some("aria_block_size".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: None,
                    range: None,
                },
                KbParsedEntry {
                    has_description: true,
                    is_removed: false,
                    cli: Some("--aria-checkpoint-interval=#".to_string()),
                    default: Some("30".to_string()),
                    dynamic: Some(true),
                    id: Some("aria_checkpoint_interval".to_string()),
                    name: Some("aria_checkpoint_interval".to_string()),
                    scope: Some(vec!["global".to_string()]),
                    r#type: Some("integer".to_string()),
                    valid_values: None,
                    range: Some(Range {
                        to_upwards: None,
                        from: Some(0),
                        to: Some(4294967295),
                        from_f: None,
                        to_f: None,
                    }),
                },
            ],
            entries
        );
    }

    #[test]
    fn test_case_21() {
        let entries = extract_mariadb_from_text(QueryResponse {
            body: get_test_data("mariadb_test_case_21.html"),
            url: "https://example.com".to_string(),
        });

        assert_eq!(
            vec![KbParsedEntry {
                has_description: true,
                is_removed: false,
                cli: Some("--transaction-read-only=#".to_string()),
                default: Some("OFF".to_string()),
                dynamic: Some(true),
                id: Some("tx_read_only".to_string()),
                name: Some("tx_read_only".to_string()),
                scope: Some(vec!["global".to_string(), "session".to_string()]),
                r#type: Some("boolean".to_string()),
                valid_values: None,
                range: None,
            },],
            entries
        );
    }
}
