use super::models::*;
use aidoku_test::aidoku_test;

#[aidoku_test]
fn fixture_search_metadata_deserializes() {
    // Synthetic API metadata only; never download gallery content in tests.
    let response: NHentaiSearchResponse = serde_json::from_str(r#"{
        "result":[{"id":1,"media_id":"fixture","thumbnail":"/fixture/cover.png","thumbnail_width":128,"thumbnail_height":128,"english_title":"Fixture","japanese_title":null,"tag_ids":[]}],
        "num_pages":2,"per_page":25,"total":26
    }"#).unwrap();
    assert_eq!(response.result.len(),1);
    assert_eq!(response.result[0].id,1);
    assert_eq!(response.result[0].english_title,"Fixture");
    assert_eq!(response.num_pages,2);
}

#[aidoku_test]
fn malformed_metadata_is_not_silently_accepted() {
    assert!(serde_json::from_str::<NHentaiSearchResponse>(r#"{"result":[{}]}"#).is_err());
}

#[aidoku_test]
fn fixture_image_paths_preserve_host_and_slashes() {
    assert_eq!(make_image_url("fixture/page.png",false),"https://i.nhentai.net/fixture/page.png");
    assert_eq!(make_image_url("/fixture/cover.png",true),"https://t.nhentai.net/fixture/cover.png");
    assert_eq!(make_image_url("https://example.invalid/image.png",false),"https://example.invalid/image.png");
}
