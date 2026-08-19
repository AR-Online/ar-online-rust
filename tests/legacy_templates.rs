//! Os templates do gateway — a família que responde 200 até em erro.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use aronline::legacy::{GwTemplateType, UpdateGwTemplate};
use common::{legacy_refusal, FakeApi};
use serde_json::json;

const GW_TEMPLATE: &str = r#"{
    "id":"9b2f-uuid","templateId":"hx_boleto_01","nome":"Aviso de boleto",
    "tipo":"whatsapp","conteudo":"Olá {{1}}","variaveis":[{"type":"body"}],
    "metadata":null,"ativo":true,"versao":1,"criadoEm":"2024-10-12T17:11:13.000Z",
    "atualizadoEm":null,"criadoPor":null
}"#;

#[test]
fn os_codigos_de_tipo_sao_os_quatro_do_legado() {
    let codes: Vec<&str> = GwTemplateType::ALL.iter().map(|t| t.as_str()).collect();

    // Código fora deles responde lista vazia, sem erro — o enum move esse
    // engano para a hora de compilar.
    assert_eq!(codes, ["1", "2", "3", "4"]);
    assert_eq!(GwTemplateType::Carta.as_str(), "4");
}

#[test]
fn list_desembrulha_o_envelope_e_devolve_a_lista() {
    let api = FakeApi::start();
    api.answers(&format!(r#"{{"data":[{GW_TEMPLATE}],"statusCode":200}}"#));

    let templates = api
        .legacy_client()
        .legacy()
        .templates()
        .list(None)
        .expect("não esperava erro");

    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].nome, "Aviso de boleto");
    assert_eq!(templates[0].template_id.as_deref(), Some("hx_boleto_01"));
    // Colunas 100% nulas no legado: continuam nulas, e não viram default.
    assert_eq!(templates[0].metadata, None);
    assert_eq!(templates[0].criado_por, None);
    assert_eq!(templates[0].versao, 1);
    assert_eq!(api.received().path(), "/gw/templates");
    assert_eq!(api.received().query(), "");
}

