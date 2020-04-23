use expect::{
    expect,
    matchers::{equal, option::be_none, path::exist, regex::match_regex},
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn it_trashes_the_target_according_to_the_spec() {
    let base_dir = create_tmp_dir();
    let file_path = base_dir.join("delete_me");
    create_text_file(&file_path, "DELETE_ME");
    let xdg_data_path = base_dir.join("xdg_data");
    let trash_path = xdg_data_path.join("Trash");

    let output = Command::new("./target/debug/trash")
        .arg(&file_path)
        .env("XDG_DATA_HOME", &xdg_data_path.to_str().unwrap())
        .output()
        .unwrap();

    expect(&output.status.success()).to(equal(true));

    expect(&file_path).not_to(exist());
    expect(&trash_path.join("files/delete_me")).to(exist());
    expect(&read_text_file(trash_path.join("files/delete_me"))).to(equal("DELETE_ME"));

    expect(&xdg_data_path.join("Trash/info/delete_me.trashinfo")).to(exist());
    let info_contents = read_text_file(trash_path.join("info/delete_me.trashinfo"));
    let mut info_lines = info_contents.lines();
    expect(&info_lines.next().unwrap()).to(equal("[Trash Info]"));
    expect(&info_lines.next().unwrap()).to(equal(
        format!("Path={}", &file_path.to_str().unwrap()).as_str(),
    ));
    expect(&info_lines.next().unwrap()).to(match_regex(
        r"DeletionDate=\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}.\d{6}\+00:00",
    ));
    expect(&info_lines.next()).to(be_none());

    fs::remove_dir_all(base_dir).unwrap();
}

fn create_tmp_dir() -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let tmp_dir = std::env::temp_dir().join(format!("trash_test_{}", now));
    fs::create_dir_all(&tmp_dir).unwrap();
    tmp_dir
}

fn create_text_file(path: &Path, contents: &str) {
    use std::io::Write;

    let mut file = fs::File::create(path).unwrap();
    file.write_all(contents.as_bytes()).unwrap();
}

fn read_text_file<P: AsRef<Path>>(path: P) -> String {
    fs::read_to_string(path).unwrap()
}
