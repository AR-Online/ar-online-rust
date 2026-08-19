//! As cinco rotas de status, e as quatro maneiras de dizer "ainda não".

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use aronline::legacy::LegacyField;
use common::{legacy_refusal, FakeApi};

#[test]
fn email_preserva_o_vazio_e_o_nulo_como_convencoes_diferentes() {
    let api = FakeApi::start();
    api.answers(
        r#"{"dateSend":"18/07/2026 01:01:32","dateDelivery":"","dateReading":null,
            "dateAcceptance":null,"error":false,"description":"Enviado",
            "failureReason":null,"customID":"pedido-4471",
            "idEmail":"c62582cc-fc79-4ef5-a20d-27a8476b651d"}"#,
    );

    let status = api
        .legacy_client()
        .legacy()
        .status()
        .email("c62582cc")
        .expect("não esperava erro");

    // As duas convenções na MESMA resposta, cada uma onde o fio a põe. Quem
    // testa `== null` nos quatro campos erra metade deles.
    assert_eq!(status.date_send, "18/07/2026 01:01:32");
    assert_eq!(status.date_delivery, "");
    assert_eq!(status.date_reading, None);
    assert_eq!(status.date_acceptance, None);
    assert_eq!(status.custom_id.as_deref(), Some("pedido-4471"));
    assert!(!status.error);
    // A chave nem veio: some não é o mesmo que veio nula.
    assert!(status.failure_reason_description.is_missing());
    assert_eq!(api.received().path(), "/gw/email/c62582cc");
}

#[test]
fn email_distingue_a_chave_que_some_da_chave_que_veio_nula() {
    let api = FakeApi::start();
    api.answers(
        r#"{"dateSend":"","dateDelivery":"","dateReading":null,"dateAcceptance":null,
            "error":true,"description":"Falha","failureReason":"caixa cheia",
            "failureReasonDescription":null,"customID":null,"idEmail":"c62582cc"}"#,
    );

    let status = api
        .legacy_client()
        .legacy()
        .status()
        .email("c62582cc")
        .expect("não esperava erro");

    // Aqui a chave VEIO, carregando null. É outra coisa do teste anterior, e o
    // tipo mostra a diferença em vez de escondê-la num Option só.
    assert!(status.failure_reason_description.is_null());
    assert!(!status.failure_reason_description.is_missing());
    assert_eq!(status.failure_reason_description.value(), None);
}

#[test]
fn email_de_outra_pessoa_chega_como_o_404_do_gateway() {
    let api = FakeApi::start();
    api.answers_raw(
        404,
        r#"{"message":"E-mail não encontrado"}"#,
        "application/json",
    );

    let failure = legacy_refusal(api.legacy_client().legacy().status().email("sumido"));

    assert_eq!(failure.status, 404);
    assert_eq!(failure.http_status, 404);
    assert_eq!(failure.message, "E-mail não encontrado");
}

#[test]
fn sms_traz_answered_como_lista_de_objetos() {
    let api = FakeApi::start();
    api.answers(
        r#"{"description":"Entregue","dateSend":"18/07/2026 01:01:32","dateReading":null,
            "dateAnswered":null,"answered":[{"resposta":"SIM","em":"18/07/2026 02:00:00"}]}"#,
    );

    let status = api
        .legacy_client()
        .legacy()
        .status()
        .sms("c62582cc")
        .expect("não esperava erro");

    // A documentação antiga diz lista de texto e está errada: quem integrou leu
    // os objetos, então é o objeto que fica.
    assert_eq!(status.answered.len(), 1);
    assert_eq!(
        status.answered[0].get("resposta").and_then(|v| v.as_str()),
        Some("SIM")
    );
    assert_eq!(status.date_send, "18/07/2026 01:01:32");
    assert_eq!(api.received().path(), "/gw/sms/c62582cc");
}

#[test]
fn whatsapp_mantem_sumida_a_data_que_sumiu() {
    let api = FakeApi::start();
    api.answers(
        r#"{"description":"Enviado","dateSent":"18/07/2026 01:01:32","error":false,
            "failureReason":null,"customID":null,"idEmail":"c62582cc"}"#,
    );

    let status = api
        .legacy_client()
        .legacy()
        .status()
        .whatsapp("c62582cc")
        .expect("não esperava erro");

    assert_eq!(
        status.date_sent,
        LegacyField::Present("18/07/2026 01:01:32".to_owned())
    );
    assert!(status.date_delivery.is_missing());
    assert!(status.date_response.is_missing());
    assert!(status.date_access_link.is_missing());
    // Sempre nulo nesta rota, mesmo quando a mensagem tem um: quem precisa
    // dele lê na rota de e-mail.
    assert_eq!(status.custom_id, None);
    assert_eq!(api.received().path(), "/gw/whatsapp/c62582cc");
}

