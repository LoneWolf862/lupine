//! Pure, deterministic name matching. Callers supply only eligible visible targets.
use crate::{state::GameState, world::World};

pub fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[derive(Debug, Clone)]
pub struct Candidate {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
}
impl Candidate {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            aliases: Vec::new(),
        }
    }
    fn labels(&self) -> impl Iterator<Item = &str> {
        std::iter::once(self.name.as_str()).chain(self.aliases.iter().map(String::as_str))
    }
}

pub fn items(world: &World, state: &GameState) -> Vec<Candidate> {
    world
        .items
        .iter()
        .filter(|i| state.is_accessible(&i.id))
        .map(|i| Candidate {
            id: i.id.clone(),
            name: i.name.clone(),
            aliases: i.aliases.clone(),
        })
        .collect()
}

pub fn matches<'a>(query: &str, candidates: &'a [Candidate]) -> Vec<&'a Candidate> {
    let query = normalize(query);
    if query.is_empty() {
        return Vec::new();
    }
    let ids = candidates
        .iter()
        .filter(|c| normalize(&c.id) == query)
        .collect::<Vec<_>>();
    if !ids.is_empty() {
        return ids;
    }
    let exact = candidates
        .iter()
        .filter(|c| c.labels().any(|label| normalize(label) == query))
        .collect::<Vec<_>>();
    if !exact.is_empty() {
        return exact;
    }
    candidates
        .iter()
        .filter(|c| c.labels().any(|label| normalize(label).contains(&query)))
        .collect()
}

/// Bounded Unicode edit distance. Suggestions are advisory and never resolved actions.
pub fn distance(a: &str, b: &str) -> Option<usize> {
    let a: Vec<_> = a.chars().take(81).collect();
    let b: Vec<_> = b.chars().take(81).collect();
    if a.len() > 80 || b.len() > 80 || a.len().abs_diff(b.len()) > 2 {
        return None;
    }
    let mut previous = (0..=b.len()).collect::<Vec<_>>();
    let mut older = previous.clone();
    for (row, c) in a.iter().enumerate() {
        let mut next = vec![row + 1];
        for (col, other) in b.iter().enumerate() {
            next.push(
                (previous[col + 1] + 1)
                    .min(next[col] + 1)
                    .min(previous[col] + usize::from(c != other)),
            );
            if row > 0 && col > 0 && *c == b[col - 1] && a[row - 1] == *other {
                next[col + 1] = next[col + 1].min(older[col - 1] + 1);
            }
        }
        older = previous;
        previous = next;
    }
    (previous[b.len()] <= 2).then_some(previous[b.len()])
}

pub fn suggestion<'a>(query: &str, candidates: &'a [Candidate]) -> Option<&'a Candidate> {
    let query = normalize(query);
    if query.chars().count() < 3 {
        return None;
    }
    let mut ranked = candidates
        .iter()
        .filter_map(|c| {
            c.labels()
                .chain(std::iter::once(c.id.as_str()))
                .filter_map(|s| distance(&query, &normalize(s)))
                .min()
                .map(|d| (d, c))
        })
        .collect::<Vec<_>>();
    ranked.sort_by_key(|(d, c)| (*d, &c.id));
    let (distance, candidate) = *ranked.first()?;
    if ranked.get(1).is_some_and(|(d, _)| *d == distance) {
        None
    } else {
        Some(candidate)
    }
}

/// Noninteractive callers must return ambiguity, never pick by map iteration order.
pub fn unique(query: &str, candidates: &[Candidate]) -> Result<String, String> {
    match matches(query, candidates).as_slice() {
        [candidate] => Ok(candidate.id.clone()),
        [] => {
            let mut error = format!("You don't see '{query}' among the available targets.");
            if let Some(c) = suggestion(query, candidates) {
                error.push_str(&format!(" Did you mean '{}' ({})?", c.name, c.id));
            }
            Err(error)
        }
        many => Err(format!(
            "Which {query} do you mean: {}?",
            many.iter()
                .map(|c| c.name.as_str())
                .collect::<Vec<_>>()
                .join(" or ")
        )),
    }
}
