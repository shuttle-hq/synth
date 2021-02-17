use super::prelude::*;

pub trait CategoricalType: Eq + Hash + Clone + FromStr {}

impl<T> CategoricalType for T where T: Eq + Hash + Clone + FromStr {}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(from = "CategoricalShadow<T>")]
pub struct Categorical<T: CategoricalType> {
    #[serde(flatten)]
    pub(crate) seen: HashMap<T, u64>,
    #[serde(skip_serializing)]
    pub(crate) total: u64,
}

fn categorical_map_de<'de, D, T>(deserializer: D) -> Result<HashMap<T, u64>, D::Error>
where
    T: CategoricalType,
    D: Deserializer<'de>,
{
    let hm_string_keys: HashMap<String, u64> = HashMap::deserialize(deserializer)?;
    let mut hm_final: HashMap<T, u64> = HashMap::new();
    for (k, v) in hm_string_keys {
        hm_final.insert(
            k.parse()
                .map_err(|_| serde::de::Error::custom(format!("could not deserialize {}", k)))?,
            v,
        );
    }
    if hm_final.is_empty() {
        return Err(serde::de::Error::custom(
            "Could not create categorical. Categorical must be non-empty (i.e. have at least one value in 'seen').",
        ));
    }
    Ok(hm_final)
}

impl<T: CategoricalType> Categorical<T> {
    pub fn push(&mut self, t: T) {
        match self.seen.get_mut(&t) {
            Some(occurrences) => {
                *occurrences += 1;
            }
            None => {
                self.seen.insert(t, 1);
            }
        };
        self.total += 1;
    }
}

/// This struct purely serves as an intermediary to check invariants in the
/// Categorical struct
#[derive(Deserialize)]
struct CategoricalShadow<T: CategoricalType> {
    #[serde(deserialize_with = "categorical_map_de")]
    #[serde(flatten)]
    seen: HashMap<T, u64>,
}

impl<T: CategoricalType> From<CategoricalShadow<T>> for Categorical<T> {
    fn from(shadow: CategoricalShadow<T>) -> Self {
        let total: u64 = shadow.seen.values().sum();
        Categorical {
            seen: shadow.seen,
            total,
        }
    }
}

impl<T: CategoricalType> Distribution<T> for Categorical<T> {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> T {
        let f = rng.gen_range(0.0, 1.0);
        let mut index = (f * self.total as f64).floor() as i64;
        for (k, v) in self.seen.iter() {
            index -= *v as i64;
            if index < 0 {
                return k.clone();
            }
        }
        unreachable!()
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_push() {
        let categorical_json = json!(
            {
                "a" : 5,
                "b" : 10,
            }
        );
        let mut categorical: Categorical<String> =
            serde_json::from_value(categorical_json).unwrap();
        categorical.push("a".to_string());
        categorical.push("b".to_string());
        assert_eq!(categorical.seen.get("a").unwrap(), &6);
        assert_eq!(categorical.seen.get("b").unwrap(), &11);
        assert_eq!(categorical.total, 17);
    }

    #[test]
    fn test_sample() {
        use rand::distributions::Distribution;
        let categorical_json = json!(
            {
                "a" : 5,
                "b" : 10,
            }
        );
        let categorical: Categorical<String> = serde_json::from_value(categorical_json).unwrap();
        let mut rng = rand::thread_rng();
        for _ in 1..100 {
            match categorical.sample(&mut rng).as_ref() {
                "a" => {}
                "b" => {}
                _ => panic!("Should only get 'a's and 'b's"),
            }
        }
    }

    #[test]
    fn test_categorical_empty_invariant() {
        let categorical_json = json!({});
        assert!(serde_json::from_value::<Categorical<String>>(categorical_json).is_err())
    }
}
