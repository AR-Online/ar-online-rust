# AR Online — SDK Rust

[![CI](https://github.com/AR-Online/ar-online-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/AR-Online/ar-online-rust/actions/workflows/ci.yml)
[![Licença: Apache 2.0](https://img.shields.io/badge/licen%C3%A7a-Apache%202.0-blue.svg)](LICENSE)

Cliente oficial da API do AR Online para Rust.

Você não monta URL, não escreve cabeçalho, não desembrulha envelope e não lê
status para saber se deu certo. Chama método, recebe struct tipada, e a falha
chega como `ApiError` no `Err`.

## Instalação

```bash
cargo add aronline-sdk
```

Rust 1.79 ou mais novo. A crate se chama `aronline-sdk` no registro, mas a
**lib** se chama `aronline` — é assim que ela entra no seu código.

## Começando

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

O cliente é **síncrono**, e de propósito: um SDK não escolhe o executor
assíncrono da aplicação que o instala. Se o seu programa é async, chame dentro
de `spawn_blocking`.

O token é emitido pelo AR Online. Se você ainda não tem o seu, fale com o
suporte — a API só verifica token, ela não emite.

## O que dá para fazer

### Modelos

```rust
let todos = client.templates.list(None)?;
let do_whatsapp = client.templates.list(Some(Channel::WhatsApp))?;
let um = client.templates.get("9b2f-uuid")?;
```

`Channel` é um enum: `Email`, `Sms`, `WhatsApp`, `Voice` e `Letter`. Valor
fora da lista não compila. `Channel::ALL` tem a lista inteira.

### Etiquetas

```rust
let etiquetas = client.tags.list()?;
let uma = client.tags.get("12")?;
```

Etiqueta é **pessoal**: esses métodos respondem às etiquetas de quem está no
token. Token de integração recebe `403` dizendo isso, em vez de uma lista
vazia — que leria como "você não tem nenhuma".

### Lista de permitidos

```rust
let permitidos = client.allowlist.list()?;
```

Também pessoal, pelo mesmo motivo.

### Frescor dos dados

```rust
let frescor = client.freshness.get()?;

if frescor.sources_behind > 0 {
    eprintln!("{} de {} atrasadas", frescor.sources_behind, frescor.sources_tracked);
}
```

Responde a pergunta prática de quando uma consulta devolve menos do que você
esperava: o defeito é da API, ou a carga está atrasada? Sem esse número as
duas hipóteses parecem a mesma coisa.

Ela responde em **contagens**, não numa lista de tabelas: "46 acompanhadas, 3
atrasadas" responde "está fresco?"; quarenta e seis nomes de tabela é um
relatório que ninguém lê na hora em que a pergunta é feita.

Campo que a API responde `null` é `Option` aqui. `worst_lag_seconds` em `None`
é "nenhuma tabela tem marca de leitura", que não é "está tudo em dia" — por
isso não vira `0`.

### Versão

```rust
let info = client.version.get()?;
println!("{} {}", info.version, info.environment);
```

O único método que funciona **sem token** — é rota aberta. É o primeiro dado
que o suporte pede.

## Quando dá errado

Toda recusa da API vem como `Err(ApiError)`.

```rust
match client.templates.get("nao-existe") {
    Ok(template) => println!("{}", template.name),
    Err(failure) => {
        eprintln!("{}", failure.code);            // "not_found"
        eprintln!("{}", failure.status);          // 404
        eprintln!("{:?}", failure.request_id);    // o número que o suporte pede
    }
}
```

O que vem em `ApiError`:

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

Repetir a chamada é decisão sua — o SDK não repete sozinho.

Rede fora do ar e resposta que não é JSON (um proxy respondendo no lugar da
API) também chegam como `ApiError`, com `code` `unreachable` e
`invalid_response`. Você tem um tipo só para tratar, e ele implementa
`std::error::Error` — então o `?` funciona com `anyhow`, `eyre` e o que você
já usa.

## Configuração

```rust
Client::builder()
    .token("…")                             // opcional: sem ele, só version funciona
    .base_url("https://v3.ar-online.com.br") // padrão; troque para homologação
    .timeout(Duration::from_secs(30))       // padrão
    .build()
```

`Client::builder().build()` já é utilizável: aponta para produção sem
credencial, que é o suficiente para `version.get()`.

## Dependências

Três, e é a única das cinco linguagens que carrega alguma — a `std` não tem
HTTP nem JSON:

| crate | por quê |
|---|---|
| `ureq` (rustls) | HTTP **síncrono**, sem arrastar runtime assíncrono junto |
| `serde` + `serde_json` | desserialização |

## Escopo

Este SDK fala **só a `/v3`**. As rotas `/v1` e `/v2` continuam de pé, mas elas
respondem byte a byte o que as APIs antigas respondiam, idiossincrasias
incluídas — inclusive erro com status `200`. São espelhos para ninguém
precisar migrar no mesmo dia, e um cliente tipado que as "melhorasse"
quebraria exatamente quem elas protegem.

A superfície `/v3` é só de leitura hoje. Escrita entra nos cinco SDKs na mesma
leva em que entrar na API.

Quem precisa do contrato HTTP cru — porque está escrevendo um cliente em outra
linguagem, ou depurando o que passou no fio — encontra em
[docs.ar-online.com.br](https://docs.ar-online.com.br).

## Desenvolvimento

| comando | o que cobra |
|---|---|
| `cargo fmt --check` | formato |
| `cargo clippy --all-targets -- -D warnings` | clippy com `pedantic`, e `unwrap`/`expect`/`panic` proibidos no `src/` |
| `codespell` | ortografia |
| `cargo test --all-targets && cargo test --doc` | testes e os exemplos deste README |
| `cargo llvm-cov --fail-under-lines 95` | cobertura mínima de **95%** |
| `cargo audit` | vulnerabilidade conhecida em dependência |

Hoje: **29 testes** contando os 2 doctests (os exemplos deste README compilam
de verdade), com 96,9% de cobertura.

Os testes sobem um `TcpListener` de verdade numa porta livre e falam HTTP com
ele. Não há dublê: o que este SDK precisa acertar é justamente o fio. E o
servidor de mentira é `std` puro — um SDK que arrasta servidor de teste para
dentro do `Cargo.toml` cobra esse custo de todo mundo que compila a árvore.

## Licença

[Apache 2.0](LICENSE) — © 2026 AR ONLINE TECNOLOGIA LTDA.
