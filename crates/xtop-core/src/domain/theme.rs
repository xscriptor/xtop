use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

fn hex_to_rgb(hex: &str) -> [u8; 3] {
    let hex = hex.trim_start_matches('#');
    let r = u8::from_str_radix(hex.get(0..2).unwrap_or("00"), 16).unwrap_or(0);
    let g = u8::from_str_radix(hex.get(2..4).unwrap_or("00"), 16).unwrap_or(0);
    let b = u8::from_str_radix(hex.get(4..6).unwrap_or("00"), 16).unwrap_or(0);
    [r, g, b]
}

#[derive(Clone, Debug, Serialize)]
pub struct Theme {
    pub name: String,
    pub palette: [[u8; 3]; 16],
}

impl Theme {
    pub fn bg(&self) -> &[u8; 3] {
        &self.palette[0]
    }

    pub fn fg(&self) -> &[u8; 3] {
        &self.palette[7]
    }
}

// Custom deserializer that reads palette as [String; 16] (hex) and converts to [[u8; 3]; 16]
impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Name,
            Palette,
        }

        struct ThemeVisitor;
        impl<'de> Visitor<'de> for ThemeVisitor {
            type Value = Theme;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Theme")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Theme, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut name = None;
                let mut palette: Option<[String; 16]> = None;

                while let Some(key) = map.next_key::<Field>()? {
                    match key {
                        Field::Name => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value::<String>()?);
                        }
                        Field::Palette => {
                            if palette.is_some() {
                                return Err(de::Error::duplicate_field("palette"));
                            }
                            palette = Some(map.next_value::<[String; 16]>()?);
                        }
                    }
                }

                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let palette_str =
                    palette.ok_or_else(|| de::Error::missing_field("palette"))?;

                let mut palette = [[0u8; 3]; 16];
                for (i, hex) in palette_str.iter().enumerate() {
                    palette[i] = hex_to_rgb(hex);
                }

                Ok(Theme { name, palette })
            }
        }

        deserializer.deserialize_struct("Theme", &["name", "palette"], ThemeVisitor)
    }
}

pub fn hex_to_rgb_pub(hex: &str) -> [u8; 3] {
    hex_to_rgb(hex)
}
