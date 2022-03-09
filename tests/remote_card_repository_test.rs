use anyhow::Result;
use chrono::Local;
use reqwest::blocking::Client;

use cardamom::domain::{card_repositories::RemoteCardRepository, Card, CardRepository};

#[test]
/// Tests the remote card repository methods by running a simple flow create -> read -> update ->
/// delete.
fn test_remote_card_repository() -> Result<()> {
    let host = "http://localhost:5232";
    let client = Client::new();
    let repository = RemoteCardRepository::new(host, &client)?;

    let id = "4d60020b-7ee8-4a36-8d3a-eec1323def45";
    let mut card = Card {
        id: id.to_string(),
        etag: None,
        date: Local::now(),
        raw: [
            "BEGIN:VCARD",
            "VERSION:3.0",
            &format!("UID:{}", id),
            "EMAIL:test@mail.com",
            "FN:Test",
            "N:Nom;Prenom;;;",
            "ORG:Test",
            "TEL;TYPE=pref:06 06 06 06 06",
            "END:VCARD",
            "",
        ]
        .join("\r\n"),
    };

    // Creates a card and checks that the etag is well set.
    repository.create(&mut card)?;
    assert!(card.etag.is_some());

    // Checks that the card has been created.
    let expected_card = repository.read(id)?;
    assert_eq!(expected_card.id, card.id);
    assert_eq!(expected_card.etag, card.etag);
    assert_eq!(expected_card.raw, card.raw);

    // Updates a card and checks that the etag is well changed.
    card.raw = [
        "BEGIN:VCARD",
        "VERSION:3.0",
        &format!("UID:{}", id),
        "EMAIL:test@mail.com",
        "FN:UpdatedTest",
        "N:Nom;Prenom;;;",
        "ORG:UpdatedTest",
        "TEL;TYPE=pref:06 06 06 06 06",
        "END:VCARD",
        "",
    ]
    .join("\r\n");
    repository.update(&mut card)?;
    assert_ne!(expected_card.etag, card.etag);

    // Checks that the card has been updated.
    let expected_card = repository.read(id)?;
    assert_eq!(expected_card.id, card.id);
    assert_eq!(expected_card.etag, card.etag);
    assert_eq!(expected_card.raw, card.raw);

    // Deletes the card.
    repository.delete(&card)?;

    // Checks that the card has been deleted.
    let res = repository.read(id);
    assert_eq!(
        res.unwrap_err().to_string(),
        format!(r#"cannot read card "{}""#, id)
    );

    Ok(())
}
