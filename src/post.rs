use crate::traits::{TimestampId, Validatable};
use serde::{Deserialize, Serialize};
use std::fmt;
use url::Url;
use utoipa::ToSchema;

// Validation
const MAX_SHORT_CONTENT_LENGTH: usize = 1000;
const MAX_LONG_CONTENT_LENGTH: usize = 50000;

/// Represents the type of pubky-app posted data
/// Used primarily to best display the content in UI
#[derive(Serialize, Deserialize, ToSchema, Default, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum PubkyAppPostKind {
    #[default]
    Short,
    Long,
    Image,
    Video,
    Link,
    File,
}

impl fmt::Display for PubkyAppPostKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string_repr = serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default();
        write!(f, "{}", string_repr)
    }
}

/// Used primarily to best display the content in UI
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PubkyAppPostEmbed {
    kind: PubkyAppPostKind, // If a repost: `short`, and uri of the reposted post.
    uri: String,
}

/// Represents raw post in homeserver with content and kind
/// URI: /pub/pubky.app/posts/:post_id
/// Where post_id is CrockfordBase32 encoding of timestamp
///
/// Example URI:
///
/// `/pub/pubky.app/posts/00321FCW75ZFY`
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PubkyAppPost {
    content: String,
    kind: PubkyAppPostKind,
    parent: Option<String>, // If a reply, the URI of the parent post.
    embed: Option<PubkyAppPostEmbed>,
    attachments: Option<Vec<String>>,
}

impl TimestampId for PubkyAppPost {}

impl Validatable for PubkyAppPost {
    fn sanitize(self) -> Self {
        // Sanitize content
        let mut content = self.content.trim().to_string();

        // We are using content keyword `[DELETED]` for deleted posts from a homeserver that still have relationships
        // placed by other users (replies, tags, etc). This content is exactly matched by the client to apply effects to deleted content.
        // Placing posts with content `[DELETED]` is not allowed.
        if content == *"[DELETED]" {
            content = "empty".to_string()
        }

        // Define content length limits based on PubkyAppPostKind
        let max_content_length = match self.kind {
            PubkyAppPostKind::Short => MAX_SHORT_CONTENT_LENGTH,
            PubkyAppPostKind::Long => MAX_LONG_CONTENT_LENGTH,
            _ => MAX_SHORT_CONTENT_LENGTH, // Default limit for other kinds
        };

        let content = content.chars().take(max_content_length).collect::<String>();

        // Sanitize parent URI if present
        let parent = if let Some(uri_str) = &self.parent {
            match Url::parse(uri_str) {
                Ok(url) => Some(url.to_string()), // Valid URI, use normalized version
                Err(_) => None,                   // Invalid URI, discard or handle appropriately
            }
        } else {
            None
        };

        // Sanitize embed if present
        let embed = if let Some(embed) = &self.embed {
            match Url::parse(&embed.uri) {
                Ok(url) => Some(PubkyAppPostEmbed {
                    kind: embed.kind.clone(),
                    uri: url.to_string(), // Use normalized version
                }),
                Err(_) => None, // Invalid URI, discard or handle appropriately
            }
        } else {
            None
        };

        PubkyAppPost {
            content,
            kind: self.kind,
            parent,
            embed,
            attachments: self.attachments,
        }
    }

    fn validate(&self, id: &str) -> Result<(), String> {
        self.validate_id(id)?;

        // Validate content length
        match self.kind {
            PubkyAppPostKind::Short => {
                if self.content.chars().count() > MAX_SHORT_CONTENT_LENGTH {
                    return Err(
                        "Validation Error: Post content exceeds maximum length for Short kind"
                            .into(),
                    );
                }
            }
            PubkyAppPostKind::Long => {
                if self.content.chars().count() > MAX_LONG_CONTENT_LENGTH {
                    return Err(
                        "Validation Error: Post content exceeds maximum length for Short kind"
                            .into(),
                    );
                }
            }
            _ => (),
        };

        // TODO: additional validation?

        Ok(())
    }
}
