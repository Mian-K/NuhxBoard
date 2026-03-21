use bitflags::bitflags;
use schemars::JsonSchema;
use serde::{
    Deserialize, Serialize,
    de::Deserializer,
    ser::{SerializeSeq, Serializer},
};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Style {
    /// Background color of the window. Will be overridden by background image if present.
    pub background_color: NohRgb,
    pub background_image_file_name: Option<String>,
    pub default_key_style: DefaultKeyStyle,
    pub default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle,
    #[serde(with = "CustomMap")]
    pub element_styles: HashMap<u32, ElementStyle>,
}

// This allows `HashMap<u32, ElementStyle>` to be serialized as a list of `{Key: u32, Value: ElementStyle}`
struct CustomMap;
impl CustomMap {
    pub fn serialize<S: Serializer>(
        map: &HashMap<u32, ElementStyle>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for (key, value) in map {
            seq.serialize_element(&KeyValue {
                key: *key,
                value: value.clone(),
            })?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<HashMap<u32, ElementStyle>, D::Error> {
        Ok(Vec::<KeyValue>::deserialize(deserializer)?
            .into_iter()
            .map(|item| (item.key, item.value))
            .collect())
    }
}

impl JsonSchema for CustomMap {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "CustomMap".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        generator.subschema_for::<Vec<KeyValue>>()
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "PascalCase")]
struct KeyValue {
    key: u32,
    value: ElementStyle,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct NohRgb {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
}

impl NohRgb {
    pub const BLACK: NohRgb = NohRgb {
        red: 0.0,
        green: 0.0,
        blue: 0.0,
    };

    pub const WHITE: NohRgb = NohRgb {
        red: 255.0,
        green: 255.0,
        blue: 255.0,
    };

    pub const DEFAULT_GRAY: NohRgb = NohRgb {
        red: 100.0,
        green: 100.0,
        blue: 100.0,
    };
}

impl From<NohRgb> for colorgrad::Color {
    fn from(value: NohRgb) -> Self {
        colorgrad::Color::new(
            value.red / 255.0,
            value.green / 255.0,
            value.blue / 255.0,
            1.0,
        )
    }
}

#[derive(Serialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct DefaultKeyStyle {
    pub loose: KeySubStyle,
    pub pressed: KeySubStyle,
}

impl JsonSchema for DefaultKeyStyle {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "DefaultKeyStyle".into()
    }

    fn schema_id() -> std::borrow::Cow<'static, str> {
        "nuhxboard_types::style::DefaultKeyStyle".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let sub_style = generator.subschema_for::<KeySubStyle>();
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "Loose": sub_style,
                "Pressed": sub_style
            },
            "anyOf": [
                { "required": ["Loose"] },
                { "required": ["Pressed"] }
            ]
        })
    }
}

// Custom impl allows for at most one missing field
impl<'de> Deserialize<'de> for DefaultKeyStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{Error, IgnoredAny, Visitor};

        enum Field {
            Loose,
            Pressed,
            Ignore,
        }

        struct FieldVisitor;
        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("field identifier")
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    0 => Ok(Field::Loose),
                    1 => Ok(Field::Pressed),
                    _ => Ok(Field::Ignore),
                }
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "Loose" => Ok(Field::Loose),
                    "Pressed" => Ok(Field::Pressed),
                    _ => Ok(Field::Ignore),
                }
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    b"Loose" => Ok(Field::Loose),
                    b"Pressed" => Ok(Field::Pressed),
                    _ => Ok(Field::Ignore),
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct SelfVisitor;
        impl<'de> Visitor<'de> for SelfVisitor {
            type Value = DefaultKeyStyle;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct DefaultKeyStyle")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let loose: KeySubStyle = seq.next_element()?.ok_or_else(|| {
                    Error::invalid_length(0, &"struct DefaultKeyStyle with 1-2 elements")
                })?;
                let pressed = seq.next_element()?.unwrap_or(loose.clone());
                Ok(DefaultKeyStyle { loose, pressed })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut loose: Option<KeySubStyle> = None;
                let mut pressed = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Loose => {
                            if loose.is_some() {
                                return Err(Error::duplicate_field("Loose"));
                            }
                            loose = Some(map.next_value()?);
                        }
                        Field::Pressed => {
                            if pressed.is_some() {
                                return Err(Error::duplicate_field("Pressed"));
                            }
                            pressed = Some(map.next_value()?);
                        }
                        Field::Ignore => {
                            let _ = map.next_value::<IgnoredAny>()?;
                        }
                    }
                }

                let (loose, pressed) = match (loose, pressed) {
                    (Some(l), Some(p)) => (l, p),
                    (Some(l), None) => (l.clone(), l),
                    (None, Some(p)) => (p.clone(), p),
                    (None, None) => return Err(Error::missing_field("Loose or Pressed")),
                };

                Ok(DefaultKeyStyle { loose, pressed })
            }
        }

        deserializer.deserialize_struct("DefaultKeyStyle", &["Loose", "Pressed"], SelfVisitor)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct KeyStyle {
    pub loose: Option<KeySubStyle>,
    pub pressed: Option<KeySubStyle>,
}

