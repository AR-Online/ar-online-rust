//! Comprovante, laudo e o GET que finaliza a régua.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::{legacy_refusal, FakeApi};

#[test]
fn comprovante_decodifica_o_base64_e_deixa_o_cru_acessivel() {
    let api = FakeApi::start();
    // 'JVBERi0x' é '%PDF-1' — o começo de qualquer PDF de verdade.
    api.answers(r#"{"content":"JVBERi0x"}"#);

    let proof = api
        .legacy_client()
        .legacy()
        .sending_proof("f6cb58f2")
        .expect("não esperava erro");

    assert_eq!(proof.pdf.as_deref(), Some(&b"%PDF-1"[..]));
    assert_eq!(proof.content_base64.as_deref(), Some("JVBERi0x"));
    assert_eq!(proof.message, None);
    assert_eq!(api.received().path(), "/gw/sending-proof/f6cb58f2");
}

#[test]
fn comprovante_aceita_base64_com_padding_e_quebra_de_linha() {
    let api = FakeApi::start();
    // 'JVBERi0xLjQK' com padding e uma quebra no meio: é o mesmo PDF.
    api.answers("{\"content\":\"JVBE\\nRi0xLjQ=\"}");

    let proof = api
        .legacy_client()
        .legacy()
        .sending_proof("f6cb58f2")
        .expect("não esperava erro");

    assert_eq!(proof.pdf.as_deref(), Some(&b"%PDF-1.4"[..]));
}

#[test]
fn comprovante_decodifica_o_alfabeto_inteiro() {
    let api = FakeApi::start();
    // `+` e `/` são os dois símbolos que só aparecem em bytes altos, e é neles
    // que um decodificador escrito à mão costuma errar.
    api.answers(r#"{"content":"+//+"}"#);

    let proof = api
        .legacy_client()
        .legacy()
        .sending_proof("f6cb58f2")
        .expect("não esperava erro");

    assert_eq!(proof.pdf.as_deref(), Some(&[0xFB, 0xFF, 0xFE][..]));
}

#[test]
fn sem_status_de_entrega_vem_mensagem_e_isso_nao_e_erro() {
    let api = FakeApi::start();
    api.answers(
        r#"{"message":"O comprovante para e-mail consultado ainda não possui o status de entrega"}"#,
    );

    let proof = api
        .legacy_client()
        .legacy()
        .sending_proof("f6cb58f2")
        .expect("mensagem de espera não é recusa");

    assert_eq!(proof.pdf, None);
    assert_eq!(proof.content_base64, None);
    assert!(proof
        .message
        .expect("esperava a frase do gateway")
        .contains("ainda não possui o status"));
}

#[test]
fn corpo_sem_content_nem_message_resolve_com_os_tres_campos_vazios() {
    let api = FakeApi::start();
    api.answers("{}");

    let proof = api
        .legacy_client()
        .legacy()
        .sending_proof("f6cb58f2")
        .expect("não esperava erro");

    assert_eq!(proof.pdf, None);
    assert_eq!(proof.content_base64, None);
    assert_eq!(proof.message, None);
}

#[test]
fn base64_com_caractere_fora_do_alfabeto_vira_erro_do_sdk() {
    let api = FakeApi::start();
    api.answers(r#"{"content":"%%%nao-e-base64%%%"}"#);

    let failure = legacy_refusal(api.legacy_client().legacy().sending_proof("f6cb58f2"));

    // Estrito de propósito: decodificador leniente entregaria bytes plausíveis
    // e quem chamou gravaria um PDF corrompido em vez de saber do problema.
    assert!(failure.message.contains("base64"));
    assert_eq!(failure.body.as_deref(), Some("%%%nao-e-base64%%%"));
}

#[test]
fn base64_de_comprimento_invalido_tambem_vira_erro() {
    let api = FakeApi::start();
    // Cinco símbolos: o último grupo tem um símbolo só, que não codifica nada.
    api.answers(r#"{"content":"JVBER"}"#);
    let cinco = legacy_refusal(api.legacy_client().legacy().sending_proof("f6cb58f2"));
    assert!(cinco.message.contains("base64"));

    // Padding no lugar errado.
    api.answers(r#"{"content":"JVBE====","message":null}"#);
    let padding = legacy_refusal(api.legacy_client().legacy().sending_proof("f6cb58f2"));
    assert!(padding.message.contains("base64"));

    // Dado depois do padding: texto cortado e colado de volta.
    api.answers(r#"{"content":"JVBE=Ri0x"}"#);
    let depois = legacy_refusal(api.legacy_client().legacy().sending_proof("f6cb58f2"));
    assert!(depois.message.contains("base64"));
}

#[test]
fn laudo_entrega_os_bytes_do_pdf_como_vieram() {
    let api = FakeApi::start();
    // Bytes de verdade, inclusive os que não são texto: um laudo é binário.
    let pdf = [0x25, 0x50, 0x44, 0x46, 0x2D, 0x31, 0x2E, 0x34, 0x00, 0xFF];
    api.answers_bytes(200, &pdf, "application/pdf");

    let bytes = api
        .legacy_client()
        .legacy()
        .laudo("f6cb58f2")
        .expect("não esperava erro");

    assert_eq!(bytes, pdf);
    assert_eq!(api.received().path(), "/gw/email/laudo/f6cb58f2");
}

#[test]
fn laudo_de_registro_inexistente_responde_404_em_json() {
    let api = FakeApi::start();
    api.answers_raw(
        404,
        r#"{"statusCode":404,"message":"Registro não encontrado"}"#,
        "application/json",
    );

    let failure = legacy_refusal(api.legacy_client().legacy().laudo("sumido"));

    assert_eq!(failure.status, 404);
    assert_eq!(failure.http_status, 404);
    assert_eq!(failure.message, "Registro não encontrado");
}

#[test]
fn laudo_sem_credencial_falha_antes_do_socket() {
    let api = FakeApi::start();

    let failure = legacy_refusal(api.legacy_anonymous().legacy().laudo("f6cb58f2"));

    assert_eq!(failure.status, 401);
    assert!(!api.saw_request());
}

#[test]
fn finalizar_regua_e_get_com_efeito_colateral() {
    let api = FakeApi::start();
    api.answers(r#"{"message":"Regua de notificação finalizada com sucesso"}"#);

    let result = api
        .legacy_client()
        .legacy()
        .finalizar_regua("f6cb58f2")
        .expect("não esperava erro");

    assert_eq!(
        result.message.as_deref(),
        Some("Regua de notificação finalizada com sucesso")
    );
    // GET, e não POST: é o contrato antigo, e o SDK não o "conserta" — quem
    // integrasse contra um POST bateria numa rota que não existe.
    assert_eq!(api.received().method, "GET");
    assert_eq!(
        api.received().path(),
        "/regua-notificacao/finalizar/f6cb58f2"
    );
}