#[test]
fn list_leva_o_codigo_legado_como_filtro() {
    let api = FakeApi::start();
    api.answers(r#"{"data":[],"statusCode":200}"#);

    api.legacy_client()
        .legacy()
        .templates()
        .list(Some(GwTemplateType::WhatsApp))
        .expect("não esperava erro");

    assert_eq!(api.received().query(), "type=1");
}

#[test]
fn resposta_sem_o_envelope_prometido_vira_erro() {
    let api = FakeApi::start();
    api.answers(&format!("[{GW_TEMPLATE}]"));

    let failure = legacy_refusal(api.legacy_client().legacy().templates().list(None));

    assert!(failure.message.contains("envelope"));
}

#[test]
fn envelope_sem_a_chave_data_tambem_vira_erro() {
    let api = FakeApi::start();
    api.answers(r#"{"statusCode":200}"#);

    let failure = legacy_refusal(api.legacy_client().legacy().templates().list(None));

    assert!(failure.message.contains("envelope"));
}

#[test]
fn duzentos_com_404_dentro_do_envelope_vira_erro_tipado() {
    let api = FakeApi::start();
    let wire = r#"{"data":{"error":"Template não encontrado"},"statusCode":404}"#;
    api.answers(wire);

    let failure = legacy_refusal(api.legacy_client().legacy().templates().get("sumido"));

    // O código que vale é o de dentro; o do fio fica registrado ao lado. É o
    // defeito nº 1 de quem integra na mão, e é o que esta área abstrai.
    assert_eq!(failure.status, 404);
    assert_eq!(failure.http_status, 200);
    assert_eq!(failure.message, "Template não encontrado");
    assert_eq!(failure.body.as_deref(), Some(wire));
}

#[test]
fn template_de_outra_entidade_responde_o_403_do_envelope() {
    let api = FakeApi::start();
    api.answers(r#"{"data":{"error":"Acesso negado ao template"},"statusCode":403}"#);

    let failure = legacy_refusal(api.legacy_client().legacy().templates().get("9b2f-uuid"));

    assert_eq!(failure.status, 403);
    assert_eq!(failure.http_status, 200);
    assert_eq!(failure.message, "Acesso negado ao template");
}

#[test]
fn id_que_nao_e_uuid_responde_o_500_do_envelope() {
    let api = FakeApi::start();
    api.answers(r#"{"data":{"error":"Erro ao buscar template(s)"},"statusCode":500}"#);

    let failure = legacy_refusal(api.legacy_client().legacy().templates().get("torto"));

    assert_eq!(failure.status, 500);
    assert_eq!(failure.http_status, 200);
}

#[test]
fn envelope_recusado_sem_frase_ainda_diz_o_codigo() {
    let api = FakeApi::start();
    api.answers(r#"{"data":{},"statusCode":503}"#);

    let failure = legacy_refusal(api.legacy_client().legacy().templates().get("9b2f-uuid"));

    assert_eq!(failure.status, 503);
    assert!(failure.message.contains("503"));
}

#[test]
fn get_busca_pelo_uuid_publico_escapado() {
    let api = FakeApi::start();
    api.answers(&format!(r#"{{"data":{GW_TEMPLATE},"statusCode":200}}"#));

    let template = api
        .legacy_client()
        .legacy()
        .templates()
        .get("9b2f uuid")
        .expect("não esperava erro");

    assert_eq!(template.id, "9b2f-uuid");
    assert_eq!(api.received().path(), "/gw/templates/9b2f%20uuid");
}

#[test]
fn data_com_forma_errada_vira_recusa_e_nao_valor_vazio() {
    let api = FakeApi::start();
    api.answers(r#"{"data":{"nao":"e um template"},"statusCode":200}"#);

    let failure = legacy_refusal(api.legacy_client().legacy().templates().get("9b2f-uuid"));

    assert!(failure.message.contains("forma"));
}

#[test]
fn update_manda_put_com_so_o_que_o_gateway_deixa_editar() {
    let api = FakeApi::start();
    api.answers(r#"{"data":{"ok":true},"statusCode":200}"#);

    let result = api
        .legacy_client()
        .legacy()
        .templates()
        .update(
            "9b2f-uuid",
            &UpdateGwTemplate {
                nome: Some("Novo nome".to_owned()),
                compartilhado_com_entidade: Some(true),
            },
        )
        .expect("não esperava erro");

    assert_eq!(
        result.get("ok").and_then(serde_json::Value::as_bool),
        Some(true)
    );

    let seen = api.received();
    assert_eq!(seen.method, "PUT");
    assert_eq!(seen.path(), "/gw/templates/9b2f-uuid");
    assert_eq!(
        seen.body_json(),
        json!({ "nome": "Novo nome", "compartilhadoComEntidade": true })
    );
}

#[test]
fn update_sem_campo_nenhum_manda_corpo_vazio_e_nao_nulos() {
    let api = FakeApi::start();
    api.answers(r#"{"data":{},"statusCode":200}"#);

    api.legacy_client()
        .legacy()
        .templates()
        .update("9b2f-uuid", &UpdateGwTemplate::default())
        .expect("não esperava erro");

    // `{}` e não `{"nome":null}`: nulo aqui seria "apague o nome".
    assert_eq!(api.received().body_json(), json!({}));
}

#[test]
fn deactivate_e_delete_sem_corpo() {
    let api = FakeApi::start();
    api.answers(r#"{"data":{"ok":true},"statusCode":200}"#);

    api.legacy_client()
        .legacy()
        .templates()
        .deactivate("9b2f-uuid")
        .expect("não esperava erro");

    let seen = api.received();
    assert_eq!(seen.method, "DELETE");
    assert_eq!(seen.path(), "/gw/templates/9b2f-uuid");
    assert!(seen.body.is_empty());
}

#[test]
fn set_status_e_patch_em_status_com_ativo() {
    let api = FakeApi::start();
    api.answers(r#"{"data":{"ok":true},"statusCode":200}"#);

    api.legacy_client()
        .legacy()
        .templates()
        .set_status("9b2f-uuid", false)
        .expect("não esperava erro");

    let seen = api.received();
    assert_eq!(seen.method, "PATCH");
    assert_eq!(seen.path(), "/gw/templates/9b2f-uuid/status");
    assert_eq!(seen.body_json(), json!({ "ativo": false }));
}

#[test]
fn validacao_que_recusa_varios_campos_responde_message_em_lista() {
    let api = FakeApi::start();
    // O NestJS do gateway responde lista de frases quando recusa mais de um
    // campo. Ler só o formato de texto perderia junto o statusCode.
    api.answers_raw(
        400,
        r#"{"statusCode":400,"message":["nome deve ser texto","tipo é obrigatório"],"error":"Bad Request"}"#,
        "application/json",
    );

    let failure = legacy_refusal(
        api.legacy_client()
            .legacy()
            .templates()
            .update("9b2f-uuid", &UpdateGwTemplate::default()),
    );

    assert_eq!(failure.status, 400);
    assert_eq!(failure.message, "nome deve ser texto; tipo é obrigatório");
}

#[test]
fn message_em_lista_vazia_cai_na_frase_padrao() {
    let api = FakeApi::start();
    api.answers_raw(
        400,
        r#"{"statusCode":400,"message":[]}"#,
        "application/json",
    );

    let failure = legacy_refusal(api.legacy_client().legacy().templates().list(None));

    assert_eq!(failure.status, 400);
    assert!(failure.message.contains("sem o corpo de erro esperado"));
}

#[test]
fn templates_sem_credencial_falha_antes_do_socket() {
    let api = FakeApi::start();

    let failure = legacy_refusal(api.legacy_anonymous().legacy().templates().list(None));

    assert_eq!(failure.status, 401);
    assert!(!api.saw_request());
}
