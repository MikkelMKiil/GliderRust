use glider_rust::profile::load_profile;

#[test]
fn parses_simple_legacy_profile() {
    let xml = r#"
<GlideProfile>
  <Name>Test Route</Name>
  <MinLevel>1</MinLevel>
  <MaxLevel>10</MaxLevel>
  <Waypoint>-8949.12 -132.44</Waypoint>
  <Waypoint>-8952.00 -140.00</Waypoint>
</GlideProfile>
"#;

    let file_path = "tests/tmp_profile.xml";
    std::fs::write(file_path, xml).expect("write profile fixture");

    let parsed = load_profile(file_path).expect("parse profile");

    std::fs::remove_file(file_path).expect("cleanup fixture");

    assert_eq!(parsed.name.as_deref(), Some("Test Route"));
    assert_eq!(parsed.min_level, Some(1));
    assert_eq!(parsed.max_level, Some(10));
    assert_eq!(parsed.waypoints.len(), 2);
    assert!((parsed.waypoints[0].x + 8949.12).abs() < 0.01);
}

#[test]
fn parses_real_remake_profile_file() {
    let profile_path =
        "..\\..\\..\\Remake\\01-04 Elwynn Forest - Northshire Valley (kobold-wolves-defias).xml";

    assert!(
        std::path::Path::new(profile_path).exists(),
        "expected real profile file to exist at {profile_path}"
    );

    let parsed = load_profile(profile_path).expect("parse real remake profile");

    assert_eq!(parsed.min_level, Some(1));
    assert_eq!(parsed.max_level, Some(5));
    assert!(
        parsed.waypoints.len() > 100,
        "expected large waypoint route, got {}",
        parsed.waypoints.len()
    );
}
