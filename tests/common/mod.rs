//! Um servidor de verdade numa porta de verdade, e não um dublê de HTTP.
//!
//! O que este SDK precisa acertar É o fio: qual rota embrulha a resposta, como
//! uma recusa volta, o que acontece quando algo que não é a API responde. Um
//! dublê provaria apenas que o código chama o dublê. E ele é `std` puro — um
//! SDK que arrasta servidor de teste para dentro do `Cargo.toml` cobra isso de
//! todo mundo que compila a árvore.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use aronline::legacy::LegacyApiError;
use aronline::{ApiError, Client};
use std::fmt::Write as _;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

/// O que o servidor de mentira responde na próxima chamada.
#[derive(Clone)]
pub struct Reply {
    pub status: u16,
    /// Bytes, e não texto: o laudo é PDF binário, e um `String` obrigaria o
    /// teste do binário a mandar só o que por acaso é UTF-8 válido.
    pub body: Vec<u8>,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    /// Um `Content-Length` mentiroso, para provar o corpo que acaba no meio.
    pub claimed_length: Option<usize>,
}

impl Default for Reply {
    fn default() -> Self {
        Self {
            status: 200,
            body: b"{}".to_vec(),
            content_type: "application/json".to_owned(),
            headers: Vec::new(),
            claimed_length: None,
        }
    }
}

/// O que o servidor de mentira viu.
#[derive(Clone, Default)]
pub struct Received {
    pub method: String,
    /// O alvo cru — caminho e query como viajaram no fio.
    pub target: String,
    pub authorization: Option<String>,
    pub accept: Option<String>,
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}

impl Received {
    /// Só o caminho, sem a query.
    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or("")
    }

    /// Só a query, sem o caminho.
    pub fn query(&self) -> &str {
        self.target.split_once('?').map_or("", |(_, query)| query)
    }

    /// O corpo como texto.
    pub fn body_text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }

    /// O corpo já desserializado, para comparar JSON sem depender da ordem das chaves.
    pub fn body_json(&self) -> serde_json::Value {
        serde_json::from_slice(&self.body).expect("o corpo que chegou não é JSON")
    }
}

pub struct FakeApi {
    port: u16,
    reply: Arc<Mutex<Reply>>,
    received: Arc<Mutex<Option<Received>>>,
}

impl FakeApi {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("não achei porta livre");
        let port = listener.local_addr().expect("sem endereço local").port();

        let reply = Arc::new(Mutex::new(Reply::default()));
        let received = Arc::new(Mutex::new(None));

        let thread_reply = Arc::clone(&reply);
        let thread_received = Arc::clone(&received);

        thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { break };
                let current = thread_reply.lock().expect("reply envenenado").clone();

                serve(stream, &current, &thread_received);
            }
        });

        Self {
            port,
            reply,
            received,
        }
    }

    pub fn base_url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    /// Um cliente apontado para o servidor de mentira, com token.
    pub fn client(&self) -> Client {
        Client::builder()
            .base_url(self.base_url())
            .token("tok")
            .build()
    }

    /// Um cliente sem credencial.
    ///
    /// Um construtor à parte, e não um argumento opcional: um teste que quer
    /// provar o caminho SEM token não pode receber o do padrão.
    pub fn anonymous(&self) -> Client {
        Client::builder().base_url(self.base_url()).build()
    }

    /// Um cliente com a credencial do gateway, e só ela.
    pub fn legacy_client(&self) -> Client {
        Client::builder()
            .legacy_base_url(self.base_url())
            .legacy_token("tok-gw")
            .build()
    }

    /// Um cliente com as duas credenciais, as duas apontadas para cá.
    pub fn both(&self) -> Client {
        Client::builder()
            .base_url(self.base_url())
            .token("tok-v3")
            .legacy_base_url(self.base_url())
            .legacy_token("tok-gw")
            .build()
    }

    /// Um cliente sem a credencial do gateway.
    pub fn legacy_anonymous(&self) -> Client {
        Client::builder().legacy_base_url(self.base_url()).build()
    }

    /// Responde este corpo com 200.
    pub fn answers(&self, body: &str) {
        self.set(Reply {
            body: body.as_bytes().to_vec(),
            ..Reply::default()
        });
    }

    /// Responde o envelope de erro do catálogo com um status.
    pub fn refuses(&self, status: u16, error: &str, headers: &[(&str, &str)]) {
        self.set(Reply {
            status,
            body: format!("{{\"error\":{error}}}").into_bytes(),
            headers: headers
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect(),
            ..Reply::default()
        });
    }

    /// Responde algo que não é JSON — um proxy no meio.
    pub fn answers_raw(&self, status: u16, body: &str, content_type: &str) {
        self.set(Reply {
            status,
            body: body.as_bytes().to_vec(),
            content_type: content_type.to_owned(),
            ..Reply::default()
        });
    }

    /// Responde bytes crus — o laudo em PDF.
    pub fn answers_bytes(&self, status: u16, body: &[u8], content_type: &str) {
        self.set(Reply {
            status,
            body: body.to_vec(),
            content_type: content_type.to_owned(),
            ..Reply::default()
        });
    }

    /// Promete mais corpo do que entrega e fecha a conexão no meio.
    pub fn answers_truncated(&self, body: &str, content_type: &str) {
        self.set(Reply {
            body: body.as_bytes().to_vec(),
            content_type: content_type.to_owned(),
            claimed_length: Some(body.len() + 4_096),
            ..Reply::default()
        });
    }

    /// A requisição que o servidor viu por último.
    pub fn received(&self) -> Received {
        self.received
            .lock()
            .expect("received envenenado")
            .clone()
            .expect("nenhuma requisição chegou ao servidor de mentira")
    }

    /// Se alguma requisição chegou.
    pub fn saw_request(&self) -> bool {
        self.received.lock().expect("received envenenado").is_some()
    }

    fn set(&self, reply: Reply) {
        *self.reply.lock().expect("reply envenenado") = reply;
        *self.received.lock().expect("received envenenado") = None;
    }
}

