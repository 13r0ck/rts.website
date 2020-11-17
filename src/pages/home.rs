use anyhow::Error;
use serde_derive::{Deserialize, Serialize};
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use wasm_bindgen::prelude::*;
use yew::format::{Text, Nothing};
use yew::prelude::*;
use easy_hasher::easy_hasher::*;
use rand::prelude::*;
use rand_seeder::{Seeder, SipHasher};
use rand_pcg::Pcg64;
use rand::seq::SliceRandom;

struct State {
    passwd: Option<String>,
    // Hashed (load/click/delta_fetch) time as seed for password
    site_load_time: Option<f64>,
    button_press_time: Option<f64>,
    delta_fetch: Option<f64>,
    //words: Option<Vec<String>>,
    words: Option<Vec<String>>,
    hash: Option<String>,
}

pub struct PasswdGen {
    state: State,
    link: ComponentLink<Self>,
    task: Option<FetchTask>,
}

pub enum Msg {
    CreatePasswd,
    StartFetch,
    GetWordsSuccess(String),
    GetWordsError,
}

impl Component for PasswdGen {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let now = js_sys::Date::new_0().get_time();
        link.send_message(Msg::StartFetch);
        Self {
            state: State {
                passwd: None,
                site_load_time: Some(now),
                button_press_time: None,
                delta_fetch: None,
                words: None,
                hash: None,
            },
            link,
            task: None,
        }
    }

    fn update(&mut self, message: Self::Message) -> ShouldRender {
        match message {
            Msg::StartFetch => {
                // grab the list of words via a fetch
                let get_request = Request::get("/sections/passwd/words.txt")
                    .body(Nothing)
                    .expect("Failed get request");
                let callback = self.link.callback(|response: Response<Text>| {
                    if let (_, Ok(body)) = response.into_parts() {
                        return Msg::GetWordsSuccess(body);
                    } else {
                        Msg::GetWordsError
                    }
                });
                self.task = Some(FetchService::fetch(get_request, callback).unwrap());
                true
            }
            Msg::CreatePasswd => {
                let now = js_sys::Date::new_0().get_time();
                self.state.button_press_time = Some(now);
                // b/c rand reliablilty on wasm is spotty we cannot use system
                // seed sources to create the seed. Rather we create one from the hash
                // from physical values, time (page load time), and virtual distance
                // from server (delta time to fetch from server). And time of button
                // press so that hash is different every time
                let seed = keccak512(&format!("{}{}{}", 
                                        &self.state.button_press_time.as_ref().unwrap(),
                                        &self.state.site_load_time.as_ref().unwrap(),
                                        &self.state.delta_fetch.as_ref().unwrap())).to_hex_string();
                // generate random object (rng) with seed
                let mut rng: Pcg64 = Seeder::from(seed).make_rng();
                let mut rand_num_list: Vec<String> = vec!["".to_string(),"".to_string(),rng.gen_range(0,10).to_string()];
                rand_num_list.shuffle(&mut rng);
                let word_list = self.state.words.as_ref().unwrap()[33..].to_vec();
                let seperator = "@%+\\/`!#$^?:,(){}[]~-_.\"".chars().choose(&mut rng).unwrap();
                let mut three_words: [String; 3] = [
                                    word_list.choose(&mut rng).unwrap().clone(),
                                    word_list.choose(&mut rng).unwrap().clone(),
                                    word_list.choose(&mut rng).unwrap().clone()];
                let tw_index = rng.gen_range(0,3);
                three_words[tw_index] = three_words[tw_index].to_uppercase();
                self.state.passwd = Some(format!("{}{}{}{}{}{}{}{}",
                        three_words[0],
                        rand_num_list[0],
                        seperator,
                        three_words[1],
                        rand_num_list[1],
                        seperator,
                        three_words[2],
                        rand_num_list[2]
                ));
                true
            },
            Msg::GetWordsSuccess(words_txt) => {
                // if the fetch is successfull with grabbing words from server
                println!("get words sucessfully");
                let now = js_sys::Date::new_0().get_time();
                if let Some(site_load_time) = self.state.site_load_time {
                    let delta_fetch = now - site_load_time;
                    self.state.delta_fetch = Some(delta_fetch);
                }
                self.state.words = Some(words_txt.lines()
                                        .map(|s| s.to_string())
                                        .collect());
                true
            },
            Msg::GetWordsError => {
                // There was some error, I would rather not show the password generator
                // than to show a possible insecure one.
                println!("There was an error generating the password. Password gen hidden");
                false
            }
        }
    }

    fn change(&mut self, _: Self::Properties) -> ShouldRender {
        true
    }

    fn view(&self) -> Html {
        if let Some(_) = &self.state.passwd {
            html! {
                <div>
                    <p>{&self.state.passwd.as_ref().unwrap()}</p>
                    <p>{&self.state.button_press_time.as_ref().unwrap()}</p>
                    <p>{&self.state.delta_fetch.as_ref().unwrap()}</p>
                    //<p>{&self.state.hash.as_ref().unwrap()}</p>
                    //<p>{&self.state.words.as_ref().unwrap()}</p>
                    <button onclick=self.link.callback(move |_| Msg::CreatePasswd)>{"Generate Password"}</button>
                </div>
            }
        } else {
            html! {
                <div>
                    <button onclick=self.link.callback(move |_| Msg::CreatePasswd)>{"Generate Password"}</button>
                </div>
            }
        }
    }
}
