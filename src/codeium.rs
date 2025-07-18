use serde::{Deserialize, Serialize};
use serde_json::Value;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionParams, InsertTextFormat,
};
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
            extension_name: "helix-gpt".to_string(),
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

        // Detect line ending from file content
        let line_ending = if file_contents.contains("\r\n") {
            "\r\n"
        } else if file_contents.contains("\r") {
            "\r"
        } else {
            "\n"
        }
        .to_string();

        const WIN: usize = 2048;
        let off = analysis.cursor_position;
        let start = off.saturating_sub(WIN);
        let end = (off + WIN).min(file_contents.len());
        let window_text = &file_contents[start..end];

        Self {
            editor_language: "rust".into(),
            language: 36,
            cursor_offset: off - start, // adjust offset into the window
            line_ending,
            absolute_path: document_uri.to_string(),
            relative_path: document_uri.to_string(),
            text: window_text.to_string(),
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

pub struct CodeiumResponse {
    pub raw_completions: Vec<String>,
}

impl CodeiumResponse {
    fn from_codeium_response(resp: &str) -> CodeiumResponse {
        let parsed: Value = serde_json::from_str(resp).unwrap();

        let completion_items = parsed.as_object().unwrap().get("completionItems");

        let raw_completions = match completion_items {
            Some(items) => {
                let items = items.as_array().unwrap();
                items
                    .into_iter()
                    .map(|v| {
                        v.as_object()
                            .unwrap()
                            .get("completion")
                            .unwrap()
                            .as_object()
                            .unwrap()
                            .get("text")
                            .unwrap()
                            .to_string()
                    })
                    .collect()
            }
            None => vec![],
        };

        Self { raw_completions }
    }
}

impl From<CodeiumResponse> for Vec<CompletionItem> {
    fn from(resp: CodeiumResponse) -> Self {
        // Turn the raw completions into completion response
        resp.raw_completions
            .into_iter()
            .enumerate()
            .map(|(index, completion_text)| CompletionItem {
                label: completion_text.clone(),
                kind: Some(CompletionItemKind::TEXT), // or whatever default kind you want
                detail: None,
                documentation: None,
                deprecated: Some(false),
                preselect: Some(index == 0), // preselect first item
                sort_text: Some(format!("{:04}", index)), // sort by index
                filter_text: Some(completion_text.clone()),
                insert_text: Some(completion_text),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                text_edit: None,
                additional_text_edits: None,
                command: None,
                commit_characters: None,
                data: None,
                tags: None,
                ..Default::default()
            })
            .collect()
    }
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

    pub async fn completion(&self, params: CompletionParams) -> CodeiumResponse {
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

        let req = self
            .client
            .post(req_path)
            .header("Content-Type", "application/json")
            .header("Authorization", auth)
            .json(&body);

        let res = req.send().await;
        let text = res.unwrap().text().await.unwrap();
        CodeiumResponse::from_codeium_response(&text)
    }
}
