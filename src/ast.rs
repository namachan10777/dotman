use std::collections::HashMap;
use yaml_rust::Yaml;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Real(f64),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),
    Hash(HashMap<String, Value>),
}

#[derive(Debug)]
pub enum ConvError {
    YamlAlias,
    YamlBadValue,
    YamlNull,
    YamlInvalidHashKey(Yaml),
    YamlCannotParseReal(String),
}
impl Value {
    pub fn from_yaml(yaml: Yaml) -> Result<Self, ConvError> {
        match yaml {
            Yaml::Alias(_) => Err(ConvError::YamlAlias),
            Yaml::Array(arr) => Ok(Value::Array(
                arr.into_iter()
                    .map(Self::from_yaml)
                    .collect::<Result<Vec<_>, _>>()?,
            )),
            Yaml::BadValue => Err(ConvError::YamlBadValue),
            Yaml::Boolean(b) => Ok(Value::Bool(b)),
            Yaml::Hash(hash) => Ok(Value::Hash(
                hash.into_iter()
                    .map(|(key, val)| match key {
                        Yaml::Boolean(b) => Ok((b.to_string(), Self::from_yaml(val)?)),
                        Yaml::Integer(i) => Ok((i.to_string(), Self::from_yaml(val)?)),
                        Yaml::Real(s) => Ok((s, Self::from_yaml(val)?)),
                        Yaml::String(s) => Ok((s, Self::from_yaml(val)?)),
                        Yaml::Null => Ok(("null".to_owned(), Self::from_yaml(val)?)),
                        yaml => Err(ConvError::YamlInvalidHashKey(yaml)),
                    })
                    .collect::<Result<HashMap<_, _>, _>>()?,
            )),
            Yaml::Integer(i) => Ok(Value::Int(i)),
            Yaml::Null => Err(ConvError::YamlNull),
            Yaml::Real(r) => Ok(Value::Real(
                r.parse().map_err(|_| ConvError::YamlCannotParseReal(r))?,
            )),
            Yaml::String(s) => Ok(Value::Str(s)),
        }
    }
}

impl Value {
    pub fn as_hash(&self) -> Option<&HashMap<String, Self>> {
        match &self {
            Value::Hash(h) => Some(h),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Self>> {
        if let Value::Array(arr) = &self {
            Some(arr)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Value::Str(s) = &self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = &self {
            Some(*b)
        } else {
            None
        }
    }
}

pub fn not_allowed_member<'a, K, T>(map: &'a HashMap<K, T>, allowed: &[&K]) -> Vec<(&'a K, &'a T)>
where
    K: PartialEq,
{
    map.iter()
        .filter_map(|(key, val)| {
            if allowed.contains(&key) {
                None
            } else {
                Some((key, val))
            }
        })
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod test {
    use super::*;
    use maplit::hashmap;

    #[test]
    fn test_not_allowed_member() {
        let map = hashmap! {
            "a" => 1,
            "b" => 2,
            "c" => 3,
            "d" => 4,
        };
        let mut not_allowed = not_allowed_member(&map, &[&"a", &"c"]);
        not_allowed.sort_by(|(k1, _), (k2, _)| k1.cmp(&k2));

        assert_eq!(not_allowed, vec![(&"b", &2), (&"d", &4)]);
    }
}
