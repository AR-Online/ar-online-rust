//! O transporte: endereço, cabeçalho, envelope e recusa.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use aronline::{ApiError, Client, DEFAULT_BASE_URL};
use common::{refusal, FakeApi};
use std::time::Duration;

#[test]
fn monta_o_bearer_sozinho() {
    let api = FakeApi::start();
    api.answers(r#"{"data":[]}"#);

    api.client().tags.list().expect("não esperava erro");

    let seen = api.received();
    assert_eq!(seen.authorization.as_deref(), Some("Bearer tok"));
    assert_eq!(seen.accept.as_deref(), Some("application/json"));
    assert_eq!(seen.method, "GET");
}

#[test]
fn rota_aberta_nao_manda_credencial() {
    let api = FakeApi::start();
    api.answers(r#"{"version":"0.1.0","min_migration":"x","environment":"local"}"#);

    api.client().version.get().expect("não esperava erro");

    assert_eq!(api.received().authorization, None);
}

#[test]
fn desembrulha_o_data() {
    let api = FakeApi::start();
    api.answers(r#"{"data":[{"id":"1","name":"urgente","color":null,"created_at":"x"}]}"#);

    let tags = api.client().tags.list().expect("não esperava erro");

    assert_eq!(tags.len(), 1);
    assert_eq!(tags[0].name, "urgente");
}

#[test]
fn entrega_direto_quando_a_rota_nao_envelopa() {
    let api = FakeApi::start();
    api.answers(
        r#"{"refreshed_at":null,"last_load_at":null,"worst_lag_seconds":null,
            "tables_tracked":46,"tables_never_loaded":2,"behind":[]}"#,
    );

    let freshness = api.client().freshness.get().expect("não esperava erro");

    assert_eq!(freshness.tables_tracked, 46);
    // Nulo, e não 0: "nenhuma tabela tem marca" não é "está tudo em dia".
    assert_eq!(freshness.worst_lag_seconds, None);
}

#[test]
fn recusa_quando_a_rota_promete_data_e_nao_entrega() {
    let api = FakeApi::start();
    api.answers(r#"{"tags":[]}"#);

    let failure = refusal(api.client().tags.list());

    assert_eq!(failure.code, "invalid_response");
}

#[test]
fn recusa_carrega_tudo_que_o_corpo_traz() {
    let api = FakeApi::start();
    api.refuses(
        404,
        r#"{"code":"not_found","message":"Etiqueta não encontrada.","request_id":"req-do-corpo"}"#,
        &[],
    );

    let failure = refusal(api.client().tags.get("1"));

    assert_eq!(failure.status, 404);
    assert_eq!(failure.code, "not_found");
    assert_eq!(failure.message, "Etiqueta não encontrada.");
    assert_eq!(failure.request_id.as_deref(), Some("req-do-corpo"));
    assert!(!failure.is_retryable());
}

#[test]
fn cai_no_request_id_do_cabecalho() {
    let api = FakeApi::start();
    api.refuses(
        403,
        r#"{"code":"forbidden","message":"sem permissão"}"#,
        &[],
    );

    let failure = refusal(api.client().tags.list());

    assert_eq!(failure.request_id.as_deref(), Some("req-do-cabecalho"));
}

#[test]
fn le_o_retry_after_e_marca_repetivel() {
    let api = FakeApi::start();
    api.refuses(
        429,
        r#"{"code":"rate_limited","message":"aguarde"}"#,
        &[("Retry-After", "30")],
    );

    let failure = refusal(api.client().tags.list());

    assert_eq!(failure.retry_after_seconds, Some(30.0));
    assert!(failure.is_retryable());
}

#[test]
fn ignora_retry_after_que_nao_eh_numero() {
    let api = FakeApi::start();
    api.refuses(
        503,
        r#"{"code":"unavailable","message":"volte depois"}"#,
        &[("Retry-After", "Wed, 21 Oct 2026 07:28:00 GMT")],
    );

    let failure = refusal(api.client().freshness.get());

    // Nada, e não um atraso chutado: atraso errado é pior que nenhum.
    assert_eq!(failure.retry_after_seconds, None);
    assert!(failure.is_retryable());
}

#[test]
fn carrega_field_e_details() {
    let api = FakeApi::start();
    api.refuses(
        400,
        r#"{"code":"invalid_value","message":"fora da lista","field":"channel",
            "details":[{"field":"channel","code":"invalid_value","message":"fora da lista"}]}"#,
        &[],
    );

    let failure = refusal(api.client().templates.list(None));

    assert_eq!(failure.field.as_deref(), Some("channel"));
    assert_eq!(failure.details.len(), 1);
    assert_eq!(failure.details[0].code, "invalid_value");
}

#[test]
fn resposta_que_nao_eh_json_vira_api_error() {
    let api = FakeApi::start();
    api.answers_raw(502, "<html>bad gateway</html>", "text/html");

    let failure = refusal(api.client().tags.list());

    assert_eq!(failure.status, 502);
    assert_eq!(failure.code, "invalid_response");
}

#[test]
fn duzentos_que_nao_eh_json_tambem() {
    let api = FakeApi::start();
    api.answers_raw(200, "nem json", "text/plain");

    let failure = refusal(api.client().freshness.get());

    assert_eq!(failure.code, "invalid_response");
}

#[test]
fn endereco_fora_do_ar_vira_unreachable() {
    let client = Client::builder()
        .base_url("http://127.0.0.1:1")
        .token("tok")
        .timeout(Duration::from_secs(2))
        .build();

    let failure = refusal(client.freshness.get());

    assert_eq!(failure.status, 0);
    assert_eq!(failure.code, "unreachable");
}

#[test]
fn rota_autenticada_sem_token_falha_antes_de_sair() {
    let api = FakeApi::start();

    let failure = refusal(api.anonymous().tags.list());

    assert_eq!(failure.status, 401);
    assert_eq!(failure.code, "unauthenticated");
    assert_eq!(failure.request_id, None);
    // E não saiu da máquina.
    assert!(!api.saw_request());
}

#[test]
fn display_mostra_o_que_o_suporte_pede() {
    let mut failure = ApiError::local(404, "not_found", "sumiu".to_owned());
    failure.request_id = Some("r-9".to_owned());

    assert_eq!(
        failure.to_string(),
        "aronline: not_found (404): sumiu [request_id=r-9]"
    );

    let sem = ApiError::local(401, "unauthenticated", "sem token".to_owned());
    assert_eq!(
        sem.to_string(),
        "aronline: unauthenticated (401): sem token"
    );
}

#[test]
fn base_url_padrao_e_producao() {
    // Sem servidor: só o endereço, que é o que interessa aqui.
    let failure = refusal(
        Client::builder()
            .token("tok")
            .timeout(Duration::from_millis(1))
            .build()
            .tags
            .list(),
    );

    // Um milissegundo não fala com produção; o que se prova é que ele TENTOU.
    assert_eq!(failure.code, "unreachable");
    assert!(failure.message.contains(DEFAULT_BASE_URL));
}
