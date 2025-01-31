pub(crate) fn parse(res: &Vec<u8>) -> Vec<Vec<String>> {
    let info_string = String::from_utf8(res.to_owned()).unwrap();
    info_string
        .split("\n")
        .map(|it| {
            it.split("\t")
                .map(|it| it.to_string())
                .collect::<Vec<String>>()
        })
        .collect()
}
