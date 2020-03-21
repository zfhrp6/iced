use oauth_client::Token;
use std::fs;
use std::io::{BufReader, Read};

#[derive(Deserialize, Debug, Default)]
struct User {
    screen_name: Option<String>,
    app_key: String,
    app_secret: String,
    oauth_token: String,
    oauth_token_secret: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Conf {
    account: User,
}

pub fn read_conf() -> Conf {
    let mut s = String::new();
    fs::File::open("C:\\workspace\\iced\\examples\\clock\\user.toml")
        .map(|f| BufReader::new(f))
        .map_err(|e| e.to_string())
        .expect("cannot load")
        .read_to_string(&mut s)
        .expect("cannot load");

    toml::from_str(&s).expect("cannot load")
}

pub fn hoge(text: &str, conf: &Conf) {
    let ct = Token::new(&conf.account.app_key, &conf.account.app_secret);
    let at =
        Token::new(&conf.account.oauth_token, &conf.account.oauth_token_secret);

    let _ = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(twitter_api::update_status(&ct, &at, &format!("{}", text)));
}
