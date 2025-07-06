use codeium_lsp::Lsp;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) =
        LspService::new(|client| Lsp::new(client, codeium_lsp::PUBLIC_API_KEY.to_string()));
    Server::new(stdin, stdout, socket).serve(service).await;

    // Generate something to add 5 + 3 and print it out
}