#[test]
fn voz_sem_registro_responde_200_e_isso_nao_e_erro() {
    let api = FakeApi::start();
    api.answers(r#"{"description":"Não há registro de voz para este envio"}"#);

    let status = api
        .legacy_client()
        .legacy()
        .status()
        .voz("qualquer")
        .expect("uuid sem registro de voz não é recusa");

    assert_eq!(status.description, "Não há registro de voz para este envio");
    assert!(status.date_sent.is_missing());
    assert!(status.date_success_call.is_missing());
    assert!(status.link_call.is_missing());
    assert_eq!(api.received().path(), "/gw/voz/qualquer");
}

#[test]
fn voz_conta_so_a_falha_quando_a_ligacao_falhou() {
    let api = FakeApi::start();
    api.answers(r#"{"description":"Não atendida","dateFailureCall":"18/07/2026 09:14:02"}"#);

    let status = api
        .legacy_client()
        .legacy()
        .status()
        .voz("c62582cc")
        .expect("não esperava erro");

    // A API antiga para na primeira etapa de falha: os dois nunca viajam juntos.
    assert_eq!(
        status.date_failure_call.value().map(String::as_str),
        Some("18/07/2026 09:14:02")
    );
    assert!(status.date_success_call.is_missing());
}

#[test]
fn carta_expoe_as_etapas_renomeadas_do_contrato() {
    let api = FakeApi::start();
    api.answers(
        r#"{"description":"Entregue","error":false,"dateProcessing":"05/08/2025 11:00:15",
            "datePreparation":"05/08/2025 11:02:40","dateSent":"27/08/2025 15:23:57",
            "dateDelivery":"02/09/2025 10:11:00","sro":"YQ694562879BR",
            "linkRastreio":"https://rastreamento.correios.com.br/YQ694562879BR"}"#,
    );

    let status = api
        .legacy_client()
        .legacy()
        .status()
        .carta("c62582cc")
        .expect("não esperava erro");

    // O provedor produz datePrepared e dateDelivered; o contrato entrega
    // datePreparation e dateDelivery, e é isso que o SDK expõe.
    assert_eq!(
        status.date_preparation.into_value(),
        Some("05/08/2025 11:02:40".to_owned())
    );
    assert_eq!(
        status.date_delivery.into_value(),
        Some("02/09/2025 10:11:00".to_owned())
    );
    assert_eq!(
        status.sro.value().map(String::as_str),
        Some("YQ694562879BR")
    );
    // Não emitido hoje — e a chave nem vem.
    assert!(status.link_ar_carta_comprovante.is_missing());
    assert_eq!(api.received().path(), "/gw/carta/c62582cc");
}

#[test]
fn full_entrega_o_consolidado_com_os_blocos_crus() {
    let api = FakeApi::start();
    api.answers(
        r#"{"codEmail":12345,
            "statusFull":{"email":[{"label":"Enviado","dateTime":"14/05/2025 17:04:44"}]},
            "lastStatus":{"email":{"label":"Enviado","dateTime":"14/05/2025 17:04:44"}},
            "email":[{"subject":"Documento","remetente":"noreply@empresa.com"}],
            "sms":[{}],"whatsapp":[],"voz":[],"carta":[]}"#,
    );

    let full = api
        .legacy_client()
        .legacy()
        .status()
        .full("f6cb58f2")
        .expect("não esperava erro");

    assert_eq!(full.cod_email, 12345);
    assert_eq!(full.status_full.email.len(), 1);
    assert_eq!(full.status_full.email[0].date_time, "14/05/2025 17:04:44");
    assert_eq!(
        full.last_status.email.map(|event| event.label),
        Some("Enviado".to_owned())
    );
    assert_eq!(full.last_status.sms, None);
    assert_eq!(
        full.email[0].get("remetente").and_then(|v| v.as_str()),
        Some("noreply@empresa.com")
    );
    // A quarta convenção: bloco de detalhe sem dado chega `{}` e continua `{}`.
    assert!(full.sms[0].is_empty());
    assert!(full.carta.is_empty());
    assert_eq!(api.received().path(), "/gw/full/f6cb58f2");
}

#[test]
fn corpo_fora_do_contrato_vira_recusa_com_o_corpo_cru_junto() {
    let api = FakeApi::start();
    // Sem `description`, que o contrato promete em toda resposta de voz.
    api.answers(r#"{"dateSent":"18/07/2026 01:01:32"}"#);

    let failure = legacy_refusal(api.legacy_client().legacy().status().voz("c62582cc"));

    // Falha tipada carregando o corpo, e não um erro de serde solto: dá para
    // ver no erro o que o gateway respondeu de verdade.
    assert_eq!(failure.status, 200);
    assert!(failure.message.contains("não é o JSON"));
    assert!(failure.body_json().is_some());
}
