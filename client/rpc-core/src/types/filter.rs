use ethereum_types::{H160, H256, U256};
use serde::de::{DeserializeOwned, Error};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{from_value, Value};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use crate::types::{BlockNumber, Log};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum VariadicValue<T>
    where
        T: DeserializeOwned,
{
    Single(T),
    Multiple(Vec<T>),
    Null,
}

impl<'a, T> Deserialize<'a> for VariadicValue<T>
    where
        T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<VariadicValue<T>, D::Error>
        where
            D: Deserializer<'a>,
    {
        let v: Value = Deserialize::deserialize(deserializer)?;
        if v.is_null() {
            return Ok(VariadicValue::Null);
        }
        from_value(v.clone())
            .map(VariadicValue::Single)
            .or_else(|_| from_value(v).map(VariadicValue::Multiple))
            .map_err(|err| D::Error::custom(format!("Invalid variadic value type: {}", err)))
    }
}

pub type FilterAddress = VariadicValue<H160>;
pub type Topic = VariadicValue<Option<VariadicValue<Option<H256>>>>;
pub type FlatTopic = VariadicValue<Option<H256>>;

#[derive(Debug, PartialEq, Clone, Deserialize, Eq, Hash)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub from_block: Option<BlockNumber>,
    pub to_block: Option<BlockNumber>,
    pub block_hash: Option<H256>,
    pub address: Option<FilterAddress>,
    pub topics: Option<Topic>,
}

#[derive(Debug)]
pub struct FilteredParams {
    pub filter: Option<Filter>,
    flat_topics: Vec<FlatTopic>,
}

impl Default for FilteredParams {
    fn default() -> Self {
        FilteredParams {
            filter: None,
            flat_topics: Vec::new(),
        }
    }
}

impl FilteredParams {
    pub fn new(f: Option<Filter>) -> Self {
        if let Some(f) = f {
            return FilteredParams {
                filter: Some(f.clone()),
                flat_topics: {
                    if let Some(t) = f.clone().topics {
                        Self::flatten(&t)
                    } else {
                        Vec::new()
                    }
                },
            };
        }
        Self::default()
    }

    fn flatten(topic: &Topic) -> Vec<FlatTopic> {
        fn cartesian(lists: &Vec<Vec<Option<H256>>>) -> Vec<Vec<Option<H256>>> {
            let mut res = vec![];
            let mut list_iter = lists.iter();
            if let Some(first_list) = list_iter.next() {
                for &i in first_list {
                    res.push(vec![i]);
                }
            }
            for l in list_iter {
                let mut tmp = vec![];
                for r in res {
                    for &el in l {
                        let mut tmp_el = r.clone();
                        tmp_el.push(el);
                        tmp.push(tmp_el);
                    }
                }
                res = tmp;
            }
            res
        }
        let mut out: Vec<FlatTopic> = Vec::new();
        match topic {
            VariadicValue::Multiple(multi) => {
                let mut foo: Vec<Vec<Option<H256>>> = Vec::new();
                for v in multi {
                    foo.push({
                        if let Some(v) = v {
                            match v {
                                VariadicValue::Single(s) => {
                                    vec![s.clone()]
                                }
                                VariadicValue::Multiple(s) => s.clone(),
                                VariadicValue::Null => {
                                    vec![None]
                                }
                            }
                        } else {
                            vec![None]
                        }
                    });
                }
                for permut in cartesian(&foo) {
                    out.push(FlatTopic::Multiple(permut));
                }
            }
            VariadicValue::Single(single) => {
                if let Some(single) = single {
                    out.push(single.clone());
                }
            }
            VariadicValue::Null => {
                out.push(FlatTopic::Null);
            }
        }
        out
    }

    pub fn replace(&self, log: &Log, topic: FlatTopic) -> Option<Vec<H256>> {
        let mut out: Vec<H256> = Vec::new();
        match topic {
            VariadicValue::Single(value) => {
                if let Some(value) = value {
                    out.push(value);
                }
            }
            VariadicValue::Multiple(value) => {
                for (k, v) in value.into_iter().enumerate() {
                    if let Some(v) = v {
                        out.push(v);
                    } else {
                        out.push(log.topics[k].clone());
                    }
                }
            }
            _ => {}
        };
        if out.len() == 0 {
            return None;
        }
        Some(out)
    }

    pub fn filter_block_range(&self, block_number: u64) -> bool {
        let mut out = true;
        let filter = self.filter.clone().unwrap();
        if let Some(from) = filter.from_block {
            match from {
                BlockNumber::Num(_) => {
                    if from.to_min_block_num().unwrap_or(0 as u64) > block_number {
                        out = false;
                    }
                }
                _ => {}
            }
        }
        if let Some(to) = filter.to_block {
            match to {
                BlockNumber::Num(_) => {
                    if to.to_min_block_num().unwrap_or(0 as u64) < block_number {
                        out = false;
                    }
                }
                BlockNumber::Earliest => {
                    out = false;
                }
                _ => {}
            }
        }
        out
    }

    pub fn filter_block_hash(&self, block_hash: H256) -> bool {
        if let Some(h) = self.filter.clone().unwrap().block_hash {
            if h != block_hash {
                return false;
            }
        }
        true
    }

    pub fn filter_address(&self, log: &Log) -> bool {
        if let Some(input_address) = &self.filter.clone().unwrap().address {
            match input_address {
                VariadicValue::Single(x) => {
                    if log.address != *x {
                        return false;
                    }
                }
                VariadicValue::Multiple(x) => {
                    if !x.contains(&log.address) {
                        return false;
                    }
                }
                _ => {
                    return true;
                }
            }
        }
        true
    }

    pub fn filter_topics(&self, log: &Log) -> bool {
        let mut out: bool = true;
        for topic in self.flat_topics.clone() {
            match topic {
                VariadicValue::Single(single) => {
                    if let Some(single) = single {
                        if !log.topics.starts_with(&vec![single]) {
                            out = false;
                        }
                    }
                }
                VariadicValue::Multiple(multi) => {
                    let mut new_multi = multi;
                    while new_multi
                        .iter()
                        .last()
                        .unwrap_or(&Some(H256::default()))
                        .is_none()
                    {
                        new_multi.pop();
                    }
                    if new_multi.len() > log.topics.len() {
                        out = false;
                        break;
                    }
                    let replaced: Option<Vec<H256>> =
                        self.replace(log, VariadicValue::Multiple(new_multi));
                    if let Some(replaced) = replaced {
                        out = false;
                        if log.topics.starts_with(&replaced[..]) {
                            out = true;
                            break;
                        }
                    }
                }
                _ => {
                    out = true;
                }
            }
        }
        out
    }
}

#[derive(Debug, PartialEq)]
pub enum FilterChanges {
    Logs(Vec<Log>),
    Hashes(Vec<H256>),
    Empty,
}

impl Serialize for FilterChanges {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        match *self {
            FilterChanges::Logs(ref logs) => logs.serialize(s),
            FilterChanges::Hashes(ref hashes) => hashes.serialize(s),
            FilterChanges::Empty => (&[] as &[Value]).serialize(s),
        }
    }
}

#[derive(Debug, Clone)]
pub enum FilterType {
    Block,
    PendingTransaction,
    Log(Filter),
}

#[derive(Debug, Clone)]
pub struct FilterPoolItem {
    pub last_poll: BlockNumber,
    pub filter_type: FilterType,
    pub at_block: u64,
}

pub type FilterPool = Arc<Mutex<BTreeMap<U256, FilterPoolItem>>>;