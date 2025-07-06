use codeium::CodeiumApi;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::{
    CompletionItem, CompletionOptions, CompletionParams, CompletionResponse, InitializeParams,
    InitializeResult, InitializedParams, MessageType, ServerCapabilities,
};
use tower_lsp::{Client, LanguageServer};

mod codeium;
mod util;

pub use codeium::PUBLIC_API_KEY;
use util::log;

#[derive(Debug)]
pub struct Lsp {
    client: Client,
    codeium: CodeiumApi,
}

impl Lsp {
    pub fn new(client: Client, key: String) -> Self {
        Self {
            client,
            codeium: CodeiumApi::new(key),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Lsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        " ".to_string(),
                        "(".to_string(),
                        "{".to_string(),
                    ]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let msg = self.codeium.completion(params).await;

        log("into called").await;
        let completions: Vec<_> = msg.into();
        log(format!("into done. found {} completions", completions.len()).as_str()).await;

        for completion in &completions {
            log(format!("completion: {:?}", completion).as_str()).await;
        }

        let response = CompletionResponse::Array(completions);

        Ok(Some(response))
    }
}
