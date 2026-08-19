//! Os recursos, um por família de rota.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use aronline::Channel;
use common::{refusal, FakeApi};

const TEMPLATE: &str = r#"{
    "id":"9b2f-uuid","name":"Aviso de boleto","channel":"whatsapp","subject":null,
    "body":"Olá {{1}}","active":true,"provider_identifier":"hx_boleto_01",
    "variables":[{"name":"1","component":"body","type":"text"}],
    "created_at":"2024-10-12T14:11:13-03:00","updated_at":null
}"#;

#[test]
fn templates_list_decodifica() {
    let api = FakeApi::start();
    api.answers(&format!(r#"{{"data":[{TEMPLATE}]}}"#));

    let templates = api
        .client()
        .templates
        .list(None)
        .expect("não esperava erro");

    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].name, "Aviso de boleto");
    assert_eq!(
        templates[0].provider_identifier.as_deref(),
        Some("hx_boleto_01")
    );
    assert_eq!(templates[0].variables.len(), 1);
    // `type` no fio, `kind` aqui: `type` é palavra reservada do Rust.
    assert_eq!(templates[0].variables[0].kind.as_deref(), Some("text"));

    // subject é null na API — tem de continuar None, não virar string vazia:
    // "sem assunto" e "assunto em branco" são coisas diferentes.
    assert_eq!(templates[0].subject, None);
    assert_eq!(templates[0].updated_at, None);

    assert_eq!(api.received().path(), "/v3/templates");
    assert_eq!(api.received().query(), "");
}

#[test]
fn templates_list_leva_o_canal() {
    let api = FakeApi::start();
    api.answers(r#"{"data":[]}"#);

    api.client()
        .templates
        .list(Some(Channel::WhatsApp))
        .expect("não esperava erro");

    assert_eq!(api.received().query(), "channel=whatsapp");
}

#[test]
fn templates_get_escapa_o_id() {
    let api = FakeApi::start();
    api.answers(&format!(r#"{{"data":{TEMPLATE}}}"#));

    api.client()
        .templates
        .get("../../ops/health")
        .expect("não esperava erro");

    // O que viajou no fio: as barras escapadas. Sem isso, o id levaria a
    // chamada para outra rota.
    assert_eq!(
        api.received().path(),
        "/v3/templates/..%2F..%2Fops%2Fhealth"
    );
}

#[test]
fn templates_get_404() {
    let api = FakeApi::start();
    api.refuses(
        404,
        r#"{"code":"not_found","message":"Modelo não encontrado."}"#,
        &[],
    );

    let failure = refusal(api.client().templates.get("sumido"));

    assert_eq!(failure.status, 404);
    assert_eq!(failure.code, "not_found");
}

#[test]
fn tags_list() {
    let api = FakeApi::start();
    // r##…##: a cor `"#ff0000"` traz `"#`, que fecharia um raw string de um
    // cerquilha só bem no meio do JSON.
    api.answers(
        r##"{"data":[{"id":"12","name":"urgente","color":"#ff0000","created_at":"2024-10-12T14:11:13-03:00"}]}"##,
    );

    let tags = api.client().tags.list().expect("não esperava erro");

    assert_eq!(tags[0].name, "urgente");
    assert_eq!(tags[0].color.as_deref(), Some("#ff0000"));
    assert_eq!(api.received().path(), "/v3/tags");
}

#[test]
fn tags_token_de_integracao_recebe_403() {
    let api = FakeApi::start();
    api.refuses(
        403,
        r#"{"code":"forbidden","message":"Esta rota responde a token de pessoa."}"#,
        &[],
    );

    // 403, e não lista vazia: vazio leria como "você não tem nenhuma".
    let failure = refusal(api.client().tags.list());

    assert_eq!(failure.status, 403);
    assert_eq!(failure.code, "forbidden");
}

#[test]
fn allowlist_list() {
    let api = FakeApi::start();
    api.answers(
        r#"{"data":[{"id":"7","recipient":"alguem@exemplo.com.br","created_at":"2024-10-12T14:11:13-03:00"}]}"#,
    );

    let entries = api.client().allowlist.list().expect("não esperava erro");

    assert_eq!(entries[0].recipient, "alguem@exemplo.com.br");
    assert_eq!(api.received().path(), "/v3/allowlist");
}

#[test]
fn freshness_decodifica() {
    let api = FakeApi::start();
    api.answers(
        r#"{"refreshed_at":"2026-08-18T11:42:03-03:00","last_load_at":"2026-08-18T11:40:00-03:00",
            "worst_lag_seconds":34904,"tables_tracked":46,"tables_never_loaded":2,
            "behind":[{"legacy":"geral.ger_voz","lag_seconds":34904}]}"#,
    );

    let freshness = api.client().freshness.get().expect("não esperava erro");

    assert_eq!(freshness.tables_tracked, 46);
    assert_eq!(freshness.worst_lag_seconds, Some(34904));
    assert_eq!(freshness.behind.len(), 1);
    assert_eq!(freshness.behind[0].legacy, "geral.ger_voz");
    assert_eq!(api.received().path(), "/v3/freshness");
}

#[test]
fn freshness_exige_token() {
    let api = FakeApi::start();

    let failure = refusal(api.anonymous().freshness.get());

    assert_eq!(failure.status, 401);
    assert_eq!(failure.code, "unauthenticated");
}

#[test]
fn version_responde_sem_token() {
    let api = FakeApi::start();
    api.answers(
        r#"{"version":"0.1.0","min_migration":"0025_credentials.sql","environment":"local"}"#,
    );

    let info = api.anonymous().version.get().expect("não esperava erro");

    assert_eq!(info.version, "0.1.0");
    assert_eq!(info.environment, "local");
    assert_eq!(api.received().path(), "/v3/version");
    assert_eq!(api.received().authorization, None);
}

#[test]
fn canais_sao_os_que_a_v3_aceita() {
    // A lista é duplicada do servidor. Este teste é o que torna a duplicação
    // segura: canal novo lá e não aqui faria o SDK recusar valor que a API
    // aceita, e quem chamou culparia o SDK, com razão.
    let values: Vec<&str> = Channel::ALL.iter().map(|c| c.as_str()).collect();

    assert_eq!(values, ["email", "sms", "whatsapp", "voice", "letter"]);
    assert_eq!(Channel::WhatsApp.to_string(), "whatsapp");
}
