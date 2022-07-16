static AGLENV_FILE_INFO: &str = include_str!("../../data/aglenv_file_info.json");

pub(crate) fn get_aglenv_file_info() -> &'static str {
    AGLENV_FILE_INFO
}
