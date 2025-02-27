use anyhow::Context;

use super::function_traits::BorrowFromStr;
use std::borrow::Cow;

type HeaderVec<'a> = Vec<(Cow<'a, str>, Cow<'a, str>)>;
#[derive(serde::Deserialize, Debug)]
#[serde(expecting = "an http request object, e.g. '{\"url\":\"http://example.com\"}'")]
pub(super) struct HttpFetchRequest<'b> {
    #[serde(borrow)]
    pub url: Cow<'b, str>,
    #[serde(borrow)]
    pub method: Option<Cow<'b, str>>,
    pub timeout_ms: Option<u64>,
    #[serde(borrow, deserialize_with = "deserialize_map_to_vec_pairs")]
    pub headers: HeaderVec<'b>,
    #[serde(borrow)]
    pub body: Option<Cow<'b, serde_json::value::RawValue>>,
}

fn deserialize_map_to_vec_pairs<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<HeaderVec<'de>, D::Error> {
    struct Visitor;

    impl<'de> serde::de::Visitor<'de> for Visitor {
        type Value = Vec<(Cow<'de, str>, Cow<'de, str>)>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some((key, value)) = map.next_entry()? {
                vec.push((key, value));
            }
            Ok(vec)
        }
    }

    deserializer.deserialize_map(Visitor)
}

impl<'a> BorrowFromStr<'a> for HttpFetchRequest<'a> {
    fn borrow_from_str(s: Cow<'a, str>) -> anyhow::Result<Self> {
        Ok(if s.starts_with("http") {
            HttpFetchRequest {
                url: s,
                method: None,
                headers: Vec::new(),
                body: None,
                timeout_ms: None,
            }
        } else {
            match s {
                Cow::Borrowed(s) => serde_json::from_str(s),
                Cow::Owned(ref s) => serde_json::from_str::<HttpFetchRequest<'_>>(s)
                    .map(HttpFetchRequest::into_owned),
            }
            .with_context(|| format!("Invalid http fetch request definition: {s}"))?
        })
    }
}

impl HttpFetchRequest<'_> {
    fn into_owned(self) -> HttpFetchRequest<'static> {
        HttpFetchRequest {
            url: Cow::Owned(self.url.into_owned()),
            method: self.method.map(Cow::into_owned).map(Cow::Owned),
            headers: self
                .headers
                .into_iter()
                .map(|(k, v)| (Cow::Owned(k.into_owned()), Cow::Owned(v.into_owned())))
                .collect(),
            body: self.body.map(Cow::into_owned).map(Cow::Owned),
            timeout_ms: self.timeout_ms,
        }
    }
}