impl From<DefaultKeyStyle> for KeyStyle {
    fn from(val: DefaultKeyStyle) -> Self {
        Self {
            loose: Some(val.loose),
            pressed: Some(val.pressed),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct KeySubStyle {
    pub background: NohRgb,
    pub text: NohRgb,
    pub outline: NohRgb,
    pub show_outline: bool,
    /// Outline thickness in pixels.
    pub outline_width: u32,
    pub font: Font,
    pub background_image_file_name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Font {
    pub font_family: String,
    /// Font size in pixels.
    pub size: f32,
    pub style: FontStyle,
}

impl From<FontStyle> for gpui::FontWeight {
    fn from(val: FontStyle) -> Self {
        if val.contains(FontStyle::BOLD) {
            gpui::FontWeight::BOLD
        } else {
            gpui::FontWeight::NORMAL
        }
    }
}

impl From<FontStyle> for gpui::FontStyle {
    fn from(val: FontStyle) -> Self {
        if val.contains(FontStyle::ITALIC) {
            gpui::FontStyle::Italic
        } else {
            gpui::FontStyle::Normal
        }
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct FontStyle: u8 {
        const BOLD = 0b0001;
        const ITALIC = 0b0010;
        const UNDERLINE = 0b0100;
        const STRIKETHROUGH = 0b1000;
    }
}

impl Serialize for FontStyle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.bits())
    }
}

impl<'de> Deserialize<'de> for FontStyle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        FontStyle::from_bits(bits).ok_or_else(|| serde::de::Error::custom("Extraneous bits set"))
    }
}

impl JsonSchema for FontStyle {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FontStyle".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        generator.subschema_for::<u8>()
    }
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct MouseSpeedIndicatorStyle {
    pub inner_color: NohRgb,
    pub outer_color: NohRgb,
    pub outline_width: u32,
}

#[derive(Serialize, Deserialize, Debug, JsonSchema, Clone)]
#[serde(tag = "__type")]
pub enum ElementStyle {
    KeyStyle(KeyStyle),
    MouseSpeedIndicatorStyle(MouseSpeedIndicatorStyle),
}

impl ElementStyle {
    pub fn as_key_style(&self) -> Option<&KeyStyle> {
        match self {
            ElementStyle::KeyStyle(key_style) => Some(key_style),
            _ => None,
        }
    }

    pub fn as_mouse_speed_indicator_style(&self) -> Option<&MouseSpeedIndicatorStyle> {
        match self {
            ElementStyle::MouseSpeedIndicatorStyle(mouse_speed_indicator_style) => {
                Some(mouse_speed_indicator_style)
            }
            _ => None,
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Style {
            background_color: NohRgb {
                red: 0.0,
                green: 0.0,
                blue: 100.0,
            },
            background_image_file_name: None,
            default_key_style: DefaultKeyStyle {
                loose: KeySubStyle {
                    background: NohRgb::DEFAULT_GRAY,
                    text: NohRgb::BLACK,
                    outline: NohRgb {
                        red: 0.0,
                        green: 255.0,
                        blue: 0.0,
                    },
                    show_outline: false,
                    outline_width: 1,
                    font: Font::default(),
                    background_image_file_name: None,
                },
                pressed: KeySubStyle {
                    background: NohRgb::WHITE,
                    text: NohRgb::BLACK,
                    outline: NohRgb {
                        red: 0.0,
                        green: 255.0,
                        blue: 0.0,
                    },
                    show_outline: false,
                    outline_width: 1,
                    font: Font::default(),
                    background_image_file_name: None,
                },
            },
            default_mouse_speed_indicator_style: MouseSpeedIndicatorStyle {
                inner_color: NohRgb::DEFAULT_GRAY,
                outer_color: NohRgb::WHITE,
                outline_width: 1,
            },
            element_styles: HashMap::new(),
        }
    }
}

impl Default for Font {
    fn default() -> Self {
        Self {
            font_family: "Courier New".into(),
            size: 10.0,
            style: FontStyle::empty(),
        }
    }
}
