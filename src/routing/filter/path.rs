use std::{collections::HashMap, fmt, sync::Arc};

use once_cell::sync::Lazy;
use parking_lot::RwLock;
use regex::Regex;

use crate::routing::PathState;

pub trait PathWisp: Send + Sync + 'static + fmt::Debug {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Self>()
    }
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
    fn detect(&self, state: &mut PathState) -> bool;
}

pub trait WispBuilder: Send + Sync {
    fn build(
        &self,
        name: String,
        sign: String,
        args: Vec<String>,
    ) -> Result<Box<dyn PathWisp>, String>;
}

type WispBuilderMap = RwLock<HashMap<String, Arc<Box<dyn WispBuilder>>>>;

static WISP_BUILDERS: Lazy<WispBuilderMap> = Lazy::new(|| {
    let mut map: HashMap<String, Arc<Box<dyn WispBuilder>>> = HashMap::with_capacity(8);
    RwLock::new(map)
});

fn is_num(ch: char) -> bool {
    ch.is_ascii_digit()
}
fn is_hex(ch: char) -> bool {
    ch.is_ascii_hexdigit()
}

#[derive(Debug)]
struct RegexWisp {
    name: String,
    regex: Regex,
}
impl RegexWisp {
    fn new(name: String, regex: Regex) -> RegexWisp {
        RegexWisp { name, regex }
    }
}
impl PartialEq for RegexWisp {
    fn eq(&self, other: &Self) -> bool {
        self.regex.as_str() == other.regex.as_str()
    }
}
impl PathWisp for RegexWisp {
    fn detect(&self, state: &mut PathState) -> bool {
        if self.name.starts_with('*') {
            let rest = state.all_rest();
            if rest.is_none() {
                return false;
            }
            let rest = &*rest.unwrap();

            if !rest.is_empty() || self.name.starts_with("**") {
                let cap = self.regex.captures(rest).and_then(|caps| caps.get(0));
                if let Some(cap) = cap {
                    let cap = cap.as_str().to_owned();
                    state.forward(cap.len());
                    state.params.insert(self.name.clone(), cap);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            let picked = state.pick();
            if picked.is_none() {
                return false;
            }
            let picked = picked.unwrap();
            let cap = self.regex.captures(picked).and_then(|caps| caps.get(0));
            if let Some(cap) = cap {
                let cap = cap.as_str().to_owned();
                state.forward(cap.len());
                state.params.insert(self.name.clone(), cap);
                true
            } else {
                false
            }
        }
    }
}

pub struct RegexWispBuilder(Regex);
impl RegexWispBuilder {
    pub fn new(checker: Regex) -> Self {
        Self(checker)
    }
}
impl WispBuilder for RegexWispBuilder {
    fn build(
        &self,
        name: String,
        sign: String,
        args: Vec<String>,
    ) -> Result<Box<dyn PathWisp>, String> {
        Ok(Box::new(RegexWisp {
            name,
            regex: self.0.clone(),
        }))
    }
}

struct CharWisp<C> {
    name: String,
    checker: Arc<C>,
    min_width: usize,
    max_width: Option<usize>,
}
impl<C> fmt::Debug for CharWisp<C> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CharWisp {{ name: {:?}, min_width: {:?}, max_width: {:?} }}",
            self.name, self.min_width, self.max_width
        )
    }
}
impl<C> PathWisp for CharWisp<C>
where
    C: Fn(char) -> bool + Send + Sync + 'static,
{
    fn detect(&self, state: &mut PathState) -> bool {
        let picked = state.pick();
        if picked.is_none() {
            return false;
        }
        let picked = picked.unwrap();
        if let Some(max_width) = self.max_width {
            let mut chars = Vec::with_capacity(max_width);
            for ch in picked.chars() {
                if (self.checker)(ch) {
                    chars.push(ch);
                }
                if chars.len() == max_width {
                    state.forward(max_width);
                    state
                        .params
                        .insert(self.name.clone(), chars.into_iter().collect());
                    return true;
                }
            }
            if chars.len() >= self.min_width {
                state.forward(chars.len());
                state
                    .params
                    .insert(self.name.clone(), chars.into_iter().collect());
                true
            } else {
                false
            }
        } else {
            let mut chars = Vec::with_capacity(16);
            for ch in picked.chars() {
                if (self.checker)(ch) {
                    chars.push(ch);
                }
            }
            if chars.len() >= self.min_width {
                state.forward(chars.len());
                state
                    .params
                    .insert(self.name.clone(), chars.into_iter().collect());
                true
            } else {
                false
            }
        }
    }
}

pub struct CharWispBuilder<C>(Arc<C>);
impl<C> CharWispBuilder<C> {
    pub fn new(checker: C) -> Self {
        Self(Arc::new(checker))
    }
}

