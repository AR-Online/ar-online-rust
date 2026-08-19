//! Os tipos da área de legado, sem fio no meio.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use aronline::legacy::{
    EnvioRequest, LegacyApiError, LegacyField, SmsTypeSend, StatusEmail, StatusVoz, WebhookChannel,
    WebhookPayloadV1, WebhookPayloadV2,
};

#[test]
fn o_erro_diz_os_dois_status_e_devolve_o_corpo() {
    let failure = LegacyApiError {
        status: 404,
        http_status: 200,
        message: "Template não encontrado".to_owned(),
        body: Some(r#"{"data":{"error":"Template não encontrado"},"statusCode":404}"#.to_owned()),
    };

    // Os dois códigos aparecem: "200 carregando um 404" é justamente o que se
    // quer ver no log ao depurar o contrato antigo.
    assert_eq!(
        failure.to_string(),
        "aronline legado: 404 (http 200): Template não encontrado"
    );
    assert_eq!(
        failure
            .body_json()
            .and_then(|body| body.get("statusCode").and_then(serde_json::Value::as_u64)),
        Some(404)
    );

    // E ele é um erro da linguagem, então o `?` funciona com anyhow e afins.
    let as_error: &dyn std::error::Error = &failure;
    assert!(as_error.to_string().contains("404"));
}

#[test]
fn erro_sem_corpo_nao_inventa_json() {
    let failure = LegacyApiError {
        status: 0,
        http_status: 0,
        message: "sem rede".to_owned(),
        body: None,
    };

    assert_eq!(failure.body_json(), None);
}

#[test]
fn o_campo_do_legado_separa_a_chave_ausente_do_nulo() {
    let ausente: LegacyField<String> = LegacyField::Missing;
    let nulo: LegacyField<String> = LegacyField::Null;
    let presente = LegacyField::Present("18/07/2026 01:01:32".to_owned());

    assert!(ausente.is_missing() && !ausente.is_null());
    assert!(nulo.is_null() && !nulo.is_missing());
    assert!(!presente.is_missing() && !presente.is_null());

    assert_eq!(ausente.value(), None);
    assert_eq!(nulo.clone().into_value(), None);
    assert_eq!(
        presente.value().map(String::as_str),
        Some("18/07/2026 01:01:32")
    );
    assert_eq!(LegacyField::<String>::default(), LegacyField::Missing);
}

#[test]
fn o_tipo_de_envio_do_sms_e_o_codigo_do_legado() {
    assert_eq!(SmsTypeSend::SomenteSeFalhar.as_str(), "1");
    assert_eq!(SmsTypeSend::Sempre.as_str(), "2");
    assert_eq!(
        serde_json::to_string(&SmsTypeSend::Sempre).expect("não esperava erro"),
        r#""2""#
    );
}

#[test]
fn o_envio_comeca_com_os_tres_campos_que_o_gateway_sempre_quer() {
    let envio = EnvioRequest::new("João", "Assunto", "<p>Corpo</p>");

    assert_eq!(envio.name_to, "João");
    assert_eq!(envio.to, None);
    assert!(envio.attachments.is_empty());
    // Nada de opcional viaja: o corpo tem só os três.
    assert_eq!(
        serde_json::to_value(&envio).expect("não esperava erro"),
        serde_json::json!({ "nameTo": "João", "subject": "Assunto", "content": "<p>Corpo</p>" })
    );
}

#[test]
fn o_payload_v1_do_webhook_traz_as_tres_datas_anulaveis() {
    let event: WebhookPayloadV1 = serde_json::from_str(
        r#"{"notificationID":"c62582cc","channel":"email","description":"Falha",
            "dateSent":null,"dateDelivery":null,"dateRead":null,
            "logDate":"18/07/2026 01:01:32"}"#,
    )
    .expect("não esperava erro");

    assert_eq!(event.notification_id, "c62582cc");
    assert_eq!(event.date_sent, None);
    assert_eq!(event.log_date, "18/07/2026 01:01:32");
}

#[test]
fn o_payload_v2_do_webhook_se_le_pelo_canal() {
    let event: WebhookPayloadV2 = serde_json::from_str(
        r#"{"eventVersion":"2.0","occurredAt":"2026-07-18T01:01:51-03:00",
            "notificationID":"c62582cc","channel":"email","status":"Entregue",
            "statusTimestamp":"18/07/2026 01:01:51",
            "payload":{"dateSend":"18/07/2026 01:01:32","dateDelivery":"18/07/2026 01:01:51",
                "dateReading":null,"dateAcceptance":null,"error":false,
                "description":"Entregue","failureReason":null,"customID":null,
                "idEmail":"c62582cc"},
            "metadata":{"webhookVersion":"v2","attempt":1}}"#,
    )
    .expect("não esperava erro");

    assert_eq!(event.channel, WebhookChannel::Email);
    assert_eq!(event.metadata.attempt, 1);
    assert_eq!(
        event.status_timestamp,
        LegacyField::Present("18/07/2026 01:01:51".to_owned())
    );

    let status: StatusEmail = event.payload_as().expect("não esperava erro");
    assert_eq!(status.description, "Entregue");

    // E é por isso que o payload não é um enum sem etiqueta: esta MESMA
    // resposta de e-mail também passa por StatusVoz, que só exige
    // `description`. Sem o `channel` decidindo, a leitura errada passaria
    // despercebida.
    assert!(event.payload_as::<StatusVoz>().is_ok());
}

#[test]
fn o_payload_v2_sem_status_timestamp_nao_inventa_um() {
    let event: WebhookPayloadV2 = serde_json::from_str(
        r#"{"eventVersion":"2.0","occurredAt":"2026-07-18T01:01:51-03:00",
            "notificationID":"c62582cc","channel":"voz","status":"Atendida",
            "payload":{"description":"Atendimento Confirmado"},
            "metadata":{"webhookVersion":"v2","attempt":4}}"#,
    )
    .expect("não esperava erro");

    assert_eq!(event.channel, WebhookChannel::Voz);
    assert!(event.status_timestamp.is_missing());

    let status: StatusVoz = event.payload_as().expect("não esperava erro");
    assert_eq!(status.description, "Atendimento Confirmado");

    // No outro sentido a leitura errada não cola: a voz não tem nada do que o
    // e-mail exige, e o erro do serde chega inteiro para quem chamou.
    assert!(event.payload_as::<StatusEmail>().is_err());
}
