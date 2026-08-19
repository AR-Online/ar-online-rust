# Changelog

Todas as mudanças notáveis deste SDK são documentadas aqui.

Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/)
e versionamento [SemVer](https://semver.org/lang/pt-BR/).

O SDK acompanha a superfície `/v3` da API: rota nova na API vira função nova
aqui, na mesma leva. A área de legado (`client.legacy()`) acompanha o contrato
antigo do gateway: quando a `/v3` ganha o equivalente de uma rota, a função
troca o transporte **sem mudar de assinatura**, e a troca é registrada aqui,
rota a rota.

---

## [Unreleased]

Tudo abaixo entra na **0.1.0**, a primeira publicação. Enquanto a tag não sai,
o conteúdo fica aqui — ver [PUBLICANDO.md](PUBLICANDO.md).

### Added

- **A área de legado** (`client.legacy()`): tudo o que a documentação pública
  do gateway documenta, como função tipada apontando para
  `api.ar-online.com.br` — envio multicanal (`send`), status por canal e
  consolidado (`status().email/sms/whatsapp/voz/carta/full`), comprovante com o
  PDF já decodificado do base64 (`sending_proof`), laudo pericial binário
  (`laudo`), finalizar régua (`finalizar_regua`) e os templates do gateway
  (`templates().list/get/update/deactivate/set_status`). A credencial é a do
  gateway (`legacy_token`), enviada **crua** no `authorization`, sem `Bearer`,
  e o endereço é próprio, independente do da `/v3`.
- **O envelope do gateway resolvido.** A família de templates responde
  `{ data, statusCode }` com **HTTP 200 até em erro**; o SDK lê o código de
  dentro e devolve `Err(LegacyApiError)`, que carrega `status` (o que vale),
  `http_status` (o que o fio disse) e `body` (o corpo cru, com `body_json()`
  para quem quiser o JSON pronto).
- **As quatro convenções de ausência distinguíveis no tipo.** `""` é `String`,
  `null` é `Option`, `{}` é mapa vazio — e a chave que simplesmente não vem é
  `LegacyField`, um enum de três estados (`Missing`, `Null`, `Present`). Em
  Rust o `Option` sozinho não separa "veio nulo" de "não veio", e essas são
  duas coisas diferentes nesta API: o e-mail manda `dateReading: null`, o
  WhatsApp não manda `dateDelivery` nenhum.
- **Fidelidade ao contrato antigo, de propósito.** A voz responde 200 com
  frase para uuid sem registro, e isso não é erro; `finalizar_regua` é GET com
  efeito colateral e o SDK não "conserta"; data do legado fica `String`, porque
  `"18/07/2026 01:01:32"` não nomeia um instante inequívoco. Normalizar
  qualquer uma dessas quebraria quem já integrou.
- **`message` de erro nos dois formatos.** O gateway responde uma frase, mas
  responde uma **lista** de frases quando recusa vários campos de uma vez. Ler
  só o formato de texto derrubaria a desserialização inteira e levaria junto o
  `statusCode` — o SDK aceita os dois e junta a lista numa linha.
- **Tipos dos webhooks** (`WebhookPayloadV1`, `WebhookPayloadV2`) exportados
  para quem recebe as chamadas — o contrato pronto, sem digitar à mão.
- **Nenhuma dependência nova.** O base64 do comprovante é decodificado à mão,
  em sessenta linhas de `std`, e **estrito**: caractere fora do alfabeto,
  padding errado ou comprimento inválido viram `Err`, nunca bytes silenciosos
  que virariam um PDF corrompido no disco de quem chamou.
- **O cliente da `/v3`**, com os cinco recursos que a API responde hoje:
  modelos (listar com filtro de canal, buscar por id), etiquetas (listar,
  buscar), lista de permitidos (listar), frescor da carga e versão. Quem
  instala não escreve HTTP: não monta URL, não põe cabeçalho, não desembrulha
  envelope e não lê status para saber se deu certo.
- **O envelope resolvido por rota.** `templates`, `tags` e `allowlist`
  respondem `{"data": …}`; `freshness` e `version` respondem o objeto direto.
  Desembrulhar tudo, ou nada, quebra metade das chamadas — a escolha é do SDK,
  e quem chama nem sabe que existe envelope.
- **Um tipo de falha só.** Recusa do catálogo, proxy respondendo HTML no lugar
  da API e rede fora do ar chegam todos como `ApiError`. Não há erro de parser
  cru vazando para quem chamou.
- **`request_id` como campo de primeira classe**, e não detalhe enterrado: é
  o primeiro dado que o suporte pede, e um SDK que o engolisse obrigaria quem
  bateu na falha a reproduzir tudo no `curl` só para achar o número.
- **A rota aberta funciona sem credencial.** `version.get()` é pública; um
  cliente construído sem token chama ela, o que serve para conferir a
  instalação antes de ter credencial. Exigir token no construtor tornaria
  inalcançável justamente a rota que o suporte pede primeiro.
- **`Retry-After` já lido em segundos**, com `is_retryable()` dizendo se vale
  repetir. **Repetir é decisão de quem chama** — o SDK não repete sozinho,
  porque só quem chamou sabe se a operação pode acontecer duas vezes.
- **Cliente síncrono, de propósito:** um SDK não escolhe o executor
  assíncrono da aplicação que o instala. Quem é async chama dentro de
  `spawn_blocking`.
- **`unsafe` proibido no crate inteiro**, e `unwrap`/`expect`/`panic` barrados
  no `src/` pelo clippy — um pânico aqui derruba o processo de quem instalou.
- **`ApiError` implementa `std::error::Error`**, então o `?` funciona com
  `anyhow`, `eyre` e o que a aplicação já usa.
- Três dependências, e é a única das cinco linguagens que carrega alguma: a
  `std` não tem HTTP nem JSON.

### Quality

- Portão com lint, formato, ortografia (codespell), **cobertura mínima de
  95%** e auditoria de dependência. Nada com `allow_failure`, que é a mesma
  regra do portão da API.
- Os testes falam com um **servidor de verdade numa porta livre**, não com um
  dublê de HTTP: o que um SDK precisa acertar é justamente o fio — qual rota
  embrulha, como a recusa volta, o que acontece quando algo que não é a API
  responde. Dublê provaria só que o código chama o dublê. As esquisitices do
  legado estão cobertas uma a uma: 200-com-erro dos templates, voz 200 sem
  registro, as quatro convenções de ausência, base64 e binário, 401 cru do
  gateway, `message` em lista, e as duas credenciais sem vazar uma na outra.
- CI em três sistemas operacionais, mais um job que compila na versão mínima
  declarada — `rust-version` é promessa, e sem conferir é promessa que quebra
  na máquina do parceiro, não na nossa.
- Publicação por **Trusted Publishing** no crates.io: a
  `rust-lang/crates-io-auth-action` troca o OIDC do runner por um token que
  expira em minutos, sem `CARGO_REGISTRY_TOKEN` guardado.

Hoje o portão mede: **97 testes (6 deles doctests), 97,9% de cobertura**.

[Unreleased]: https://github.com/AR-Online/ar-online-rust/commits/main