// O `received` é guardado ANTES de a resposta sair. Fazer o contrário abre uma
// corrida real: o cliente termina de ler a resposta e o teste consulta
// `received()` enquanto esta thread ainda não guardou nada — o que dá
// "nenhuma requisição chegou" num teste que funcionou. Foi assim que a suíte
// ficou instável no CI, falhando um teste diferente a cada execução.
fn serve(
    mut stream: TcpStream,
    reply: &Reply,
    received: &Arc<Mutex<Option<Received>>>,
) -> Option<()> {
    let mut reader = BufReader::new(stream.try_clone().ok()?);
    let mut line = String::new();

    reader.read_line(&mut line).ok()?;

    let mut parts = line.split_whitespace();
    let mut seen = Received {
        method: parts.next()?.to_owned(),
        target: parts.next()?.to_owned(),
        ..Received::default()
    };

    let mut length = 0usize;

    loop {
        let mut header = String::new();

        if reader.read_line(&mut header).ok()? == 0 || header.trim().is_empty() {
            break;
        }

        if let Some((name, value)) = header.split_once(':') {
            match name.trim().to_ascii_lowercase().as_str() {
                "authorization" => seen.authorization = Some(value.trim().to_owned()),
                "accept" => seen.accept = Some(value.trim().to_owned()),
                "content-type" => seen.content_type = Some(value.trim().to_owned()),
                "content-length" => length = value.trim().parse().unwrap_or(0),
                _ => {}
            }
        }
    }

    // O corpo é lido inteiro mesmo quando o teste não vai olhar para ele:
    // fechar a conexão com bytes ainda no buffer derruba a chamada do cliente
    // no Windows antes de ele terminar de ler a resposta.
    if length > 0 {
        seen.body = vec![0; length];
        reader.read_exact(&mut seen.body).ok()?;
    }

    *received.lock().expect("received envenenado") = Some(seen);

    let mut response = format!(
        "HTTP/1.1 {} X\r\nContent-Type: {}\r\nContent-Length: {}\r\nX-Request-Id: req-do-cabecalho\r\nConnection: close\r\n",
        reply.status,
        reply.content_type,
        reply.claimed_length.unwrap_or(reply.body.len()),
    );

    for (name, value) in &reply.headers {
        let _ = write!(response, "{name}: {value}\r\n");
    }

    response.push_str("\r\n");

    stream.write_all(response.as_bytes()).ok()?;
    stream.write_all(&reply.body).ok()?;
    stream.flush().ok()?;

    Some(())
}

/// Extrai a recusa, ou reprova o teste dizendo o que veio no lugar.
pub fn refusal<T: std::fmt::Debug>(result: aronline::Result<T>) -> ApiError {
    match result {
        Ok(value) => panic!("esperava uma recusa da API, veio Ok({value:?})"),
        Err(failure) => failure,
    }
}

/// A mesma coisa, para a área de legado.
pub fn legacy_refusal<T: std::fmt::Debug>(
    result: aronline::legacy::LegacyResult<T>,
) -> LegacyApiError {
    match result {
        Ok(value) => panic!("esperava uma recusa do gateway, veio Ok({value:?})"),
        Err(failure) => failure,
    }
}
