# Publicando no crates.io

O workflow [`release.yml`](.github/workflows/release.yml) publica por
**Trusted Publishing**: não existe `CARGO_REGISTRY_TOKEN` guardado no
repositório. A action [`rust-lang/crates-io-auth-action`][auth] troca o OIDC do
runner por um token que expira em minutos.

[auth]: https://github.com/rust-lang/crates-io-auth-action

## O que configurar no crates.io (uma vez)

Em <https://crates.io/> → a crate `aronline-sdk` → *Settings* → *Trusted
Publishing* → *Add*:

| campo | valor |
|---|---|
| Repository owner | `AR-Online` |
| Repository name | `ar-online-rust` |
| Workflow filename | `release.yml` |
| Environment | `crates-io` |

O **Environment** não é enfeite: é ele que impede qualquer outro workflow do
repositório de publicar. O `release.yml` declara `environment: crates-io`, e as
duas pontas têm de dizer o mesmo nome.

⚠️ **A primeira publicação.** O crates.io configura o publisher confiável numa
crate que **já existe**. Então a `0.1.0` sai à mão, do seu terminal:

```bash
cargo login          # token de https://crates.io/settings/tokens
cargo publish
```

Feito isso, configure o Trusted Publishing pela tabela acima e **revogue o
token**. Da 0.1.1 em diante o CI publica sozinho.

⚠️ **O nome não volta.** `aronline-sdk` fica reservado para nós na primeira
publicação, e no crates.io **versão publicada não se apaga** (só se marca como
yanked). Confira o nome e o número antes.

## O que configurar no GitHub (uma vez)

Em *Settings → Environments*, crie o ambiente **`crates-io`**. Ele pode ficar
vazio — não precisa de segredo, porque não há segredo. O que vale a pena ligar:

- **Deployment branches and tags**: restrinja a `v*`, para que só uma tag
  publique;
- **Required reviewers**: se quiser que a publicação espere aprovação humana.

## Como publicar

```bash
# 1. a versão no Cargo.toml
# 2. a tag com o MESMO número, prefixada com v
git tag v0.1.0
git push origin v0.1.0
```

O workflow roda formato, clippy, testes e doctests, **confere que a versão do
`Cargo.toml` é a da tag** e publica.

Tag e `Cargo.toml` divergentes reprovam antes de publicar. É de propósito:
versão publicada não se apaga, e um registro que mente sobre qual código é qual
versão é um problema que não se desfaz.
