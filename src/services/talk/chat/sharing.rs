//! Files and objects shared in a conversation: uploading attachments, sharing
//! rich objects and browsing shared items.

use std::time::{SystemTime, UNIX_EPOCH};

use eventful_rs::*;
use serde::Deserialize;

use crate::{
    dto::{
        serde_ext::PhpMap,
        talk::{
            AttachmentFolder, AttachmentMetaData, ChatMessage, PostAttachment, ShareObject,
            SharedItemType, SharedItems, SharedItemsOverview,
        },
    },
    services::{ncclient::NcError, talk::chat_path},
};

service! {
    pub struct ChatShareService;
}

/// A file to upload and post to a conversation.
#[derive(Debug, Clone, Default)]
pub struct UploadAttachment {
    pub file_name: String,
    pub data: Vec<u8>,
    pub meta: AttachmentMetaData,
    /// Recognise the resulting message in the chat feed; generated when `None`.
    pub reference_id: Option<String>,
}

/// Where an uploaded attachment was stored.
#[derive(Debug, Clone)]
pub struct UploadedAttachment {
    /// Path in the user's files.
    pub file_path: String,
    pub reference_id: String,
}

#[asynchronize(pub)]
impl ChatShareService {
    /// Upload a file into the conversation's attachment folder and post it.
    #[asynced]
    pub async fn upload(
        &self,
        token: String,
        upload: UploadAttachment,
    ) -> Result<UploadedAttachment, NcError> {
        let folder = self
            .attachment_folder(token.clone(), vec![upload.file_name.clone()])
            .await?;
        let file_name = folder
            .renames
            .iter()
            .find_map(|renames| renames.get(&upload.file_name))
            .cloned()
            .unwrap_or(upload.file_name);
        let file_path = format!("{}/{file_name}", folder.folder.trim_end_matches('/'));

        self.client
            .put(&self.client.dav_file_path(&file_path)?)
            .body(upload.data)
            .send_discarding_data()
            .await?;

        let reference_id = upload.reference_id.unwrap_or_else(generate_reference_id);
        self.post_attachment(
            token,
            PostAttachment {
                file_path: file_path.clone(),
                reference_id: reference_id.clone(),
                talk_meta_data: serde_json::to_string(&upload.meta)
                    .expect("attachment metadata serializes"),
                ..Default::default()
            },
        )
        .await?;
        Ok(UploadedAttachment {
            file_path,
            reference_id,
        })
    }

    /// The upload folder for a conversation (created if needed), and the names
    /// `file_names` get there to avoid collisions.
    #[asynced]
    pub async fn attachment_folder(
        &self,
        token: String,
        file_names: Vec<String>,
    ) -> Result<AttachmentFolder, NcError> {
        Ok(self
            .client
            .post(&chat_path(&token, "/attachment/folder"))
            .json(&serde_json::json!({ "fileNames": file_names }))
            .send()
            .await?
            .data)
    }

    /// Post a file that is already in the user's storage.
    #[asynced]
    pub async fn post_attachment(
        &self,
        token: String,
        attachment: PostAttachment,
    ) -> Result<(), NcError> {
        self.client
            .post(&chat_path(&token, "/attachment"))
            .json(&attachment)
            .send_discarding_data()
            .await?;
        Ok(())
    }

    /// Share a rich object (location, deck card, ...). Returns the posted message.
    #[asynced]
    pub async fn share_object(
        &self,
        token: String,
        object: ShareObject,
    ) -> Result<Option<ChatMessage>, NcError> {
        Ok(self
            .client
            .post(&chat_path(&token, "/share"))
            .json(&object)
            .send()
            .await?
            .data)
    }

    /// Shared items of one type, newest first, older than `last_known_message_id`
    /// (0 for the newest).
    #[asynced]
    pub async fn items(
        &self,
        token: String,
        item_type: SharedItemType,
        last_known_message_id: i64,
        limit: u32,
    ) -> Result<SharedItems, NcError> {
        let response = self
            .client
            .get(&chat_path(&token, "/share"))
            .query(&serde_json::json!({
                "objectType": item_type,
                "lastKnownMessageId": last_known_message_id,
                "limit": limit,
            }))
            .send::<PhpMap<ChatMessage>>()
            .await?;
        let last_given = response.header_i64("X-Chat-Last-Given");
        let mut messages: Vec<ChatMessage> = response.data.0.into_values().collect();
        messages.sort_by_key(|message| std::cmp::Reverse(message.id));
        Ok(SharedItems {
            messages,
            last_given,
        })
    }

    /// The newest `limit` items of every type.
    #[asynced]
    pub async fn overview(
        &self,
        token: String,
        limit: u32,
    ) -> Result<SharedItemsOverview, NcError> {
        #[derive(Deserialize)]
        struct Overview(PhpMap<PhpMapOrList>);

        /// Each type is a list, or `[]`/`{}` when empty.
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum PhpMapOrList {
            List(Vec<ChatMessage>),
            Map(std::collections::HashMap<String, ChatMessage>),
        }

        let overview = self
            .client
            .get(&chat_path(&token, "/share/overview"))
            .query(&[("limit", limit)])
            .send::<Overview>()
            .await?
            .data;
        Ok(overview
            .0
            .0
            .into_iter()
            .map(|(item_type, items)| {
                let messages = match items {
                    PhpMapOrList::List(messages) => messages,
                    PhpMapOrList::Map(messages) => messages.into_values().collect(),
                };
                (item_type, messages)
            })
            .collect())
    }
}

/// Unique enough to match the echoed message; the server only stores it.
fn generate_reference_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("sea-bunny-{nanos:x}-{:x}", rand_suffix())
}

fn rand_suffix() -> u64 {
    use std::hash::{BuildHasher, RandomState};
    RandomState::new().hash_one(std::thread::current().id())
}
