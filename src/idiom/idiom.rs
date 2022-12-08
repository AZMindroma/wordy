use std::collections::HashMap;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use bimap::BiMap;
use super::top_freqs::TopFreqs;
const PLACE_VOC_LEN: usize = 200;
const PERSON_VOC_LEN: usize = 100;
const UNIQUENESS: f32 = PERSON_VOC_LEN as f32;
// computed so that (INV-1.)*UNIQUENESS = 1.
const INV: f32 = 1.+1./UNIQUENESS;

lazy_static! {
    static ref RE_TOKEN: Regex = Regex::new(r"\w+").unwrap();
}

fn tokenize(text: String) -> Vec<(String, f32)> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for token in RE_TOKEN.find_iter(&text) {
        *counts.entry(token.as_str().to_string()).or_default() += 1;
    }
    counts.into_iter().map(|(k, v)| (k, v as f32)).collect()
}

pub struct Idioms {
    places: HashMap<String, TopFreqs<PLACE_VOC_LEN>>,
    people: HashMap<String, TopFreqs<PERSON_VOC_LEN>>,
    tokens: BiMap<String, usize>,
}


impl Idioms {
    pub fn new() -> Self {
        let mut tokens = BiMap::new();
        // reserve slot 0 for empty string
        tokens.insert(String::new(), 0);
        Self {
            places: HashMap::new(), people: HashMap::new(), tokens
        }
    }

    pub fn update(&mut self, place: String, person: String, message: String) {
        let place_voc = self.places.entry(place).or_insert(TopFreqs::new());
        let user_voc = self.people.entry(person).or_insert(TopFreqs::new());
        let tokens = tokenize(message);
        for (token, value) in tokens {
            let idx = match self.tokens.get_by_left(&token) {
                Some(v) => *v,
                None => {
                    let v = self.tokens.len();
                    self.tokens.insert(token, v);
                    v
                }
            };
            place_voc.add(idx, value);
            let inctx_value = (INV-place_voc.get(&idx))*UNIQUENESS;
            user_voc.add(idx, inctx_value);
        }
    }

    pub fn idiom(&self, person: String) -> Vec<(String, f32)> {
        let res = match self.people.get(&person) {
            Some(voc) => voc.data.clone().into_iter()
                .filter(|(idx, _)| *idx != 0).collect_vec(),
            None => Vec::new()
        };
        res.into_iter()
        .map(|(idx, v)| (self.tokens.get_by_right(&idx).unwrap().clone(), v))
        .collect_vec()
    }

    pub fn people(&self) -> Vec<&String> {
        self.people.keys().collect_vec()
    }
}