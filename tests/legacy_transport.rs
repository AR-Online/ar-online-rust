//! O transporte do legado: endereço próprio, credencial crua e envio.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use aronline::legacy::{CanalSms, EnvioRequest, SmsTypeSend, DEFAULT_LEGACY_BASE_URL};
use aronline::{Client, DEFAULT_BASE_URL};
use common::{legacy_refusal, FakeApi};
use serde_json::json;
use std::time::Duration;

#[test]
fn o_endereco_do_legado_e_proprio() {
    // Independente do da /v3: são duas APIs, e apontar uma para o ambiente de
    // teste não pode arrastar a outra junto.
    assert_eq!(DEFAULT_LEGACY_BASE_URL, "https://api.ar-online.com.br");
    assert_ne!(DEFAULT_LEGACY_BASE_URL, DEFAULT_BASE_URL);
}

#[test]
fn manda_o_token_cru_sem_bearer() {
    let api = FakeApi::start();
    api.answers(r#"{"idEmail":"8c4813f5"}"#);

    api.legacy_client()
        .legacy()
        .send(&EnvioRequest::new("A", "B", "C"))
        .expect("não esperava erro");

    let seen = api.received();
    // Cru: o gateway quer o JWT sem `Bearer`, o oposto da /v3.
    assert_eq!(seen.authorization.as_deref(), Some("tok-gw"));
    assert_eq!(seen.accept.as_deref(), Some("application/json"));
}

#[test]
fn as_duas_credenciais_no_mesmo_cliente_nao_vazam_uma_na_outra() {
    let api = FakeApi::start();
    let client = api.both();

    api.answers(r#"{"description":"ok"}"#);
    client
        .legacy()
        .status()
        .voz("x")
        .expect("não esperava erro");
    assert_eq!(api.received().authorization.as_deref(), Some("tok-gw"));

    api.answers(r#"{"data":[]}"#);
    client.tags.list().expect("não esperava erro");
    assert_eq!(
        api.received().authorization.as_deref(),
        Some("Bearer tok-v3")
    );
}

#[test]
fn send_posta_o_corpo_como_json_e_devolve_o_id_email() {
    let api = FakeApi::start();
    api.answers(r#"{"idEmail":"8c4813f5-8430-4ad4-ab72-19d7eed39731"}"#);

    let request = EnvioRequest {
        to: Some("joao@exemplo.com".to_owned()),
        custom_id: Some("contrato-4471".to_owned()),
        sms: Some(CanalSms {
            number: Some("11999998888".to_owned()),
            type_send: Some(SmsTypeSend::SomenteSeFalhar),
            ..CanalSms::default()
        }),
        ..EnvioRequest::new("João da Silva", "Documento importante", "<p>Olá.</p>")
    };

    let sent = api
        .legacy_client()
        .legacy()
        .send(&request)
        .expect("não esperava erro");

    assert_eq!(sent.id_email, "8c4813f5-8430-4ad4-ab72-19d7eed39731");

    let seen = api.received();
    assert_eq!(seen.method, "POST");
    assert_eq!(seen.path(), "/gw/email");
    assert_eq!(seen.content_type.as_deref(), Some("application/json"));
    // Só o que foi preenchido viaja: campo ausente não vira `null` no corpo,
    // que o gateway leria como "apague isso".
    assert_eq!(
        seen.body_json(),
        json!({
            "nameTo": "João da Silva",
            "to": "joao@exemplo.com",
            "subject": "Documento importante",
            "content": "<p>Olá.</p>",
            "customID": "contrato-4471",
            "sms": { "number": "11999998888", "typeSend": "1" }
        })
    );
}

#[test]
fn recusa_do_gateway_vira_erro_com_o_corpo_cru() {
    let api = FakeApi::start();
    let body = r#"{"statusCode":400,"message":"O número do destinatário informado é inválido."}"#;
    api.answers_raw(400, body, "application/json");

    let failure = legacy_refusal(
        api.legacy_client()
            .legacy()
            .send(&EnvioRequest::new("A", "B", "C")),
    );

    assert_eq!(failure.status, 400);
    assert_eq!(failure.http_status, 400);
    assert_eq!(
        failure.message,
        "O número do destinatário informado é inválido."
    );
    assert_eq!(failure.body.as_deref(), Some(body));
}

#[test]
fn o_401_cru_do_gateway_chega_inteiro() {
    let api = FakeApi::start();
    api.answers_raw(
        401,
        r#"{"message":"Unauthorized","statusCode":401}"#,
        "application/json",
    );

    let failure = legacy_refusal(api.legacy_client().legacy().templates().list(None));

    assert_eq!(failure.status, 401);
    assert_eq!(failure.http_status, 401);
    assert_eq!(failure.message, "Unauthorized");
}

#[test]
fn sem_a_credencial_do_gateway_falha_antes_do_socket() {
    let api = FakeApi::start();

    let failure = legacy_refusal(api.legacy_anonymous().legacy().status().email("x"));

    assert_eq!(failure.status, 401);
    // Zero: não houve fio, então não houve status de fio.
    assert_eq!(failure.http_status, 0);
    // A mensagem nomeia o método que falta — um 401 do gateway não diria isso.
    assert!(failure.message.contains("legacy_token"));
    assert!(!api.saw_request());
}

#[test]
fn a_credencial_da_v3_nao_serve_para_o_legado() {
    let api = FakeApi::start();

    // Cliente com token da /v3 e nada de gateway: a área de legado continua sem
    // credencial, em vez de mandar a credencial errada e colher um 401.
    let client = Client::builder()
        .base_url(api.base_url())
        .token("tok-v3")
        .legacy_base_url(api.base_url())
        .build();

    let failure = legacy_refusal(client.legacy().status().email("x"));

    assert_eq!(failure.status, 401);
    assert!(!api.saw_request());
}

#[test]
fn duzentos_que_nao_e_json_vira_erro_do_sdk() {
    let api = FakeApi::start();
    api.answers_raw(200, "<html>proxy</html>", "text/html");

    let failure = legacy_refusal(api.legacy_client().legacy().status().email("x"));

    // Erro do SDK, e não um erro de serde vazando: quem viu isso iria procurar
    // defeito no próprio código.
    assert_eq!(failure.status, 200);
    assert!(failure.message.contains("não é o JSON"));
    assert_eq!(failure.body.as_deref(), Some("<html>proxy</html>"));
    // E o corpo cru não é JSON, então não há JSON para devolver.
    assert_eq!(failure.body_json(), None);
}

#[test]
fn quinhentos_e_dois_de_html_falha_com_o_status_e_sem_erro_de_parser() {
    let api = FakeApi::start();
    api.answers_raw(502, "<html>bad gateway</html>", "text/html");

    let failure = legacy_refusal(api.legacy_client().legacy().status().email("x"));

    assert_eq!(failure.status, 502);
    assert_eq!(failure.http_status, 502);
    assert!(failure.message.contains("502"));
}

#[test]
fn endereco_inalcancavel_vira_o_mesmo_erro_com_status_zero() {
    let client = Client::builder()
        .legacy_base_url("http://127.0.0.1:1")
        .legacy_token("tok-gw")
        .timeout(Duration::from_secs(2))
        .build();

    let failure = legacy_refusal(client.legacy().status().email("x"));

    assert_eq!(failure.status, 0);
    assert_eq!(failure.http_status, 0);
    assert_eq!(failure.body, None);
}

#[test]
fn o_endereco_padrao_do_legado_e_producao() {
    // Um milissegundo não fala com produção; o que se prova é que ele TENTOU
    // falar com o endereço certo.
    let failure = legacy_refusal(
        Client::builder()
            .legacy_token("tok-gw")
            .timeout(Duration::from_millis(1))
            .build()
            .legacy()
            .status()
            .email("x"),
    );

    assert_eq!(failure.status, 0);
    assert!(failure.message.contains(DEFAULT_LEGACY_BASE_URL));
}

#[test]
fn endereco_malformado_falha_sem_sair_da_maquina() {
    // Um endereço que nem vira URL: a falha é do SDK, e não uma chamada
    // fantasma que ninguém consegue depurar.
    let client = Client::builder()
        .legacy_base_url("http://exemplo torto")
        .legacy_token("tok-gw")
        .build();

    let failure = legacy_refusal(client.legacy().status().email("x"));

    assert_eq!(failure.status, 0);
    assert!(failure.message.contains("montar a chamada"));
}

#[test]
fn resposta_cortada_no_meio_vira_recusa_e_nao_panico() {
    let api = FakeApi::start();
    api.answers_truncated(r#"{"description":"Ent"#, "application/json");

    let failure = legacy_refusal(api.legacy_client().legacy().status().voz("x"));

    assert!(failure.message.contains("interrompida no meio"));
}

#[test]
fn laudo_cortado_no_meio_tambem_vira_recusa() {
    let api = FakeApi::start();
    api.answers_truncated("%PDF-1.4", "application/pdf");

    let failure = legacy_refusal(api.legacy_client().legacy().laudo("x"));

    assert!(failure.message.contains("interrompida no meio"));
}

#[test]
fn message_que_nao_e_texto_nem_lista_cai_na_frase_padrao() {
    let api = FakeApi::start();
    api.answers_raw(
        500,
        r#"{"statusCode":500,"message":{"erro":"objeto"}}"#,
        "application/json",
    );

    let failure = legacy_refusal(api.legacy_client().legacy().status().email("x"));

    assert_eq!(failure.status, 500);
    assert!(failure.message.contains("sem o corpo de erro esperado"));
}

#[test]
fn escapa_o_id_para_um_id_torto_nao_virar_outro_caminho() {
    let api = FakeApi::start();
    api.answers(r#"{"description":"x"}"#);

    api.legacy_client()
        .legacy()
        .status()
        .voz("../full/x")
        .expect("não esperava erro");

    assert_eq!(api.received().path(), "/gw/voz/..%2Ffull%2Fx");
}
