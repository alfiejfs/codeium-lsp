use serde::{Deserialize, Serialize};
use tower_lsp::lsp_types::CompletionParams;
use uuid::Uuid;

use crate::util::ContentAnalysis;

pub const PUBLIC_API_KEY: &str = "d49954eb-cfba-4992-980f-d8fb37f0e942";

const API_BASE: &str = "https://web-backend.codeium.com";
const COMPLETIONS_PATH: &str = "exa.language_server_pb.LanguageServerService/GetCompletions";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionBody {
    metadata: CompletionMetadata,
    document: CompletionDocument,
    editor_options: CompletionEditorOptions,
    other_documents: Vec<CompletionDocument>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionMetadata {
    ide_name: String,
    ide_version: String,
    extension_version: String,
    extension_name: String,
    api_key: String,
    session_id: String,
}

impl CompletionMetadata {
    fn from_api(api: &CodeiumApi) -> Self {
        Self {
            ide_name: "web".to_string(),
            ide_version: "unknown".to_string(),
            extension_version: "1.6.13".to_string(),
            extension_name: "codeium-lsp".to_string(),
            api_key: api.key.clone(),
            session_id: api.session_id.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionDocument {
    editor_language: String,
    language: u32,
    cursor_offset: usize,
    line_ending: String,
    absolute_path: String,
    relative_path: String,
    text: String,
}

impl CompletionDocument {
    async fn from_completion_params(params: &CompletionParams) -> Self {
        let document_uri = params.text_document_position.text_document.uri.clone();
        let file_contents = tokio::fs::read_to_string(document_uri.to_file_path().unwrap())
            .await
            .expect("could not read file being edited");

        let analysis = ContentAnalysis::new(
            &file_contents,
            params.text_document_position.position.line as usize,
            params.text_document_position.position.character as usize,
        );

        Self {
            editor_language: "rust".to_string(),
            language: 36,
            cursor_offset: analysis.cursor_position,
            line_ending: "\n".to_string(),
            absolute_path: document_uri.to_string(),
            relative_path: document_uri.to_string(),
            text: file_contents,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionEditorOptions {
    tab_size: usize,
    insert_spaces: bool,
}

impl Default for CompletionEditorOptions {
    fn default() -> Self {
        Self {
            tab_size: 2,
            insert_spaces: true,
        }
    }
}

#[derive(Debug)]
pub struct CodeiumApi {
    session_id: Uuid,
    key: String,
    client: reqwest::Client,
}

impl CodeiumApi {
    pub fn new(key: String) -> Self {
        let session_id = Uuid::new_v4();
        let client = reqwest::Client::new();
        Self {
            session_id,
            key,
            client,
        }
    }

    pub async fn completion(&self, params: CompletionParams) -> String {
        // concat strings hard :(
        let req_path = [API_BASE, COMPLETIONS_PATH].join("/");
        let basic_token = [self.key.clone(), self.session_id.to_string()].join("-");
        let auth = ["Basic".to_string(), basic_token].join(" ");

        let body = CompletionBody {
            metadata: CompletionMetadata::from_api(&self),
            document: CompletionDocument::from_completion_params(&params).await,
            editor_options: CompletionEditorOptions::default(),
            other_documents: vec![],
        };

        let res = self
            .client
            .post(req_path)
            .header("Content-Type", "application/json")
            .header("Authorization", auth)
            .json(&body)
            .send()
            .await;

        res.unwrap().text().await.unwrap()
    }
}
