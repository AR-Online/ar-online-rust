# AR Online — SDK Rust

[![CI](https://github.com/AR-Online/ar-online-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/AR-Online/ar-online-rust/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Edition](https://img.shields.io/badge/edition-2021-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2021/index.html)
[![Cobertura](https://img.shields.io/badge/cobertura-96.9%25-success.svg)](#-desenvolvimento)
[![unsafe](https://img.shields.io/badge/unsafe-forbid-success.svg)](#-o-que-ele-resolve)
[![Licença](https://img.shields.io/badge/licen%C3%A7a-Apache--2.0-green.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-n%C3%A3o%20publicado-orange.svg)](#-escopo)

Cliente oficial da API do AR Online para Rust. Você não monta URL, não escreve cabeçalho, não desembrulha envelope e não lê status para saber se deu certo: chama método, recebe struct tipada, e a falha chega como `ApiError` no `Err`.

## ✨ O que ele resolve

- **O envelope não é uniforme** — `templates`, `tags` e `allowlist` respondem `{"data": …}`; `freshness` e `version` respondem o objeto direto. Desembrulhar tudo, ou nada, quebra metade das chamadas. O SDK sabe por rota.
- **Um erro só, e ele é `std::error::Error`** — recusa do catálogo, proxy respondendo HTML e rede fora do ar chegam todos como `ApiError`, então o `?` funciona com `anyhow`, `eyre` e o que você já usa.
- **`request_id` de primeira classe** — é o primeiro dado que o suporte pede. Um SDK que o engolisse obrigaria você a reproduzir a falha no `curl` para achar o número.
- **Rota aberta funciona sem token** — `version` é pública. Cliente construído sem credencial chama ela, o que serve para conferir a instalação antes de ter token.
- **Síncrono, de propósito** — um SDK não escolhe o executor assíncrono da aplicação que o instala. Se o seu programa é async, chame dentro de `spawn_blocking`.
- **`unsafe` proibido no crate inteiro** (`[lints.rust] unsafe_code = "forbid"`), e `unwrap`/`expect`/`panic` barrados no `src/` pelo clippy — um pânico aqui derruba o processo de quem instalou.
- **`None` e `0` não se confundem** — campo que a API responde `null` é `Option`. "Nenhuma fonte tem marca de leitura" não é "está tudo em dia".

## 🚀 Começando

### Instalação

```bash
cargo add aronline-sdk
```

Rust 1.85 ou mais novo. A crate se chama `aronline-sdk` no registro, mas a **lib** se chama `aronline`.

### Primeira chamada

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

O token é emitido pelo AR Online — a API só verifica, ela não emite. Se você ainda não tem o seu, fale com o suporte.

## 🧰 O que dá para fazer

| recurso | métodos | precisa de token |
|---|---|---|
| Modelos | `templates.list(Option<Channel>)` · `templates.get(id)` | sim |
| Etiquetas | `tags.list()` · `tags.get(id)` | sim |
| Lista de permitidos | `allowlist.list()` | sim |
| Frescor dos dados | `freshness.get()` | sim |
| Versão | `version.get()` | **não** |

### Modelos

```rust
let todos = client.templates.list(None)?;
let do_whatsapp = client.templates.list(Some(Channel::WhatsApp))?;
let um = client.templates.get("9b2f-uuid")?;
```

`Channel` é um enum: `Email`, `Sms`, `WhatsApp`, `Voice` e `Letter`. Valor fora da lista não compila. `Channel::ALL` traz a lista inteira.

### Etiquetas e lista de permitidos

```rust
let etiquetas = client.tags.list()?;
let uma = client.tags.get("12")?;
let permitidos = client.allowlist.list()?;
```

Ambas são **pessoais**: respondem ao que pertence a quem está no token. Token de integração recebe `403` dizendo isso — e não uma lista vazia, que leria como "você não tem nenhuma".

### Frescor dos dados

```rust
let frescor = client.freshness.get()?;

if frescor.sources_behind > 0 {
    eprintln!("{} de {} atrasadas", frescor.sources_behind, frescor.sources_tracked);
}
```

Responde a pergunta prática de quando uma consulta devolve menos do que você esperava: o defeito é da API, ou a carga está atrasada? Sem esse número as duas hipóteses parecem a mesma coisa.

Ela responde em **contagens**, não em lista de tabelas: "46 acompanhadas, 3 atrasadas" responde "está fresco?"; quarenta e seis nomes de tabela é relatório que ninguém lê na hora.

### Versão

```rust
let info = client.version.get()?;
println!("{} {}", info.version, info.environment);
```

O único método que funciona **sem token**. É o primeiro dado que o suporte pede.

## ⚠️ Quando dá errado

Toda recusa vem como `Err(ApiError)`.

```rust
match client.templates.get("nao-existe") {
    Ok(template) => println!("{}", template.name),
    Err(failure) => {
        eprintln!("{}", failure.code);         // "not_found"
        eprintln!("{}", failure.status);       // 404
        eprintln!("{:?}", failure.request_id); // o número que o suporte pede
    }
}
```

| campo | o que é |
|---|---|
| `status` | o status HTTP (`0` quando a API nem foi alcançada) |
| `code` | o código do catálogo: `not_found`, `forbidden`, `rate_limited`, … |
| `message` | a mensagem da API, em pt-BR |
| `request_id` | identifica a chamada nos nossos registros — **sempre informe num chamado** |
| `field` | o campo recusado, quando a recusa é sobre um |
| `details` | uma entrada por campo, em erro de validação |
| `retry_after_seconds` | quantos segundos esperar, em `429` e `503` |
| `is_retryable()` | `true` em `429` e `503` |

Repetir é decisão sua — o SDK não repete sozinho, porque só quem chamou sabe se a operação pode acontecer duas vezes.

## ⚙️ Configuração

```rust
Client::builder()
    .token("…")                              // opcional: sem ele, só version funciona
    .base_url("https://v3.ar-online.com.br") // padrão; troque para homologação
    .timeout(Duration::from_secs(30))        // padrão
    .build()
```

`Client::builder().build()` já é utilizável: aponta para produção sem credencial, que é o suficiente para `version.get()`.

## 📦 Dependências

Três, e esta é a única das cinco linguagens que carrega alguma — a `std` não tem HTTP nem JSON.

| crate | por quê |
|---|---|
| `ureq` (rustls) | HTTP **síncrono**, sem arrastar runtime assíncrono junto |
| `serde` + `serde_json` | desserialização |

## 🎯 Escopo

Este SDK fala **só a `/v3`**. As rotas `/v1` e `/v2` continuam de pé, mas respondem byte a byte o que as APIs antigas respondiam, idiossincrasias incluídas — inclusive erro com status `200`. São espelhos para ninguém precisar migrar no mesmo dia, e um cliente tipado que as "melhorasse" quebraria exatamente quem elas protegem.

A superfície `/v3` é **só de leitura** hoje. Escrita entra nos cinco SDKs na mesma leva em que entrar na API.

## 🧪 Desenvolvimento

| comando | o que cobra |
|---|---|
| `cargo fmt --check` | formato |
| `cargo clippy --all-targets -- -D warnings` | clippy `pedantic`, e `unwrap`/`expect`/`panic` proibidos no `src/` |
| `codespell` | ortografia |
| `cargo test --all-targets && cargo test --doc` | testes, e os exemplos da documentação da crate |
| `cargo llvm-cov --fail-under-lines 95` | cobertura mínima de **95%** |
| `cargo audit` | vulnerabilidade conhecida em dependência |

| métrica | valor |
|---|---|
| Testes | 29, sendo 2 doctests |
| Cobertura de linhas | 96,9% |
| Dependências de produção | 3 |
| `unsafe` | proibido no crate inteiro |

Os doctests não são enfeite: os exemplos de `lib.rs` e `client.rs` **compilam de verdade** no CI, então documentação de API que envelhece reprova o build.

Os testes sobem um `TcpListener` **de verdade numa porta livre** e falam HTTP com ele. Não há dublê, e o servidor de mentira é `std` pura — um SDK que arrasta servidor de teste para dentro do `Cargo.toml` cobra esse custo de todo mundo que compila a árvore.

O CI também compila na versão mínima (1.85): `rust-version` no `Cargo.toml` é promessa, e sem conferir é promessa que quebra na máquina do parceiro, não na nossa.

## 📚 Documentação

- [Documentação da API](https://docs.ar-online.com.br) — o contrato HTTP cru
- [docs.rs](https://docs.rs/aronline-sdk) — a referência da crate
- `https://v3.ar-online.com.br/docs/openapi.json` — sempre a lista completa do que está no ar

## 📄 Licença

Apache License 2.0 — veja [LICENSE](LICENSE). © 2026 AR ONLINE TECNOLOGIA LTDA.
