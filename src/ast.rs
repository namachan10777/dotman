//! Abstraction layer of configuration language.
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

/// Error about conversion of ast
#[derive(Debug)]
pub enum ConvError {
    /// Yaml alias is unsupported
    YamlAlias,
    /// Yaml alias is unsupported
    YamlBadValue,
    /// Yaml null is unsupported
    YamlNull,
    /// Yaml hashkey must be string
    YamlInvalidHashKey(Yaml),
    /// Failed to parse yaml real
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

fn not_allowed_member<'a, K, T>(map: &'a HashMap<K, T>, allowed: &[&K]) -> Vec<(&'a K, &'a T)>
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

/// Check to see if invalid members are included.
/// This function will report [UnrecognizedMembers](../enum.Error.html) with passed prefix.
pub fn verify_hash(
    hash: &HashMap<String, Value>,
    allowed: &[&str],
    prefix: Option<&str>,
) -> Result<(), crate::Error> {
    let allowed = allowed.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
    let not_allowed = not_allowed_member(hash, allowed.iter().collect::<Vec<_>>().as_slice());
    if !not_allowed.is_empty() {
        return Err(crate::Error::UnrecognizedMembers {
            prefix: prefix.map(|s| s.to_owned()),
            members: not_allowed
                .iter()
                .map(|(k, v)| ((*k).clone(), (*v).clone()))
                .collect::<Vec<_>>(),
        });
    } else {
        Ok(())
    }
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
        not_allowed.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

        assert_eq!(not_allowed, vec![(&"b", &2), (&"d", &4)]);
    }
}