#[derive(Debug)]
struct CombWisp(Vec<Box<dyn PathWisp>>);
impl PathWisp for CombWisp {
    fn detect(&self, state: &mut PathState) -> bool {
        let original_cursor = state.cursor;
        for child in &self.0 {
            if !child.detect(state) {
                state.cursor = original_cursor;
                return false;
            }
        }
        true
    }
}

#[derive(Debug, Eq, PartialEq)]
struct NameWisp(String);
impl PathWisp for NameWisp {
    fn detect(&self, state: &mut PathState) -> bool {
        if self.0.starts_with('*') {
            let rest = state.all_rest().unwrap_or_default();
            if !rest.is_empty() || self.0.starts_with("**") {
                let rest = rest.to_string();
                state.params.insert(self.0.clone(), rest);
                state.cursor.0 = state.parts.len();
                true
            } else {
                false
            }
        } else {
            let picked = state.pick();
            if picked.is_none() {
                return false;
            }
            let picked = picked.unwrap().to_owned();
            state.forward(picked.len());
            state.params.insert(self.0.clone(), picked);
            true
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
struct ConstWisp(String);
impl PathWisp for ConstWisp {
    fn detect(&self, state: &mut PathState) -> bool {
        let picked = state.pick();
        if picked.is_none() {
            return false;
        }
        let picked = picked.unwrap();
        if picked.starts_with(&self.0) {
            state.forward(self.0.len());
            true
        } else {
            false
        }
    }
}

struct PathParser {
    offset: usize,
    path: Vec<char>,
}

impl PathParser {
    fn new(raw_value: &str) -> PathParser {
        PathParser {
            offset: 0,
            path: raw_value.chars().collect(),
        }
    }
    fn next(&mut self, skip_blanks: bool) -> Option<char> {
        if self.offset < self.path.len() - 1 {
            self.offset += 1;
            if skip_blanks {
                self.skip_blanks();
            }
            Some(self.path[self.offset])
        } else {
            self.offset = self.path.len();
            None
        }
    }
    fn peek(&self, skip_blanks: bool) -> Option<char> {
        if self.offset < self.path.len() - 1 {
            if skip_blanks {
                let mut offset = self.offset + 1;
                let mut ch = self.path[offset];
                while ch == ' ' || ch == '\t' {
                    offset += 1;
                    if offset >= self.path.len() {
                        return None;
                    }
                    ch = self.path[offset];
                }
                Some(ch)
            } else {
                Some(self.path[self.offset + 1])
            }
        } else {
            None
        }
    }
    fn curr(&self) -> Option<char> {
        self.path.get(self.offset).copied()
    }

    fn scan_ident(&mut self) -> Result<String, String> {
        let mut ident = "".to_owned();
        let mut ch = self
            .curr()
            .ok_or_else(|| "current position is out of index when scan ident".to_owned())?;

        let characters = vec!['/', ':', '<', '>', '[', ']', '(', ')'];
        while !characters.contains(&ch) {
            ident.push(ch);
            if let Some(c) = self.next(false) {
                ch = c;
            } else {
                break;
            }
        }
        if ident.is_empty() {
            Err("ident segment is empty".to_owned())
        } else {
            Ok(ident)
        }
    }
    fn scan_regex(&mut self) -> Result<String, String> {
        let mut regex = "".to_owned();
        let mut ch = self
            .curr()
            .ok_or_else(|| "current position is out of index when scan ident".to_owned())?;
        loop {
            regex.push(ch);
            if let Some(c) = self.next(false) {
                ch = c;
                if ch == '/' {
                    let pch = self.peek(true);
                    if pch.is_none() {
                        return Err("path end but regex is not ended".to_owned());
                    } else if let Some('>') = pch {
                        self.next(true);
                        break;
                    }
                }
            } else {
                break;
            }
        }
        if regex.is_empty() {
            Err("regex segment is empty".to_owned())
        } else {
            Ok(regex)
        }
    }
    fn scan_const(&mut self) -> Result<String, String> {
        let mut cnst = "".to_owned();
        let mut ch = self
            .curr()
            .ok_or_else(|| "current position is out of index when scan ident".to_owned())?;

        let characters = vec!['/', ':', '<', '>', '[', ']', '(', ')'];
        while !characters.contains(&ch) {
            cnst.push(ch);
            if let Some(c) = self.next(false) {
                ch = c;
            } else {
                break;
            }
        }
        if cnst.is_empty() {
            Err("const segment is empty".to_owned())
        } else {
            Ok(cnst)
        }
    }
    fn skip_blanks(&mut self) {
        todo!()
    }
}

pub struct PathFilter {
    raw_value: String,
    path_wisps: Vec<Box<dyn PathWisp>>,
}
