#[test]
fn multilingual_text_fixture_round_trips_as_utf8() {
    let text = "English · Français · العربية · 日本語";
    let round_trip = String::from_utf8(text.as_bytes().to_vec()).expect("valid UTF-8");

    assert_eq!(round_trip, text);
    assert!(round_trip.chars().any(|character| character.is_ascii()));
    assert!(round_trip.chars().any(|character| !character.is_ascii()));
}
