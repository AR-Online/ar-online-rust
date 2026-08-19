# Changelog

Todas as mudanças notáveis deste SDK são documentadas aqui.

Formato baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/)
e versionamento [SemVer](https://semver.org/lang/pt-BR/).

O SDK acompanha a superfície `/v3` da API: rota nova na API vira função nova
aqui, na mesma leva.

---

## [Unreleased]

Tudo abaixo entra na **0.1.0**, a primeira publicação. Enquanto a tag não sai,
o conteúdo fica aqui — ver [PUBLICANDO.md](PUBLICANDO.md).

### Added

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
  responde. Dublê provaria só que o código chama o dublê.
- CI em três sistemas operacionais, mais um job que compila na versão mínima
  declarada — `rust-version` é promessa, e sem conferir é promessa que quebra
  na máquina do parceiro, não na nossa.
- Publicação por **Trusted Publishing** no crates.io: a
  `rust-lang/crates-io-auth-action` troca o OIDC do runner por um token que
  expira em minutos, sem `CARGO_REGISTRY_TOKEN` guardado.

Hoje o portão mede: **29 testes (2 deles doctests), 96,9% de cobertura**.

[Unreleased]: https://github.com/AR-Online/ar-online-rust/commits/main
