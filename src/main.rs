use futures::future::join_all;
use futures::future::Future;

use std::env;
use std::error::Error;
use std::fs::{self, DirEntry};
use std::io;
use std::path::Path;
use std::pin::Pin;

use regex::Regex;

static DOC_FILE_REGEX: &'static str = r"(.*\.md$)|(.*\.rst$)|(?i)README";

static RST_FILE_REGEX: &'static str = r"(.*\.rst$)";
static RST_LINK_REGEX: &'static str = r"`.* <(?P<link>[^>]*)>`_";

static MARKDOWN_FILE_REGEX: &'static str = r"(.*\.md$)";
static MARKDOWN_LINK_REGEX: &'static str = r"\[.*\]\((?P<link>.*)\)";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<String>>();
    if args.len() != 2 {
        eprintln!("Usage: link-checker dir-path");
        return Ok(());
    }

    let path = Path::new(&args[1]);

    visit_doc_files(path, &|file| async move {
        let links = extract_links_from_file(&file, file_type(&file));

        println!("checking {:#?} ..", file.path());
        let futures: Vec<_> = links.into_iter().map(|x| check_link_validity(x)).collect();
        join_all(futures).await;
    })
    .await?;

    Ok(())
}

fn is_doc_file(file: &DirEntry) -> bool {
    let re = Regex::new(DOC_FILE_REGEX).unwrap();
    re.is_match(file.file_name().to_str().unwrap())
}

enum FileType {
    TXT,
    MARKDOWN,
    RST,
}

fn file_type(file: &DirEntry) -> FileType {
    let re_rst = Regex::new(RST_FILE_REGEX).unwrap();
    let re_markdown = Regex::new(MARKDOWN_FILE_REGEX).unwrap();

    if re_rst.is_match(file.file_name().to_str().unwrap()) {
        FileType::RST
    } else if re_markdown.is_match(file.file_name().to_str().unwrap()) {
        FileType::MARKDOWN
    } else {
        FileType::TXT
    }
}

fn extract_links_from_file(file: &DirEntry, file_type: FileType) -> Vec<String> {
    let mut links: Vec<String> = Vec::new();

    let re = match file_type {
        FileType::RST => Regex::new(RST_LINK_REGEX).unwrap(),
        FileType::MARKDOWN => Regex::new(MARKDOWN_LINK_REGEX).unwrap(),
        FileType::TXT => Regex::new(MARKDOWN_LINK_REGEX).unwrap(),
    };

    let file_contents = fs::read_to_string(file.path()).unwrap_or_else(|error| {
        println!("Can't read file {:#?} (beacuse {})", file.path(), error);
        String::new()
    });

    for caps in re.captures_iter(&file_contents) {
        links.push(String::from(&caps["link"]));
    }

    links
}

fn visit_doc_files<'a, Fut>(
    dir: &'a Path,
    cb: &'a dyn Fn(DirEntry) -> Fut,
) -> Pin<Box<dyn Future<Output = io::Result<()>> + 'a>>
where
    Fut: Future<Output = ()>,
{
    Box::pin(async move {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)?.filter(|f| {
                !f.as_ref().unwrap().path().is_symlink()
                    && (f.as_ref().unwrap().path().is_dir() || is_doc_file(f.as_ref().unwrap()))
            }) {
                let entry = entry?;
                let path = entry.path();
                assert!(!path.is_symlink());
                if path.is_dir() {
                    visit_doc_files(&path, cb).await?;
                } else {
                    cb(entry).await;
                }
            }
        }
        Ok(())
    })
}

async fn check_link_validity(link: String) {
    match reqwest::get(&link).await {
        Ok(_) => (),
        Err(_) => println!("--- Dead link : {}", link),
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use regex::Captures;

    #[test]
    fn test_regex_doc() {
        let re = Regex::new(DOC_FILE_REGEX).unwrap();
        // first pattern, .md file
        assert!(re.is_match("myDoc.md"));

        // second pattern, .rst file
        assert!(re.is_match("abc.rst"));

        // third pattern, case insensitive README
        assert!(re.is_match("README"));
        assert!(re.is_match("ReAdMe"));
        assert!(!re.is_match("toto"));
    }

    #[test]
    fn test_rst_link_regex() {
        let re = Regex::new(RST_LINK_REGEX).unwrap();
        let text = r"blabla `link <mylink.com>`_ blabla `another one <link.com>`_";
        let captures: Vec<Captures> = re.captures_iter(text).collect();

        assert_eq!(&captures[0]["link"], "mylink.com");
        assert_eq!(&captures[1]["link"], "link.com");
    }
}
