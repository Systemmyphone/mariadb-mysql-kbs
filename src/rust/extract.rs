use crate::data::{DataFile, PageProcess, QueryErrorResponse, QueryResponse};
use crate::{aurora_mysql, mariadb, mysql};
use serde::Serialize;
use std::time::Duration;
use std::{env, fs};
use ureq::{Agent, Error, ResponseExt};

const UA_FROM: &str = "williamdes+mariadb-mysql-kbs@wdes.fr";
const UA: &str = "mariadb-mysql-kbs-bot (+https://github.com/williamdes/mariadb-mysql-kbs; williamdes+mariadb-mysql-kbs@wdes.fr)";
// dev.mysql.com 403s custom bot User-Agents, so present a browser there only.
const UA_BROWSER: &str = "Mozilla/5.0 (X11; Linux x86_64; rv:152.0) Gecko/20100101 Firefox/152.0";

pub enum ExtractionType {
    MariaDB,
    MySQL,
    AuroraMySQL,
}

pub enum ExtractionPreference {
    All,
    MariaDB,
    MySQL,
    AuroraMySQL,
}

pub fn extract(only: ExtractionPreference) {
    println!("Run build...");
    let pages: Vec<PageProcess> = match only {
        ExtractionPreference::All => [
            mysql::get_pages(),
            aurora_mysql::get_pages(),
            mariadb::get_pages(),
        ]
        .concat(),
        ExtractionPreference::MySQL => mysql::get_pages(),
        ExtractionPreference::AuroraMySQL => aurora_mysql::get_pages(),
        ExtractionPreference::MariaDB => mariadb::get_pages(),
    };

    for page in pages {
        extract_page(page);
    }
    println!("All done.");
    println!("End !");
}

pub fn get_html_from_url(agent: Agent, url: &str) -> Result<QueryResponse, QueryErrorResponse> {
    let request = agent.get(url);
    // dev.mysql.com blocks the bot User-Agent, so send browser headers there;
    // every other source keeps the original self-identifying bot request.
    let request = if url.contains("dev.mysql.com") {
        request
            .header("User-Agent", UA_BROWSER)
            .header(
                "Accept",
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            )
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("DNT", "1")
            .header("Sec-GPC", "1")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Priority", "u=0, i")
    } else {
        request.header("From", UA_FROM).header("User-Agent", UA)
    };
    match request.call() {
        Ok(mut response) => Ok(QueryResponse {
            url: response.get_uri().to_string(),
            body: response
                .body_mut()
                .read_to_string()
                .expect("Should have text"),
        }),
        Err(Error::StatusCode(code)) => Err(QueryErrorResponse {
            url: Some(url.to_string()),
            code: Some(code),
            message: "".to_string(),
        }),
        Err(err) => Err(QueryErrorResponse {
            url: None,
            code: None,
            message: err.to_string(),
        }),
    }
}

fn extract_page(page: PageProcess) {
    let agent: Agent = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(30)))
        .build()
        .into();
    match get_html_from_url(agent, &page.url) {
        Ok(response) => {
            let final_url = response.url.clone();
            println!("URL : {} -> {}", &page.url, final_url);
            let data = DataFile {
                data: match page.get_data_type() {
                    ExtractionType::MariaDB => mariadb::extract_mariadb_from_text(response),
                    ExtractionType::MySQL => mysql::extract_mysql_from_text(response),
                    ExtractionType::AuroraMySQL => {
                        aurora_mysql::extract_aurora_mysql_from_text(response)
                    }
                },
                url: final_url.as_str(),
                name: &page.name,
            };
            write_page(page.data_type, page.get_data_prefix(), data);
        }
        Err(err) => {
            eprintln!(
                "URL : {} ended in {} ({})",
                &page.url,
                err.code.unwrap_or(0),
                err.url.unwrap_or("".to_string())
            );
        }
    }
}

fn write_json(filename: String, data: DataFile) {
    let buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"  ");
    let mut ser = serde_json::Serializer::with_formatter(buf, formatter);
    data.serialize(&mut ser).expect("Unable to serialize data");
    let mut data = ser.into_inner();
    data.push(0x0a); // LF
    fs::write(filename, data).expect("Unable to write file");
}

fn write_page(data_type: &str, file_prefix: &str, data: DataFile) {
    let current_dir = env::current_dir().unwrap();
    write_json(
        format!(
            "{}/data/{}/{}{}.json",
            current_dir.to_str().unwrap().to_owned(),
            data_type,
            file_prefix,
            data.name
        ),
        data,
    );
}
