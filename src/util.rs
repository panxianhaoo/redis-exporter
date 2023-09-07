use std::collections::HashMap;

pub fn handle_str_to_map(
    to_handle_str: &str,
    first_match: &str,
    second_match: &str,
) -> HashMap<String, String> {
    let mut res_map: HashMap<String, String> = HashMap::new();
    for ele in to_handle_str.split(first_match) {
        let (key, value) = ele.split_once(second_match).unwrap();
        res_map.insert(key.to_string(), value.to_string());
    }
    res_map
}
