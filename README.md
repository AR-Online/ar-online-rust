# AR Online SDK para Rust

[![crates.io](https://img.shields.io/crates/v/aronline-sdk.svg)](https://crates.io/crates/aronline-sdk)
[![docs.rs](https://img.shields.io/docsrs/aronline-sdk)](https://docs.rs/aronline-sdk)
[![CI](https://github.com/AR-Online/ar-online-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/AR-Online/ar-online-rust/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Licença](https://img.shields.io/badge/licen%C3%A7a-Apache--2.0-green.svg)](LICENSE)

Cliente oficial da API da AR Online para Rust.

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

A plataforma tem duas superfícies de API, e cada uma usa uma credencial
diferente. O SDK aceita as duas no mesmo cliente e envia cada uma no formato que
a sua superfície espera.

### Token do gateway (API legada)

É a credencial que você usa para enviar notificações e consultar status hoje.
Solicite em <suporte@ar-online.com.br>. No SDK, ela vai em `legacy_token`.

### Token da API /v3

Solicite em <suporte@ar-online.com.br>. O token fica preso a uma entidade da sua
conta, e é ela que define quais dados ele enxerga — se você precisa consultar
mais de uma, peça um token para cada. O padrão é somente leitura.

O token tem prazo de validade. Token ausente, expirado ou revogado responde
`401`; se um token vazar, peça a revogação e ele deixa de ser aceito na chamada
seguinte.

> **A /v3 ainda não está publicada.** O endereço `v3.ar-online.com.br`, que é o
> padrão do SDK para essa superfície, entra no ar junto com ela — assim como a
> emissão de token por conta própria, na tela *Gerar Token* da documentação, com
> o mesmo usuário e senha do portal. Até lá, a parte da /v3 deste SDK serve para
> desenvolver contra um ambiente de teste, e é o `client.legacy()` que fala com
> a API em produção.

## Primeiros passos

O envio de notificações é feito hoje pela API legada, exposta no SDK em
`client.legacy()`:

```rust
use aronline::legacy::{CanalSms, EnvioRequest};
use aronline::Client;

fn main() -> Result<(), aronline::legacy::LegacyApiError> {
    let client = Client::builder()
        .legacy_token(std::env::var("AR_GW_TOKEN").unwrap_or_default())
        .build();

    let envio = EnvioRequest {
        to: Some("joao@exemplo.com".to_owned()),
        sms: Some(CanalSms {
            number: Some("11999998888".to_owned()),
            ..CanalSms::default()
        }),
        ..EnvioRequest::new(
            "João da Silva",
            "Notificação de vencimento",
            "<p>Prezado João, identificamos uma pendência em seu contrato.</p>",
        )
    };

    let sent = client.legacy().send(&envio)?;

    println!("notificação aceita: {}", sent.id_email);

    Ok(())
}
```

Guarde o `id_email`: é com ele que você consulta o status de qualquer canal e
baixa os comprovantes.

```rust
let status = client.legacy().status().email(&sent.id_email)?;

println!("{}", status.description); // "Processado", "Enviado", "Entregue", "Lido"
```

O cliente é **síncrono**, para não impor um executor assíncrono à aplicação que
o instala. Se o seu programa é async, chame dentro de `spawn_blocking`.

## Referência

### Envio e acompanhamento (`client.legacy()`)

| método | o que faz |
|---|---|
| `legacy().send(&envio)` | envia a notificação em um ou mais canais |
| `legacy().status().email(id)` | status do AR-Email |
| `legacy().status().sms(id)` | status do AR-SMS |
| `legacy().status().whatsapp(id)` | status do AR-WhatsApp |
| `legacy().status().voz(id)` | status do AR-Voz |
| `legacy().status().carta(id)` | status do AR-Cartas, com o rastreio dos Correios |
| `legacy().status().full(id)` | dados de perícia de todos os canais numa chamada |
| `legacy().sending_proof(id)` | comprovante de envio em PDF |
| `legacy().laudo(id)` | laudo pericial em PDF |
| `legacy().finalizar_regua(id)` | encerra a régua de notificação do envio |
| `legacy().templates().list(tipo)` | lista os modelos da sua entidade |
| `legacy().templates().get(id)` | busca um modelo |
| `legacy().templates().update(id, &campos)` | edita nome e compartilhamento |
| `legacy().templates().deactivate(id)` | desativa um modelo |
| `legacy().templates().set_status(id, ativo)` | ativa ou desativa um modelo |

Envio multicanal: cada canal é um bloco opcional no corpo.

```rust
use aronline::legacy::{Anexo, CanalCarta, CanalSms, CanalVoz, CanalWhatsapp, EnvioRequest, SmsTypeSend};

client.legacy().send(&EnvioRequest {
    to: Some("joao@exemplo.com".to_owned()),
    // sua referência, devolvida na consulta de status
    custom_id: Some("contrato-4471".to_owned()),
    attachments: vec![Anexo { name: "contrato.pdf".to_owned(), base64: "…".to_owned() }],
    sms: Some(CanalSms {
        number: Some("11999998888".to_owned()),
        // SomenteSeFalhar: só se o e-mail não for entregue; Sempre: sempre
        type_send: Some(SmsTypeSend::SomenteSeFalhar),
        custom_message: Some("Você recebeu um AR-Email. Acesse: {SHORT_LINK}".to_owned()),
    }),
    whatsapp: Some(CanalWhatsapp { number: Some("11999998888".to_owned()), ..Default::default() }),
    voz: Some(CanalVoz { number: Some("1133334444".to_owned()), ..Default::default() }),
    carta: Some(CanalCarta { name: Some("João da Silva".to_owned()), ..Default::default() }),
    ..EnvioRequest::new("João da Silva", "Notificação de vencimento", "<p>Conteúdo.</p>")
})?;
```

Comprovantes: o comprovante de envio chega em base64 dentro de um JSON e o SDK
já o decodifica; o laudo pericial chega como PDF binário.

```rust
let comprovante = client.legacy().sending_proof(&id_email)?;

match comprovante.pdf {
    Some(bytes) => std::fs::write("comprovante.pdf", bytes)?,
    // ainda sem status de entrega — pergunte de novo mais tarde
    None => println!("{:?}", comprovante.message),
}

std::fs::write("laudo.pdf", client.legacy().laudo(&id_email)?)?;
```

Datas do legado são `String` no formato `"18/07/2026 01:01:32"`, sem fuso. O SDK
não as converte para um tipo de data: o formato não nomeia um instante
inequívoco, e converter significaria chutar um fuso.

Campo que o gateway responde de quatro maneiras diferentes continua distinguível
no tipo — `""`, `null`, `{}` e a chave que simplesmente não vem. As três
primeiras viram `String`, `Option` e mapa vazio; a última é `LegacyField`, que
tem três estados (`Missing`, `Null`, `Present`) justamente para você não
confundir "veio nulo" com "não veio".

### Consultas da API /v3 (`client.*`)

A /v3 é a API nova, com contrato limpo e validação estrita. Hoje ela é somente
de leitura.

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

## Tratamento de erros

Toda recusa vem como `Err`. As duas superfícies têm cada uma o seu tipo, e ambos
implementam `std::error::Error`, então o operador `?` funciona com `anyhow`,
`eyre` e o que você já usa.

A /v3 devolve `ApiError`:

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

A área de legado devolve `LegacyApiError`, com os campos do contrato antigo:

```rust
use aronline::legacy::LegacyApiError;

match client.legacy().templates().get("nao-existe") {
    Ok(template) => println!("{}", template.nome),
    Err(failure) => {
        eprintln!("{}", failure.status);      // 404 — o código que vale
        eprintln!("{}", failure.http_status); // 200 — o que o fio disse
        eprintln!("{:?}", failure.body);      // o corpo cru, como chegou
    }
}
```

| campo | conteúdo |
|---|---|
| `status` | o código que vale, mesmo quando o HTTP respondeu 200 |
| `http_status` | o status que veio no protocolo (`0` quando não houve chamada) |
| `body` | o corpo da resposta, exatamente como chegou |
| `body_json()` | esse mesmo corpo já desserializado, quando é JSON |

O envelope `{ data, statusCode }` dos templates do gateway — que responde HTTP
200 até em erro — é resolvido pelo SDK: o `403`, `404` ou `500` de dentro do
corpo vira `Err`, e você não precisa ler status nenhum para saber se deu certo.

O SDK não repete chamadas automaticamente, porque só quem chamou sabe se a
operação pode acontecer duas vezes.

## Configuração do cliente

```rust
Client::builder()
    .token("…")                                       // credencial da /v3
    .legacy_token("…")                                // credencial do gateway
    .base_url("https://v3.ar-online.com.br")          // padrão
    .legacy_base_url("https://api.ar-online.com.br")  // padrão
    .timeout(Duration::from_secs(30))                 // padrão, vale para as duas
    .build()
```

Cada credencial é opcional: informe só a da superfície que você vai usar. Os
endereços podem ser trocados para apontar a um ambiente de teste, e são
independentes um do outro. Chamada de legado num cliente sem `legacy_token`
falha antes de sair da máquina, dizendo qual credencial falta.

`Client::builder().build()` já é utilizável: aponta para produção sem
credencial, o suficiente para `version.get()`.

Campo que a API responde `null` é `Option`, para que ausência e zero não se
confundam: "nenhuma fonte tem marca de leitura" não é "está tudo em dia".

Os objetos da área de legado usam os nomes de campo **como o gateway os
escreve**, só adaptados à convenção do Rust (`custom_id` para `customID`,
`id_email` para `idEmail`, `nome`, `conteudo`, `laudo`, `regua`). O vocabulário
antigo fica: traduzir criaria nomes que não existem em documentação nenhuma.

## Webhooks

Em vez de consultar o status repetidamente, você pode receber uma chamada `POST`
a cada mudança. A configuração é feita com o suporte, que cadastra o seu endpoint
e os parâmetros de autenticação. O SDK não recebe a requisição por você, mas
exporta os tipos do payload:

```rust
use aronline::legacy::{WebhookPayloadV1, WebhookPayloadV2};
```

Veja <https://docs.ar-online.com.br/webhooks/visao-geral> para o fluxo completo,
incluindo a política de retentativas.

## As duas superfícies, e o caminho entre elas

A **API legada** é a que está em produção hoje e concentra envio, status e
comprovantes. A **/v3** é a API nova, para onde as funcionalidades estão sendo
migradas aos poucos.

Quando uma rota ganha equivalente na /v3, a função correspondente de
`client.legacy()` passa a falar com a /v3 internamente, **sem mudar de
assinatura**. Na prática, você migra atualizando a versão do crate, não
reescrevendo a sua integração. Cada troca dessas é registrada no
[CHANGELOG](CHANGELOG.md).

Equivalências de hoje: a leitura de templates do gateway tem a /v3 em
`client.templates`; envio, status e provas ainda não têm.

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
| Testes | 97, sendo 6 doctests |
| Cobertura de linhas | 97,9% |
| Dependências de produção | 3 |
| `unsafe` | proibido na crate inteira |

Os exemplos da documentação compilam de verdade no CI como doctests, então
documentação desatualizada reprova o build. Os testes sobem um `TcpListener` real
em uma porta livre e falam HTTP com ele; o servidor de teste é `std` pura, para
não cobrar o custo de compilar um servidor de teste de todo mundo que compila a
árvore. A área de legado não trouxe dependência nova: o base64 do comprovante é
decodificado à mão, em sessenta linhas, e de propósito **estrito** — um
decodificador leniente devolveria bytes plausíveis para uma resposta que nunca
foi um PDF.

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
