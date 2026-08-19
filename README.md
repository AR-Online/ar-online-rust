# AR Online SDK para Rust

[![crates.io](https://img.shields.io/crates/v/aronline-sdk.svg)](https://crates.io/crates/aronline-sdk)
[![docs.rs](https://img.shields.io/docsrs/aronline-sdk)](https://docs.rs/aronline-sdk)
[![CI](https://github.com/AR-Online/ar-online-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/AR-Online/ar-online-rust/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Licença](https://img.shields.io/badge/licen%C3%A7a-Apache--2.0-green.svg)](LICENSE)

Cliente oficial da API da AR Online para Rust.

> **Status:** este SDK cobre as consultas da API /v3, que ainda não está
> publicada — o endereço `v3.ar-online.com.br` entra no ar junto com ela. O
> envio de notificações em produção é feito hoje pela API legada, que ainda não
> está neste SDK. Fale com o suporte antes de planejar uma integração em cima
> dele.

## Sobre a AR Online

A AR Online é uma plataforma brasileira de notificação eletrônica com validade
jurídica. Uma única requisição dispara a notificação em até cinco canais, e cada
etapa do percurso — envio, entrega e leitura — é registrada com carimbo do tempo
emitido por uma Autoridade de Carimbo do Tempo da ICP-Brasil. Esse registro é o
que dá à comunicação o valor de prova documental previsto na MP 2.200-2/2001, e
é o que diferencia a plataforma de um serviço comum de disparo de mensagens.

Os canais disponíveis são:

| canal | o que é |
|---|---|
| AR-Email | e-mail com comprovação de entrega e de leitura |
| AR-SMS | mensagem de texto para o celular do destinatário |
| AR-WhatsApp | notificação por WhatsApp |
| AR-Voz | chamada telefônica automatizada |
| AR-Cartas | carta física registrada, enviada pelos Correios |

Você escolhe quais canais usar em cada envio. O processamento é assíncrono: a
API confirma o recebimento na hora e devolve um identificador, que você usa
depois para consultar o status de cada canal e baixar os comprovantes.

| | |
|---|---|
| Site | <https://www.ar-online.com.br> |
| Documentação da API | <https://docs.ar-online.com.br> |
| Suporte | <suporte@ar-online.com.br> · +55 (11) 4200-7766 |

## Requisitos

- Rust 1.85 ou mais novo

## Instalação

```bash
cargo add aronline-sdk
```

A crate se chama `aronline-sdk` no registro, e a lib se chama `aronline`.

## Autenticação

### Token da API /v3

Solicite em <suporte@ar-online.com.br>. O token fica preso a uma entidade da sua
conta, e é ela que define quais dados ele enxerga — se você precisa consultar
mais de uma, peça um token para cada. O padrão é somente leitura.

O token tem prazo de validade. Token ausente, expirado ou revogado responde
`401`; se um token vazar, peça a revogação e ele deixa de ser aceito na chamada
seguinte.

Quando a /v3 for publicada, a emissão passa a ser por conta própria, na tela
*Gerar Token* da documentação, com o mesmo usuário e senha do portal.

## Primeiros passos

```rust
use aronline::{Channel, Client};

fn main() -> Result<(), aronline::ApiError> {
    let client = Client::builder()
        .token(std::env::var("AR_TOKEN").unwrap_or_default())
        .build();

    for template in client.templates.list(Some(Channel::WhatsApp))? {
        println!("{} {}", template.name, template.variables.len());
    }

    Ok(())
}
```

O cliente é **síncrono**, para não impor um executor assíncrono à aplicação que
o instala. Se o seu programa é async, chame dentro de `spawn_blocking`.

## Referência

Este SDK cobre hoje as consultas da API /v3.

| método | o que faz | precisa de token |
|---|---|---|
| `templates.list(Option<Channel>)` | lista os modelos, com filtro por canal | sim |
| `templates.get(id)` | busca um modelo pelo UUID | sim |
| `tags.list()` · `tags.get(id)` | suas etiquetas | sim |
| `allowlist.list()` | seus destinatários permitidos | sim |
| `freshness.get()` | o atraso da carga de dados | sim |
| `version.get()` | qual versão da API está no ar | não |

### Modelos

```rust
let todos = client.templates.list(None)?;
let do_whatsapp = client.templates.list(Some(Channel::WhatsApp))?;
let um = client.templates.get("9b2f-uuid")?;
```

`Channel` é um enum com `Email`, `Sms`, `WhatsApp`, `Voice` e `Letter`. Um valor
fora da lista não compila, e `Channel::ALL` traz a lista inteira.

### Etiquetas e lista de permitidos

```rust
let etiquetas = client.tags.list()?;
let uma = client.tags.get("12")?;
let permitidos = client.allowlist.list()?;
```

São recursos **pessoais**: respondem o que pertence a quem está no token. Um
token de integração, que não representa uma pessoa, recebe `403` nessas rotas.

### Atraso da carga

```rust
let frescor = client.freshness.get()?;

if frescor.sources_behind > 0 {
    eprintln!("{} de {} atrasadas", frescor.sources_behind, frescor.sources_tracked);
}
```

Serve para responder uma pergunta prática: quando uma consulta devolve menos do
que você esperava, o problema é a API ou a carga de dados está atrasada?

### Versão

```rust
let info = client.version.get()?;
println!("{} {}", info.version, info.environment);
```

É a única chamada que funciona sem token, útil para conferir a instalação antes
de ter uma credencial.

## Envio de notificações

O envio, a consulta de status por canal e os comprovantes estão na API legada do
gateway, que **ainda não está neste SDK** — hoje ela está disponível no
[SDK TypeScript](https://github.com/AR-Online/ar-online-typescript) e chega aqui
nas próximas versões.

Enquanto isso, o contrato HTTP está documentado em
<https://docs.ar-online.com.br>, e a credencial do gateway é emitida pelo
suporte.

## Tratamento de erros

Toda recusa vem como `Err(ApiError)`. Como `ApiError` implementa
`std::error::Error`, o operador `?` funciona com `anyhow`, `eyre` e o que você já
usa.

```rust
match client.templates.get("nao-existe") {
    Ok(template) => println!("{}", template.name),
    Err(failure) => {
        eprintln!("{}", failure.code);         // "not_found"
        eprintln!("{}", failure.status);       // 404
        eprintln!("{:?}", failure.request_id); // informe ao abrir um chamado
    }
}
```

| campo | conteúdo |
|---|---|
| `status` | o status HTTP (`0` quando a API não foi alcançada) |
| `code` | o código do catálogo: `not_found`, `forbidden`, `rate_limited`, … |
| `message` | a mensagem da API, em português |
| `request_id` | identifica a chamada nos nossos registros |
| `field` | o campo recusado, quando a recusa é sobre um campo |
| `details` | uma entrada por campo, em erro de validação |
| `retry_after_seconds` | quantos segundos esperar, em `429` e `503` |
| `is_retryable()` | `true` em `429` e `503` |

Erro de rede e resposta que não é JSON também chegam como `ApiError`.

O SDK não repete chamadas automaticamente, porque só quem chamou sabe se a
operação pode acontecer duas vezes.

## Configuração do cliente

```rust
Client::builder()
    .token("…")                              // opcional: sem ele, só version funciona
    .base_url("https://v3.ar-online.com.br") // padrão
    .timeout(Duration::from_secs(30))        // padrão
    .build()
```

`Client::builder().build()` já é utilizável: aponta para produção sem
credencial, o suficiente para `version.get()`.

Campo que a API responde `null` é `Option`, para que ausência e zero não se
confundam: "nenhuma fonte tem marca de leitura" não é "está tudo em dia".

## Dependências

Esta é a única das cinco linguagens em que o SDK carrega dependências, porque a
`std` não tem cliente HTTP nem JSON.

| crate | por quê |
|---|---|
| `ureq` (com rustls) | HTTP síncrono, sem arrastar um runtime assíncrono junto |
| `serde` + `serde_json` | desserialização |

`unsafe` é proibido na crate inteira (`unsafe_code = "forbid"`), e o clippy barra
`unwrap`, `expect` e `panic` no `src/`: um pânico aqui derrubaria o processo de
quem instalou.

## Desenvolvimento

| comando | o que cobra |
|---|---|
| `cargo fmt --check` | formato |
| `cargo clippy --all-targets -- -D warnings` | clippy `pedantic` |
| `codespell` | ortografia |
| `cargo test --all-targets && cargo test --doc` | testes e exemplos da documentação |
| `cargo llvm-cov --fail-under-lines 95` | cobertura mínima de 95% |
| `cargo audit` | vulnerabilidade conhecida em dependência |

| métrica | valor |
|---|---|
| Testes | 29, sendo 2 doctests |
| Cobertura de linhas | 96,9% |
| Dependências de produção | 3 |
| `unsafe` | proibido na crate inteira |

Os exemplos de `lib.rs` e `client.rs` compilam de verdade no CI como doctests,
então documentação desatualizada reprova o build. Os testes sobem um
`TcpListener` real em uma porta livre e falam HTTP com ele; o servidor de teste é
`std` pura, para não cobrar o custo de compilar um servidor de teste de todo
mundo que compila a árvore.

O CI também compila na versão mínima declarada (1.85), para que `rust-version` no
`Cargo.toml` seja uma promessa conferida.

Para publicar uma versão, veja [PUBLICANDO.md](PUBLICANDO.md).

## Suporte

- Dúvidas de integração e emissão de credenciais: <suporte@ar-online.com.br>
- Telefone: +55 (11) 4200-7766
- Defeitos neste SDK: [issues do repositório](https://github.com/AR-Online/ar-online-rust/issues)

Ao abrir um chamado sobre uma chamada que falhou, informe o `request_id` do erro:
é com ele que localizamos a requisição nos nossos registros.

## Licença

Apache License 2.0 — veja [LICENSE](LICENSE).

© 2026 AR ONLINE TECNOLOGIA LTDA.
